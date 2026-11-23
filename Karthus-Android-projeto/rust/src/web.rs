//! Pesquisa na web para a IA (só quando o usuário liga "Pesquisar na web"). Lê páginas públicas e devolve trechos com fontes.
use serde_json::{json, Value};
use std::io::Read;
use std::time::Duration;

fn busca_base() -> String { std::env::var("HIRELOD_WEB_BUSCA").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "https://html.duckduckgo.com/html/".into()) }

fn agente() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(6)).timeout(Duration::from_secs(12)).redirects(4)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Karthus/6.1").build()
}

/// Só endereços públicos http(s): nunca o próprio computador nem a rede de casa.
pub fn url_publica(u: &str) -> bool {
    if cfg!(debug_assertions) && std::env::var("HIRELOD_WEB_TESTE").is_ok() && u.contains("127.0.0.1:8898") { return true; } // só nos testes (build de desenvolvimento)
    let l = u.to_lowercase();
    let resto = if let Some(r) = l.strip_prefix("https://") { r } else if let Some(r) = l.strip_prefix("http://") { r } else { return false };
    let host = resto.split(['/', '?', '#']).next().unwrap_or("").rsplit('@').next().unwrap_or("");
    let h = host.split(':').next().unwrap_or("");
    if h.is_empty() || h == "localhost" || h.ends_with(".local") || h.ends_with(".internal") || host.starts_with('[') { return false; }
    if let Ok(ip) = h.parse::<std::net::Ipv4Addr>() { return !(ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified() || ip.octets()[0] == 100 && ip.octets()[1] >= 64 && ip.octets()[1] < 128); }
    h.contains('.')
}

fn decodificar(s: &str) -> String {
    let b = s.as_bytes();
    let mut o = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() + 0 && i + 2 <= b.len() - 1 + 0 {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) { o.push(v); i += 3; continue; }
        }
        o.push(if b[i] == b'+' { b' ' } else { b[i] });
        i += 1;
    }
    String::from_utf8_lossy(&o).to_string()
}

pub fn limpar_html(h: &str) -> String {
    let mut s = h.to_string();
    for tag in ["script", "style", "noscript", "svg", "nav", "footer", "header"] {
        loop {
            let l = s.to_lowercase();
            let a = match l.find(&format!("<{}", tag)) { Some(a) => a, None => break };
            let f = match l[a..].find(&format!("</{}>", tag)) { Some(f) => a + f + tag.len() + 3, None => s.len() };
            s.replace_range(a..f.min(s.len()), " ");
        }
    }
    let mut o = String::new();
    let mut dentro = false;
    for c in s.chars() {
        match c { '<' => { dentro = true; o.push(' '); } '>' => dentro = false, _ if !dentro => o.push(c), _ => {} }
    }
    let o = o.replace("&nbsp;", " ").replace("&amp;", "&").replace("&quot;", "\"").replace("&#x27;", "'").replace("&#39;", "'").replace("&lt;", "<").replace("&gt;", ">");
    o.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Analisa a página de resultados do DuckDuckGo (versão HTML).
pub fn analisar_resultados(h: &str) -> Vec<(String, String, String)> {
    let mut r = Vec::new();
    for bloco in h.split("class=\"result__a\"").skip(1) {
        let href = bloco.split("href=\"").nth(1).and_then(|x| x.split('"').next()).unwrap_or("");
        let url = if let Some(p) = href.split("uddg=").nth(1) { decodificar(p.split('&').next().unwrap_or("")) } else if href.starts_with("//") { format!("https:{}", href) } else { href.to_string() };
        let titulo = limpar_html(bloco.split_once('>').map(|x| x.1.split("</a>").next().unwrap_or("")).unwrap_or(""));
        let snip = bloco.split("result__snippet").nth(1).map(|x| limpar_html(x.split("</a>").next().unwrap_or("").split_once('>').map(|y| y.1).unwrap_or(""))).unwrap_or_default();
        if url_publica(&url) && !titulo.is_empty() { r.push((titulo, url, snip)); }
        if r.len() >= 6 { break; }
    }
    r
}

pub fn buscar(consulta: &str) -> Result<Vec<(String, String, String)>, String> {
    let base = busca_base();
    let sep = if base.contains('?') { '&' } else { '?' };
    let url = format!("{}{}q={}", base, sep, consulta.chars().take(200).collect::<String>().replace('%', "").replace('&', " ").replace('#', " ").replace(' ', "+"));
    let t = agente().get(&url).call().map_err(|e| format!("Sem acesso à busca na web: {}", e))?.into_string().map_err(|e| e.to_string())?;
    let r = analisar_resultados(&t);
    if r.is_empty() { Err("A busca não devolveu resultados.".into()) } else { Ok(r) }
}

pub fn ler_pagina(url: &str, max: usize) -> Option<String> {
    if !url_publica(url) { return None; }
    let r = agente().get(url).call().ok()?;
    let tipo = r.header("Content-Type").unwrap_or("").to_lowercase();
    if !(tipo.contains("text/html") || tipo.contains("text/plain") || tipo.is_empty()) { return None; }
    let mut b = Vec::new();
    r.into_reader().take(600_000).read_to_end(&mut b).ok()?;
    let t = limpar_html(&String::from_utf8_lossy(&b));
    Some(t.chars().take(max).collect())
}

/// Pesquisa e monta o contexto numerado para a IA + a lista de fontes para a tela.
pub fn contexto(consulta: &str) -> Result<(String, Value), String> {
    let res = buscar(consulta)?;
    let mut ctx = String::new();
    let mut fontes = Vec::new();
    for (i, (t, u, s)) in res.iter().take(4).enumerate() {
        let corpo = if i < 2 { ler_pagina(u, 1800).filter(|x| x.len() > 200) } else { None }.unwrap_or_else(|| s.clone());
        ctx.push_str(&format!("[{}] {} ({})\n{}\n\n", i + 1, t, u, corpo));
        fontes.push(json!({"n": i + 1, "titulo": t, "url": u}));
    }
    Ok((ctx, json!(fontes)))
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test] fn urls() {
        assert!(url_publica("https://exemplo.com/a"));
        assert!(!url_publica("http://127.0.0.1:8080/"));
        assert!(!url_publica("http://localhost/"));
        assert!(!url_publica("http://192.168.0.1/"));
        assert!(!url_publica("http://10.0.0.5/x"));
        assert!(!url_publica("file:///etc/passwd"));
        assert!(!url_publica("http://user@127.0.0.1/"));
        assert!(!url_publica("http://[::1]/"));
    }
    #[test] fn resultados() {
        let h = r#"<a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexemplo.com%2Fx&rut=1">Titulo <b>um</b></a><a class="result__snippet" href="x">Resumo aqui</a>"#;
        let r = analisar_resultados(h);
        assert_eq!(r.len(), 1); assert_eq!(r[0].1, "https://exemplo.com/x"); assert_eq!(r[0].0, "Titulo um"); assert_eq!(r[0].2, "Resumo aqui");
    }
}

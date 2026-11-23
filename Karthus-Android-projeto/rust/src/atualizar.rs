//! Atualizações .karthus (aceita também .hirelod antigos): um zip com manifest.json (nome, versão, notas, arquivos com sha256).
//! Só caminhos permitidos são aceitos; tudo que for trocado vai para dados/backups antes.
use crate::estado::Estado;
use crate::util::*;
use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;

pub struct Pacote { pub nome: String, pub versao: String, pub arquivos: Vec<(String, Vec<u8>)>, pub criado: u64 }

pub fn caminho_permitido(r: &str) -> bool {
    if r.is_empty() || r.len() > 200 || r.contains("..") || r.contains('\\') || r.starts_with('/') || r.contains(':') || r.contains("//") { return false; }
    if ["Karthus.exe", "karthus-ia.exe", "VERSION.txt"].contains(&r) { return true; }
    if r.starts_with("LEIA-ME") && !r.contains('/') { return true; }
    if let Some(n) = r.strip_prefix("ui/") { return !n.is_empty() && !n.contains('/') && [".html", ".js", ".css", ".svg", ".png", ".ico"].iter().any(|e| n.ends_with(e)); }
    if r.starts_with("instaladores/") || r.starts_with("motores/") { return r.ends_with(".txt") || r.ends_with(".md") || r.ends_with(".json"); }
    false
}

pub fn inspecionar(est: &Estado, bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > 400_000_000 { return Err("O arquivo é grande demais para uma atualização.".into()); }
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|_| "Esse arquivo não é uma atualização .karthus válida.".to_string())?;
    let mut man = String::new();
    z.by_name("manifest.json").map_err(|_| "A atualização não tem manifest.json.".to_string())?.take(2_000_000).read_to_string(&mut man).map_err(|e| e.to_string())?;
    let m: Value = serde_json::from_str(&man).map_err(|_| "manifest.json inválido.".to_string())?;
    let lista = m["files"].as_array().ok_or("manifest.json sem lista de arquivos.")?;
    if lista.is_empty() || lista.len() > 200 { return Err("Lista de arquivos inválida.".into()); }
    let mut arqs = Vec::new();
    for f in lista {
        let p = f["path"].as_str().unwrap_or("");
        let h = f["sha256"].as_str().unwrap_or("").to_lowercase();
        if !caminho_permitido(p) { return Err(format!("A atualização tenta mexer em '{}', que não é permitido.", p)); }
        let mut d = Vec::new();
        z.by_name(p).map_err(|_| format!("O arquivo '{}' está no manifesto mas não no pacote.", p))?.take(300_000_000).read_to_end(&mut d).map_err(|e| e.to_string())?;
        if sha256_hex(&d) != h { return Err(format!("O arquivo '{}' está corrompido (a verificação sha256 não bate).", p)); }
        arqs.push((p.to_string(), d));
    }
    let token = novo_id("p");
    let nome = m["name"].as_str().unwrap_or("Atualização").chars().take(80).collect::<String>();
    let versao = m["version"].as_str().unwrap_or("?").chars().take(20).collect::<String>();
    let r = json!({"token": token, "nome": nome, "versao": versao, "notas": m["notes"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.chars().take(300).collect::<String>())).take(30).collect::<Vec<_>>()).unwrap_or_default(), "arquivos": arqs.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>()});
    let mut g = est.pacotes.lock().map_err(|_| "erro interno")?;
    g.retain(|_, v| agora_s() - v.criado < 900);
    g.insert(token, Pacote { nome, versao, arquivos: arqs, criado: agora_s() });
    Ok(r)
}

fn e_exe(p: &str) -> bool { p.ends_with(".exe") || p.ends_with(".dll") }

pub fn aplicar(est: &Estado, token: &str) -> Result<Value, String> {
    let pk = est.pacotes.lock().map_err(|_| "erro interno")?.remove(token).ok_or("Esse pacote expirou. Escolha o arquivo de novo.")?;
    let bk = est.dados.join("backups").join(format!("{}-{}", agora_s(), id_seguro(&pk.versao)));
    let mut reiniciar = false;
    for (p, d) in &pk.arquivos {
        let dst = est.base.join(p);
        if dst.is_file() { let b = bk.join(p); if let Some(pa) = b.parent() { let _ = std::fs::create_dir_all(pa); } let _ = std::fs::copy(&dst, &b); }
        if e_exe(p) {
            // o Windows deixa renomear um .exe em uso: o novo entra no lugar e o antigo some na próxima abertura
            let velho = dst.with_file_name(format!("{}.old", dst.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
            let _ = std::fs::remove_file(&velho);
            if dst.is_file() { std::fs::rename(&dst, &velho).map_err(|e| format!("Não consegui trocar {}: {}", p, e))?; }
            if let Err(e) = gravar_atomico(&dst, d) { let _ = std::fs::rename(&velho, &dst); return Err(format!("Não consegui gravar {}: {}", p, e)); }
            reiniciar = true;
        } else {
            gravar_atomico(&dst, d).map_err(|e| format!("Não consegui gravar {}: {}", p, e))?;
        }
    }
    log(&est.base, &format!("atualização {} aplicada ({} arquivos)", pk.versao, pk.arquivos.len()));
    if reiniciar { est.reiniciar.store(true, std::sync::atomic::Ordering::SeqCst); }
    Ok(json!({"ok": true, "reiniciar": reiniciar, "versao": pk.versao}))
}

pub fn limpar_velhos(base: &Path) {
    if let Ok(rd) = std::fs::read_dir(base) { for e in rd.flatten() { let n = e.file_name().to_string_lossy().to_string(); if n.ends_with(".exe.old") || n.ends_with(".dll.old") { let _ = std::fs::remove_file(e.path()); } } }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test] fn caminhos() {
        for ok in ["Karthus.exe", "karthus-ia.exe", "VERSION.txt", "ui/app.js", "ui/app.css", "LEIA-ME.md"] { assert!(caminho_permitido(ok), "{}", ok); }
        for no in ["../x", "ui/../x.js", "C:/x", "/etc/passwd", "ui/sub/a.js", "evil.exe", "ui/a.exe", "dados/usuarios/x.json", "ui/", "Hirelod.exe", "WebView2Loader.dll", "Karthus.exe/../a"] { assert!(!caminho_permitido(no), "{}", no); }
    }
}

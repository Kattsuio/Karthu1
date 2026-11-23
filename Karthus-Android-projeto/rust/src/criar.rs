//! Criações em segundo plano: imagem, vídeo, arquivos de texto/código e downloads pedidos no chat.
//! Cada pedido vira uma tarefa na aba "Criações"; o resultado aparece na galeria de Criações do usuário.
use crate::carga;
use crate::downloads;
use crate::estado::Estado;
use crate::util::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

/// Registra um arquivo pronto na galeria do usuário.
pub fn registrar(est: &Estado, uid: &str, tipo: &str, arq: &str, titulo: &str, extra: Value) {
    let tam = std::fs::metadata(est.saidas().join(arq)).map(|m| m.len()).unwrap_or(0);
    let mut c = json!({"arq": arq, "tipo": tipo, "titulo": titulo, "quando": agora_ms(), "tamanho": tam});
    if let Some(o) = extra.as_object() { for (k, v) in o { c[k] = v.clone(); } }
    est.repo.add_criacao(uid, c);
}

/// Roda uma tarefa de criação em segundo plano (uma por vez, para não travar o computador).
fn tarefa(est: &Arc<Estado>, uid: &str, item: &str, titulo: &str, tipo: &str, trabalho: impl FnOnce(&Arc<Estado>, &str) -> Result<(String, String, String), String> + Send + 'static) -> String {
    let job = downloads::novo_cat(est, item, titulo, tipo, "fila", 0, "criacao");
    let (e, j, u) = (est.clone(), job.clone(), uid.to_string());
    std::thread::spawn(move || {
        downloads::mexer(&e, &j, |d| d.msg = "Na fila…".into());
        let _g = e.fila_criar.lock().unwrap_or_else(|x| x.into_inner());
        downloads::mexer(&e, &j, |d| { d.estado = "ativo".into(); d.msg = "Começando…".into(); d.desde_ms = agora_ms(); });
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| trabalho(&e, &j))).unwrap_or_else(|_| Err("Erro interno ao criar.".into()));
        log(&e.base, &format!("criar {} {}: {:?}", tipo_s(&j, &e), titulo_curto(&u), r.as_ref().map(|x| x.0.clone())));
        match r {
            Ok((arq, tipo, resumo)) => { downloads::mexer(&e, &j, |d| d.arquivo = arq.clone()); let _ = tipo; downloads::concluir(&e, &j, Ok(resumo)); }
            Err(er) => downloads::concluir(&e, &j, Err(er)),
        }
    });
    job
}
fn tipo_s(j: &str, e: &Estado) -> String { downloads::lista(e).into_iter().find(|d| d.id == j).map(|d| d.tipo).unwrap_or_default() }
fn titulo_curto(u: &str) -> String { u.chars().take(8).collect() }

// ------------------------------------------------------------------------------------------ imagem
/// Lê o progresso do sd.exe ("| 3/20 - 1.5s/it") e repassa. O sd escreve com \r, então lemos byte a byte.
fn ler_progresso(r: impl Read, cb: &(dyn Fn(u32, u32) + Sync)) {
    let mut linha = Vec::new();
    for b in r.bytes().flatten() {
        if b == b'\r' || b == b'\n' {
            let s = String::from_utf8_lossy(&linha).to_string(); linha.clear();
            if let Some(p) = s.find('|') { if let Some((a, t)) = s[p..].split_whitespace().find_map(|w| { let (a, t) = w.split_once('/')?; Some((a.parse::<u32>().ok()?, t.parse::<u32>().ok()?)) }) { if t > 0 && a <= t { cb(a, t); } } }
        } else { linha.push(b); }
    }
}

pub fn rodar_sd(exe: &Path, modelo: &Path, prompt: &str, saida: &Path, w: u32, h: u32, passos: u32, semente: Option<i64>, extra: &[String], prog: &(dyn Fn(u32, u32) + Sync)) -> Result<(), String> {
    let mut c = Command::new(exe);
    c.args(extra);
    let turbo = modelo.file_name().map(|n| n.to_string_lossy().to_lowercase().contains("turbo")).unwrap_or(false);
    let passos = if turbo { passos.min(6) } else { passos };
    if turbo { c.args(["--cfg-scale", "1.0"]); }
    c.arg("-m").arg(modelo).arg("-p").arg(prompt).arg("-o").arg(saida).arg("-W").arg(w.to_string()).arg("-H").arg(h.to_string()).arg("--steps").arg(passos.to_string())
        .arg("-t").arg(crate::ia::threads_ideais().to_string());
    if let Some(s) = semente { c.arg("-s").arg(s.to_string()); }
    if let Some(d) = exe.parent() { c.current_dir(d); }
    c.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    sem_janela(&mut c);
    let mut f = c.spawn().map_err(|e| format!("Não consegui rodar o gerador de imagens: {}", e))?;
    let (so, se) = (f.stdout.take().unwrap(), f.stderr.take().unwrap());
    let ultimo = std::sync::Mutex::new(String::new());
    std::thread::scope(|s| {
        s.spawn(|| ler_progresso(so, prog));
        s.spawn(|| { let mut t = String::new(); for l in BufReader::new(se).split(b'\n').flatten() { let l = String::from_utf8_lossy(&l).to_string(); if l.contains('|') && l.contains('/') { if let Some(p) = l.find('|') { if let Some((a, t2)) = l[p..].split_whitespace().find_map(|w| { let (a, t) = w.split_once('/')?; Some((a.parse::<u32>().ok()?, t.parse::<u32>().ok()?)) }) { if t2 > 0 && a <= t2 { prog(a, t2); } } } } else if !l.trim().is_empty() { t = l; } } if let Ok(mut u) = ultimo.lock() { *u = t; } });
    });
    let st = f.wait().map_err(|e| e.to_string())?;
    if !st.success() || !saida.is_file() { return Err(format!("O gerador de imagens falhou{}", ultimo.lock().map(|u| if u.is_empty() { String::new() } else { format!(": {}", u.chars().take(200).collect::<String>()) }).unwrap_or_default())); }
    Ok(())
}

pub fn imagem(est: &Arc<Estado>, uid: &str, prompt: &str, w: u32, h: u32, passos: u32) -> Result<String, String> {
    let (exe, mut modelo) = carga::motor_imagem_pub(est).ok_or("O gerador de imagens não está instalado. Abra Criações > Motores ou o ícone vermelho.")?;
    let prompt: String = prompt.trim().chars().take(600).collect();
    if prompt.is_empty() { return Err("Descreva a imagem.".into()); }
    let perfil = crate::recursos::perfil(est);
    let extra: Vec<String> = perfil.as_ref().map(|p| p.extra.clone()).unwrap_or_default();
    if let Some(m) = perfil.as_ref().and_then(|p| p.modelo.clone()) { modelo = m; }
    let (w, h) = (w.clamp(256, 1024) / 64 * 64, h.clamp(256, 1024) / 64 * 64);
    let passos = passos.clamp(1, 60);
    let uid2 = uid.to_string(); let p2 = prompt.clone();
    Ok(tarefa(est, uid, &format!("imagem:{}", novo_id("i")), &format!("Imagem: {}", prompt.chars().take(40).collect::<String>()), "imagem", move |e, j| {
        let nome = format!("img-{}.png", novo_id(""));
        let saida = e.saidas().join(&nome);
        downloads::pct(e, j, 2.0, "Gerando a imagem…");
        let (e2, j2) = (e.clone(), j.to_string());
        rodar_sd(&exe, &modelo, &p2, &saida, w, h, passos, None, &extra, &move |a, t| downloads::pct(&e2, &j2, 3.0 + a as f64 * 95.0 / t as f64, &format!("Passo {} de {}", a, t)))?;
        registrar(e, &uid2, "imagem", &nome, &p2.chars().take(80).collect::<String>(), json!({"prompt": p2}));
        Ok((nome, "imagem".into(), "Imagem pronta".into()))
    }))
}

// ------------------------------------------------------------------------------------------ vídeo
fn rodar_ff(ff: &Path, args: &[String], cwd: Option<&Path>) -> Result<(), String> {
    let mut c = Command::new(ff);
    c.args(["-y", "-hide_banner", "-loglevel", "error", "-nostdin"]).args(args);
    if let Some(d) = cwd { c.current_dir(d); }
    c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    sem_janela(&mut c);
    let o = c.output().map_err(|e| format!("Não consegui rodar o ffmpeg: {}", e))?;
    if o.status.success() { Ok(()) } else { Err(format!("ffmpeg: {}", String::from_utf8_lossy(&o.stderr).lines().last().unwrap_or("falhou").chars().take(220).collect::<String>())) }
}

const CENAS: [&str; 8] = ["wide establishing shot", "close-up detail", "low angle dramatic lighting", "from above, aerial view", "soft sunrise light", "cinematic side view", "macro shot, shallow depth of field", "silhouette at dusk"];

pub fn video(est: &Arc<Estado>, uid: &str, prompt: &str, cenas: u32, seg: u32, w: u32, h: u32) -> Result<String, String> {
    let (sd, mut modelo) = carga::motor_imagem_pub(est).ok_or("O gerador de imagens não está instalado (o vídeo usa ele).")?;
    let ff = carga::ffmpeg_exe(est).ok_or("O ffmpeg não está instalado. Abra o ícone vermelho > Instalar (vídeo).")?;
    let prompt: String = prompt.trim().chars().take(400).collect();
    if prompt.is_empty() { return Err("Descreva o vídeo.".into()); }
    let perfil = crate::recursos::perfil(est);
    let mut extra: Vec<String> = Vec::new();
    { let mut it = perfil.as_ref().map(|p| p.extra.clone()).unwrap_or_default().into_iter(); while let Some(x) = it.next() { if x == "--upscale-model" { it.next(); } else { extra.push(x); } } }
    if let Some(m) = perfil.as_ref().and_then(|p| p.modelo.clone()) { modelo = m; }
    let (cenas, seg) = (cenas.clamp(1, 8), seg.clamp(1, 8));
    let (w, h) = (w.clamp(256, 1024) / 64 * 64, h.clamp(256, 1024) / 64 * 64);
    let uid2 = uid.to_string(); let p2 = prompt.clone();
    Ok(tarefa(est, uid, &format!("video:{}", novo_id("v")), &format!("Vídeo: {}", prompt.chars().take(40).collect::<String>()), "video", move |e, j| {
        let work = e.dados.join("tmp").join(novo_id("vid"));
        std::fs::create_dir_all(&work).map_err(|x| x.to_string())?;
        let r = (|| -> Result<String, String> {
            let mut clipes = Vec::new();
            for i in 0..cenas {
                let png = work.join(format!("c{}.png", i));
                let ptxt = format!("{}, {}", p2, CENAS[i as usize % CENAS.len()]);
                let base = i as f64 * 85.0 / cenas as f64;
                let (e2, j2) = (e.clone(), j.to_string());
                rodar_sd(&sd, &modelo, &ptxt, &png, w, h, 18, Some(1000 + i as i64 * 77), &extra, &move |a, t| downloads::pct(&e2, &j2, 2.0 + base + a as f64 * (85.0 / cenas as f64) / t as f64, &format!("Cena {} de {}: passo {} de {}", i + 1, cenas, a, t)))?;
                downloads::pct(e, j, 2.0 + (i + 1) as f64 * 85.0 / cenas as f64, &format!("Animando a cena {}…", i + 1));
                let clip = work.join(format!("c{}.mp4", i));
                let frames = seg * 25;
                let z = if i % 2 == 0 { "min(zoom+0.0012,1.25)".to_string() } else { "if(eq(on,1),1.25,max(zoom-0.0012,1.0))".to_string() };
                rodar_ff(&ff, &["-loop".into(), "1".into(), "-i".into(), png.to_string_lossy().to_string(), "-vf".into(), format!("zoompan=z='{}':d={}:s={}x{}:fps=25,format=yuv420p", z, frames, w, h), "-t".into(), seg.to_string(), "-c:v".into(), "libx264".into(), "-preset".into(), "veryfast".into(), "-crf".into(), "23".into(), clip.to_string_lossy().to_string()], None)?;
                clipes.push(clip);
            }
            downloads::pct(e, j, 92.0, "Juntando as cenas…");
            let lista = work.join("lista.txt");
            std::fs::write(&lista, clipes.iter().map(|c| format!("file '{}'\n", c.file_name().unwrap().to_string_lossy())).collect::<String>()).map_err(|x| x.to_string())?;
            let nome = format!("video-{}.mp4", novo_id(""));
            rodar_ff(&ff, &["-f".into(), "concat".into(), "-safe".into(), "0".into(), "-i".into(), "lista.txt".into(), "-c".into(), "copy".into(), "-movflags".into(), "+faststart".into(), e.saidas().join(&nome).to_string_lossy().to_string()], Some(&work))?;
            Ok(nome)
        })();
        let _ = std::fs::remove_dir_all(&work);
        let nome = r?;
        registrar(e, &uid2, "video", &nome, &p2.chars().take(80).collect::<String>(), json!({"prompt": p2}));
        Ok((nome, "video".into(), "Vídeo pronto".into()))
    }))
}

// ------------------------------------------------------------------------------------------ arquivos (texto, código, planilha CSV...)
const EXT_OK: [&str; 22] = ["txt", "md", "json", "csv", "tsv", "html", "css", "js", "ts", "py", "rs", "go", "java", "c", "cpp", "cs", "xml", "yml", "yaml", "toml", "sql", "svg"];

/// Separa "NOME: arquivo.ext" e o conteúdo (tira a cerca ```).
pub fn separar_arquivo(resp: &str, padrao: &str) -> (String, String) {
    let mut nome = String::new(); let mut linhas: Vec<&str> = resp.lines().collect();
    if let Some(p) = linhas.iter().position(|l| l.trim().to_uppercase().starts_with("NOME:")) { nome = linhas[p].trim()[5..].trim().trim_matches(|c| c == '`' || c == '"' || c == '*').to_string(); linhas.remove(p); }
    let txt = linhas.join("\n");
    let corpo = if let Some(a) = txt.find("```") { let r = &txt[a + 3..]; let r = r.split_once('\n').map(|x| x.1).unwrap_or(r); r.rsplit_once("```").map(|x| x.0).unwrap_or(r).to_string() } else { txt };
    let nome = nome_arquivo_seguro(&nome);
    let ext_ok = nome.rsplit_once('.').map(|(_, e)| EXT_OK.contains(&e.to_lowercase().as_str())).unwrap_or(false);
    (if ext_ok && nome != "arquivo" { nome } else { padrao.to_string() }, corpo.trim_end().to_string() + "\n")
}

pub fn arquivo(est: &Arc<Estado>, uid: &str, pedido: &str, modelo: &str) -> Result<String, String> {
    let pedido: String = pedido.trim().chars().take(1500).collect();
    if pedido.is_empty() { return Err("Descreva o arquivo.".into()); }
    if modelo.is_empty() { return Err("Não há modelo de IA instalado para escrever o arquivo.".into()); }
    let (uid2, p2, m2) = (uid.to_string(), pedido.clone(), modelo.to_string());
    Ok(tarefa(est, uid, &format!("arquivo:{}", novo_id("a")), &format!("Arquivo: {}", pedido.chars().take(40).collect::<String>()), "arquivo", move |e, j| {
        downloads::pct(e, j, 10.0, "A IA está escrevendo o arquivo…");
        let sis = "Você cria arquivos. Responda SOMENTE com a primeira linha 'NOME: nome-do-arquivo.ext' (extensões permitidas: txt, md, json, csv, html, css, js, py, rs, sql, xml, yaml) e depois o conteúdo completo do arquivo dentro de um bloco de código. Nada além disso.";
        let c = json!({"model": m2, "stream": false, "think": false, "keep_alive": 7200, "options": {"num_ctx": 4096, "num_predict": 1800, "temperature": 0.3}, "messages": [{"role": "system", "content": sis}, {"role": "user", "content": p2}]});
        let resp = e.ollama.chat_json(&c)?;
        let (nome0, corpo) = separar_arquivo(&resp, "arquivo.txt");
        if corpo.trim().is_empty() { return Err("A IA não devolveu conteúdo.".into()); }
        let nome = format!("{}-{}", novo_id("f"), nome0);
        std::fs::write(e.saidas().join(&nome), corpo.as_bytes()).map_err(|x| x.to_string())?;
        registrar(e, &uid2, "arquivo", &nome, &nome0, json!({"prompt": p2}));
        Ok((nome, "arquivo".into(), format!("{} criado", nome0)))
    }))
}

// ------------------------------------------------------------------------------------------ download pedido no chat
pub fn baixar_arquivo(est: &Arc<Estado>, uid: &str, url: &str) -> Result<String, String> {
    if !crate::web::url_publica(url) || !url.to_lowercase().starts_with("https://") { return Err("Só baixo endereços https públicos (nunca do próprio computador ou da sua rede).".into()); }
    let nome0 = nome_arquivo_seguro(url.split(['?', '#']).next().unwrap_or("").rsplit('/').next().unwrap_or("download"));
    let nome0 = if nome0 == "arquivo" || !nome0.contains('.') { "download.bin".to_string() } else { nome0 };
    let bloqueado = [".exe", ".msi", ".bat", ".cmd", ".ps1", ".vbs", ".scr", ".com", ".lnk", ".js", ".jar", ".dll"];
    if bloqueado.iter().any(|e| nome0.to_lowercase().ends_with(e)) { return Err("Por segurança, o Karthus não baixa programas executáveis ou scripts sozinho.".into()); }
    let (uid2, url2) = (uid.to_string(), url.to_string());
    let job = downloads::novo_cat(est, &format!("url:{}", url), &format!("Baixando {}", nome0), "arquivo-web", "fila", 0, "criacao");
    let (e, j) = (est.clone(), job.clone());
    std::thread::spawn(move || {
        let _g = e.fila.lock().unwrap_or_else(|x| x.into_inner());
        downloads::mexer(&e, &j, |d| { d.estado = "ativo".into(); d.desde_ms = agora_ms(); });
        let nome = format!("{}-{}", novo_id("d"), nome0);
        let r = crate::hf::baixar_url(&e, &j, &url2, &e.saidas().join(&nome), 0).map(|_| nome.clone());
        match r { Ok(n) => { registrar(&e, &uid2, "download", &n, &nome0, json!({"url": url2})); downloads::mexer(&e, &j, |d| d.arquivo = n.clone()); downloads::concluir(&e, &j, Ok(format!("{} baixado", nome0))); }
                  Err(er) => downloads::concluir(&e, &j, Err(er)) }
    });
    Ok(job)
}

// ------------------------------------------------------------------------------------------ entender pedidos do chat
/// O texto pede para criar algo? Devolve (tipo, assunto).
pub fn intencao(t: &str) -> Option<(&'static str, String)> {
    let l = t.trim().to_lowercase();
    if l.len() > 600 || l.contains('\n') && l.len() > 400 { return None; }
    let verbos = ["crie", "criar", "gere", "gerar", "faça", "faca", "fazer", "desenhe", "me faça", "monte", "produza"];
    let tem_verbo = |l: &str| verbos.iter().any(|v| l.split(|c: char| !c.is_alphanumeric()).any(|w| w == *v));
    let corta = |l: &str, chaves: &[&str]| -> String {
        for k in chaves { if let Some(p) = l.find(k) { let r = l[p + k.len()..].trim_start_matches(|c: char| c == ' ' || c == ':' || c == ',').trim(); let r = r.strip_prefix("de ").or_else(|| r.strip_prefix("com ")).or_else(|| r.strip_prefix("sobre ")).or_else(|| r.strip_prefix("do ")).or_else(|| r.strip_prefix("da ")).unwrap_or(r); return r.trim().to_string(); } }
        String::new()
    };
    if (l.starts_with("baixe ") || l.starts_with("baixar ") || l.starts_with("faça o download") || l.starts_with("faca o download") || l.starts_with("download ")) && l.contains("https://") {
        let u = t.split_whitespace().find(|w| w.to_lowercase().starts_with("https://"))?.trim_end_matches(|c: char| ".,;)".contains(c)).to_string();
        return Some(("download", u));
    }
    if !tem_verbo(&l) { return None; }
    if ["explique", "como ", "o que é", "o que e", "por que", "ensine", "qual a diferença"].iter().any(|k| l.starts_with(k)) { return None; }
    if ["um vídeo", "um video", "vídeo de", "video de", "uma animação", "uma animacao"].iter().any(|k| l.contains(k)) { let s = corta(&l, &["vídeo", "video", "animação", "animacao"]); if !s.is_empty() { return Some(("video", s)); } }
    if ["uma imagem", "uma foto", "um desenho", "uma ilustração", "uma ilustracao", "imagem de", "um logo", "um wallpaper"].iter().any(|k| l.contains(k)) { let s = corta(&l, &["imagem", "foto", "desenho", "ilustração", "ilustracao", "logo", "wallpaper"]); if !s.is_empty() { return Some(("imagem", s)); } }
    if ["um arquivo", "uma planilha", "um script", "um documento", "um csv", "um json", "uma página html", "uma pagina html"].iter().any(|k| l.contains(k)) { return Some(("arquivo", t.trim().to_string())); }
    None
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test] fn intencoes() {
        assert_eq!(intencao("crie uma imagem de um gato astronauta"), Some(("imagem", "um gato astronauta".to_string())));
        assert_eq!(intencao("Gere um vídeo de ondas no mar").unwrap().0, "video");
        assert_eq!(intencao("crie um arquivo csv com 3 clientes").unwrap().0, "arquivo");
        assert_eq!(intencao("baixe https://exemplo.com/a.pdf").unwrap(), ("download", "https://exemplo.com/a.pdf".to_string()));
        assert!(intencao("oi, tudo bem?").is_none());
        assert!(intencao("explique como criar uma imagem docker").is_none());
    }
    #[test] fn separa() {
        let (n, c) = separar_arquivo("NOME: dados.csv\n```csv\na,b\n1,2\n```", "x.txt"); assert_eq!(n, "dados.csv"); assert_eq!(c, "a,b\n1,2\n");
        let (n, _) = separar_arquivo("NOME: evil.exe\n```\nx\n```", "x.txt"); assert_eq!(n, "x.txt");
    }
}

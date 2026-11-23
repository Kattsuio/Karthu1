//! Cliente do Hugging Face (catálogo de modelos .gguf). Só fala com o endereço configurado (padrão huggingface.co).
use crate::downloads;
use crate::estado::Estado;
use serde_json::Value;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

pub fn base() -> String { std::env::var("HIRELOD_HF").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "https://huggingface.co".into()) }

fn agente(s: u64) -> ureq::Agent { ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(10)).timeout_read(Duration::from_secs(s)).user_agent("Karthus/6.1").build() }

pub fn repo_valido(r: &str) -> bool {
    let p: Vec<&str> = r.split('/').collect();
    p.len() == 2 && p.iter().all(|x| !x.is_empty() && x.len() < 100 && x.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)) && !x.starts_with('.'))
}

#[derive(Clone, Debug)]
pub struct Arquivo { pub nome: String, pub tamanho: u64 }

/// Escolhe o arquivo .gguf certo do repositório (prefere Q4_K_M: bom equilíbrio entre qualidade e tamanho).
pub fn escolher(siblings: &[(String, u64)]) -> Option<Arquivo> {
    let mut c: Vec<&(String, u64)> = siblings.iter().filter(|(n, _)| {
        let l = n.to_lowercase();
        l.ends_with(".gguf") && !l.contains("mmproj") && !l.contains("imatrix") && !l.contains("-of-0") && !n.contains('/')
    }).collect();
    c.sort_by_key(|(n, _)| n.clone());
    for pref in ["q4_k_m", "q4_k_s", "q4_0", "q5_k_m", "iq4_xs", "q4_k_l", "q5_k_s", "q6_k", "q8_0"] {
        if let Some((n, t)) = c.iter().find(|(n, _)| n.to_lowercase().contains(pref)) { return Some(Arquivo { nome: n.clone(), tamanho: *t }); }
    }
    c.into_iter().min_by_key(|(_, t)| *t).map(|(n, t)| Arquivo { nome: n.clone(), tamanho: *t })
}

pub fn resolver(repo: &str) -> Result<Arquivo, String> {
    if !repo_valido(repo) { return Err("Nome de repositório inválido.".into()); }
    let url = format!("{}/api/models/{}?blobs=true", base(), repo);
    let j: Value = match agente(30).get(&url).call() {
        Ok(r) => r.into_json().map_err(|e| format!("Resposta estranha do Hugging Face: {}", e))?,
        Err(ureq::Error::Status(404, _)) => return Err(format!("O repositório {} não existe no Hugging Face (ou mudou de nome).", repo)),
        Err(ureq::Error::Status(c, _)) => return Err(format!("O Hugging Face respondeu {} ao consultar {}.", c, repo)),
        Err(e) => return Err(format!("Sem conexão com o Hugging Face: {}", e)),
    };
    let sib: Vec<(String, u64)> = j["siblings"].as_array().map(|a| a.iter().filter_map(|s| Some((s["rfilename"].as_str()?.to_string(), s["size"].as_u64().unwrap_or(0)))).collect()).unwrap_or_default();
    escolher(&sib).ok_or_else(|| format!("O repositório {} não tem um arquivo .gguf utilizável.", repo))
}

/// Procura modelos populares no catálogo online (para atualizar a lista).
pub fn buscar(consulta: &str, limite: u32) -> Result<Vec<Value>, String> {
    let url = format!("{}/api/models?search={}&filter=gguf&sort=downloads&direction=-1&limit={}", base(), consulta.replace(' ', "+"), limite.min(60));
    match agente(30).get(&url).call() {
        Ok(r) => r.into_json::<Vec<Value>>().map_err(|e| format!("Resposta estranha do Hugging Face: {}", e)),
        Err(ureq::Error::Status(c, _)) => Err(format!("O Hugging Face respondeu {}.", c)),
        Err(e) => Err(format!("Sem conexão com o Hugging Face: {}", e)),
    }
}

/// Baixa o arquivo com retomada (se cair a internet, continua de onde parou) e mostra o progresso na aba Downloads.
pub fn baixar(est: &Estado, job: &str, repo: &str, arquivo: &str, destino: &Path, total_dica: u64) -> Result<(), String> {
    if !repo_valido(repo) || arquivo.contains("..") || arquivo.starts_with('/') || arquivo.contains('\\') || !(arquivo.to_lowercase().ends_with(".gguf") || arquivo.to_lowercase().ends_with(".bin")) { return Err("Arquivo inválido.".into()); }
    let url = format!("{}/{}/resolve/main/{}", base(), repo, arquivo);
    baixar_url(est, job, &url, destino, total_dica)
}

/// Baixa qualquer endereço https (ou o falso de teste) com retomada e progresso.
pub fn baixar_url(est: &Estado, job: &str, url: &str, destino: &Path, total_dica: u64) -> Result<(), String> {
    if let Some(p) = destino.parent() { std::fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    let parte = destino.with_file_name(format!("{}.parte", destino.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
    let mut tentativas = 0;
    loop {
        let ja = std::fs::metadata(&parte).map(|m| m.len()).unwrap_or(0);
        let mut rq = agente(60).get(url);
        if ja > 0 { rq = rq.set("Range", &format!("bytes={}-", ja)); }
        let r = match rq.call() {
            Ok(r) => r,
            Err(ureq::Error::Status(416, _)) => { let _ = std::fs::remove_file(&parte); tentativas += 1; if tentativas > 3 { return Err("O servidor recusou a retomada.".into()); } continue; }
            Err(ureq::Error::Status(c, _)) => return Err(format!("O Hugging Face respondeu {} ao baixar o arquivo.", c)),
            Err(e) => { tentativas += 1; if tentativas > 4 { return Err(format!("Sem conexão: {}", e)); } downloads::mexer(est, job, |d| d.msg = format!("Sem conexão, tentando de novo ({}/4)…", tentativas)); std::thread::sleep(Duration::from_secs(3)); continue; }
        };
        let retomou = r.status() == 206;
        let resto: u64 = r.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
        let base_off = if retomou { ja } else { 0 };
        let total = if resto > 0 { base_off + resto } else { total_dica };
        let mut f = std::fs::OpenOptions::new().create(true).write(true).append(retomou).truncate(!retomou).open(&parte).map_err(|e| format!("Não consegui gravar: {}", e))?;
        let mut feito = base_off;
        let mut leitor = r.into_reader();
        let mut buf = vec![0u8; 1 << 20];
        let mut ultimo = Instant::now();
        let mut caiu = false;
        loop {
            match leitor.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    f.write_all(&buf[..n]).map_err(|e| format!("Falha ao gravar (falta espaço no disco?): {}", e))?;
                    feito += n as u64;
                    if ultimo.elapsed() > Duration::from_millis(250) { ultimo = Instant::now(); downloads::bytes(est, job, feito, total, &format!("Baixando… {}", crate::util::tamanho_humano(feito))); }
                }
                Err(_) => { caiu = true; break; }
            }
        }
        let _ = f.flush(); drop(f);
        let tam = std::fs::metadata(&parte).map(|m| m.len()).unwrap_or(0);
        if total > 0 && tam < total {
            if !caiu && tam == 0 { return Err("O arquivo veio vazio.".into()); }
            tentativas += 1; if tentativas > 4 { return Err("A conexão caiu várias vezes. Tente de novo mais tarde (o que já baixou fica guardado).".into()); }
            downloads::mexer(est, job, |d| d.msg = format!("Conexão caiu, retomando ({}/4)…", tentativas));
            std::thread::sleep(Duration::from_secs(2));
            continue;
        }
        let _ = std::fs::remove_file(destino);
        return std::fs::rename(&parte, destino).map_err(|e| format!("Não consegui finalizar o arquivo: {}", e));
    }
}

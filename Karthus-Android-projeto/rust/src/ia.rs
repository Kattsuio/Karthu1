//! Motor de IA nativo (hirelod-ia, baseado em llama.cpp). Fala a mesma API do Ollama, então o resto do Karthus é simples.
//! O motor roda como processo filho, só em 127.0.0.1, e fecha sozinho se o Karthus fechar (--pai).
use crate::estado::Estado;
use crate::util::*;
use serde_json::{json, Value};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct Cliente { url: Mutex<String>, filho: Mutex<Option<Child>>, externo: bool, llama: Option<crate::llama::Llama> }

fn porta_livre() -> u16 { std::net::TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).unwrap_or(11555) }

impl Cliente {
    pub fn novo() -> Cliente {
        let ext = std::env::var("HIRELOD_IA_URL").or_else(|_| std::env::var("OLLAMA_HOST")).ok().filter(|s| !s.is_empty());
        let llama = crate::llama::exe_configurado().map(crate::llama::Llama::novo);
        match ext {
            Some(u) => Cliente { url: Mutex::new(if u.starts_with("http") { u } else { format!("http://{}", u) }), filho: Mutex::new(None), externo: true, llama: None },
            None => Cliente { url: Mutex::new(String::new()), filho: Mutex::new(None), externo: false, llama },
        }
    }
    pub fn url(&self) -> String { self.url.lock().map(|u| u.clone()).unwrap_or_default() }
    pub fn externo(&self) -> bool { self.externo || self.llama.is_some() }

    fn ag(&self, leitura: u64) -> ureq::Agent { ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(4)).timeout_read(Duration::from_secs(leitura)).build() }

    pub fn versao(&self) -> Result<String, String> {
        if let Some(l) = &self.llama { return l.pronto().map(|_| "llama.cpp".to_string()); }
        let u = self.url(); if u.is_empty() { return Err("O motor de IA ainda não foi iniciado.".into()); }
        let j: Value = self.ag(5).get(&format!("{}/api/version", u)).call().map_err(|e| format!("O motor de IA não respondeu: {}", e))?.into_json().map_err(|e| e.to_string())?;
        Ok(j["version"].as_str().unwrap_or("?").to_string())
    }

    /// Resposta em fluxo (NDJSON linha a linha).
    pub fn chat_fluxo(&self, corpo: &Value) -> Result<Box<dyn Read + Send>, String> {
        if let Some(l) = &self.llama { return l.chat_fluxo(corpo); }
        let u = self.url(); if u.is_empty() { return Err("O motor de IA não está rodando.".into()); }
        match self.ag(900).post(&format!("{}/api/chat", u)).send_json(corpo.clone()) {
            Ok(r) => Ok(Box::new(r.into_reader())),
            Err(ureq::Error::Status(c, r)) => { let t = r.into_string().unwrap_or_default(); Err(erro_amigavel(c, &t)) }
            Err(e) => Err(format!("Não consegui falar com o motor de IA: {}", e)),
        }
    }
    /// Resposta completa de uma vez (modo fluxo e etapas internas).
    pub fn chat_json(&self, corpo: &Value) -> Result<String, String> {
        if let Some(l) = &self.llama { return l.chat_json(corpo); }
        let u = self.url(); if u.is_empty() { return Err("O motor de IA não está rodando.".into()); }
        match self.ag(1800).post(&format!("{}/api/chat", u)).send_json(corpo.clone()) {
            Ok(r) => { let j: Value = r.into_json().map_err(|e| e.to_string())?; if let Some(e) = j["error"].as_str() { return Err(e.to_string()); } Ok(j["message"]["content"].as_str().unwrap_or("").to_string()) }
            Err(ureq::Error::Status(c, r)) => { let t = r.into_string().unwrap_or_default(); Err(erro_amigavel(c, &t)) }
            Err(e) => Err(format!("Falha ao falar com o motor de IA: {}", e)),
        }
    }
    /// Carrega o modelo na memória e deixa lá (aquecimento): a primeira resposta deixa de esperar o carregamento.
    pub fn aquecer(&self, modelo: &str) -> Result<(), String> {
        if let Some(l) = &self.llama { return l.aquecer(modelo); }
        let c = json!({"model": modelo, "stream": false, "keep_alive": 7200, "think": false, "options": {"num_predict": 1, "num_ctx": 1024},
                       "messages": [{"role": "user", "content": "oi"}]});
        self.chat_json(&c).map(|_| ())
    }

    /// Liga o motor (ou usa o externo). Espera até ficar pronto.
    pub fn iniciar(&self, est: &Estado) -> Result<(), String> {
        if let Some(l) = &self.llama { l.configurar(vec![est.base.join("modelos"), est.instaladores().join("modelos")], est.dados.join("logs")); return l.pronto(); }
        if self.externo { return self.versao().map(|_| ()); }
        if self.filho.lock().map(|mut f| f.as_mut().map(|c| c.try_wait().ok().flatten().is_none()).unwrap_or(false)).unwrap_or(false) && self.versao().is_ok() { return Ok(()); }
        let exe = exe(est).ok_or("Não achei o karthus-ia.exe ao lado do Karthus.exe. Reinstale o Karthus.")?;
        let porta = porta_livre();
        let _ = crate::modelos_locais::atualizar_catalogo(est);
        let logp = est.dados.join("logs");
        let _ = std::fs::create_dir_all(&logp);
        let log = std::fs::OpenOptions::new().create(true).append(true).open(logp.join("ia.log")).map_err(|e| e.to_string())?;
        let log2 = log.try_clone().map_err(|e| e.to_string())?;
        let mut c = Command::new(&exe);
        c.arg("--porta").arg(porta.to_string()).arg("--catalogo").arg(est.dados.join("catalogo-modelos.json")).arg("--threads").arg(threads_ideais().to_string())
            .arg("--pai").arg(std::process::id().to_string()).stdin(Stdio::null()).stdout(Stdio::from(log)).stderr(Stdio::from(log2));
        sem_janela(&mut c);
        let filho = c.spawn().map_err(|e| format!("Não consegui iniciar o motor de IA: {}", e))?;
        if let Ok(mut f) = self.filho.lock() { if let Some(mut velho) = f.take() { let _ = velho.kill(); } *f = Some(filho); }
        if let Ok(mut u) = self.url.lock() { *u = format!("http://127.0.0.1:{}", porta); }
        let t0 = Instant::now();
        loop {
            if self.versao().is_ok() { return Ok(()); }
            if let Ok(mut f) = self.filho.lock() { if let Some(c) = f.as_mut() { if let Ok(Some(s)) = c.try_wait() { return Err(format!("O motor de IA fechou logo ao iniciar ({}). Veja dados/logs/ia.log.", s)); } } }
            if t0.elapsed() > Duration::from_secs(25) { return Err("O motor de IA demorou demais para iniciar.".into()); }
            std::thread::sleep(Duration::from_millis(120));
        }
    }
    pub fn parar(&self) { if let Some(l) = &self.llama { l.parar(); } if let Ok(mut f) = self.filho.lock() { if let Some(mut c) = f.take() { let _ = c.kill(); } } }
}

fn erro_amigavel(c: u16, t: &str) -> String {
    let m = serde_json::from_str::<Value>(t).ok().and_then(|j| j["error"].as_str().map(|s| s.to_string())).unwrap_or_else(|| t.chars().take(200).collect());
    if m.contains("not found") || m.contains("não encontrado") { return format!("Esse modelo não foi encontrado. Escolha outro no menu do modelo ou baixe no Catálogo de IAs. ({})", m); }
    format!("O motor de IA respondeu {}: {}", c, m)
}

/// Threads de geração: núcleos físicos (a metade dos lógicos) rende mais que usar todos em CPUs com hyper-threading.
pub fn threads_ideais() -> usize { let n = nucleos(); (if n >= 8 { n / 2 } else { n.saturating_sub(1) }).clamp(2, 16) }

/// Caminho do karthus-ia.exe (no Windows ele vem embutido no Karthus.exe e é extraído ao lado dele).
pub fn exe(est: &Estado) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("HIRELOD_IA_EXE") { if !p.is_empty() { return Some(PathBuf::from(p)); } }
    garantir_embutido(est);
    let dirs = [est.dados.join("bin"), est.base.clone()];
    dirs.iter().flat_map(|d| ["karthus-ia.exe", "karthus-ia", "hirelod-ia.exe", "hirelod-ia"].iter().map(move |n| d.join(n))).find(|p| p.is_file())
}

#[cfg(windows)]
static EMBUTIDO: &[u8] = include_bytes!("../embutido/karthus-ia.exe");
#[cfg(windows)]
fn garantir_embutido(est: &Estado) {
    let dst = est.dados.join("bin").join("karthus-ia.exe");
    let igual = std::fs::metadata(&dst).map(|m| m.len() == EMBUTIDO.len() as u64).unwrap_or(false);
    if !igual { if let Some(p) = dst.parent() { let _ = std::fs::create_dir_all(p); } let _ = gravar_atomico(&dst, EMBUTIDO); }
}
#[cfg(not(windows))]
fn garantir_embutido(_est: &Estado) {}

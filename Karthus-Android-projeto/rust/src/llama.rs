//! Motor de IA para Android (e testes): usa o `llama-server` do llama.cpp e traduz o formato do Karthus (estilo Ollama).
use crate::util::*;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

struct Rodando { filho: Option<Child>, porta: u16, modelo: String, ctx: u32 }
pub struct Llama { exe: PathBuf, r: Mutex<Rodando>, dirs: OnceLock<(Vec<PathBuf>, PathBuf)> }

fn ag(leitura: u64) -> ureq::Agent { ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(4)).timeout_read(Duration::from_secs(leitura)).build() }
fn porta_livre() -> u16 { std::net::TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).unwrap_or(11556) }

pub fn exe_configurado() -> Option<PathBuf> {
    std::env::var("HIRELOD_LLAMA_SERVER").ok().filter(|s| !s.is_empty()).map(PathBuf::from)
}

impl Llama {
    pub fn novo(exe: PathBuf) -> Llama { Llama { exe, r: Mutex::new(Rodando { filho: None, porta: 0, modelo: String::new(), ctx: 0 }), dirs: OnceLock::new() } }
    pub fn configurar(&self, modelos: Vec<PathBuf>, logs: PathBuf) { let _ = self.dirs.set((modelos, logs)); }
    pub fn pronto(&self) -> Result<(), String> { if self.exe.is_file() { Ok(()) } else { Err(format!("Não achei o motor llama.cpp ({}).", self.exe.display())) } }
    pub fn parar(&self) { if let Ok(mut r) = self.r.lock() { if let Some(mut c) = r.filho.take() { let _ = c.kill(); let _ = c.wait(); } r.modelo.clear(); } }

    fn achar_modelo(&self, nome: &str) -> Result<PathBuf, String> {
        let (dirs, _) = self.dirs.get().ok_or("Motor ainda não configurado.")?;
        let mut todos: Vec<(String, PathBuf)> = Vec::new();
        for d in dirs { if let Ok(rd) = std::fs::read_dir(d) { for e in rd.flatten() { let p = e.path(); let n = e.file_name().to_string_lossy().to_string(); if n.to_lowercase().ends_with(".gguf") { todos.push((n[..n.len() - 5].to_string(), p)); } } } }
        let alvo = nome.trim().trim_end_matches(":latest").to_lowercase();
        if let Some((_, p)) = todos.iter().find(|(n, _)| n.to_lowercase() == alvo) { return Ok(p.clone()); }
        if let Some((_, p)) = todos.iter().find(|(n, _)| { let n = n.to_lowercase(); !alvo.is_empty() && (n.contains(&alvo) || alvo.contains(&n)) }) { return Ok(p.clone()); }
        if alvo.is_empty() && todos.len() == 1 { return Ok(todos[0].1.clone()); }
        Err(format!("Esse modelo não foi encontrado ({}). Baixe um no Catálogo de IAs.", nome))
    }

    /// Deixa o llama-server rodando com o modelo certo (reinicia só se mudar o modelo ou se precisar de mais contexto).
    fn garantir(&self, modelo: &str, ctx: u32) -> Result<String, String> {
        self.pronto()?;
        let arq = self.achar_modelo(modelo)?;
        let chave = arq.to_string_lossy().to_string();
        let ctx = ctx.clamp(1024, 8192);
        let mut r = self.r.lock().map_err(|_| "erro interno".to_string())?;
        let vivo = r.filho.as_mut().map(|c| c.try_wait().ok().flatten().is_none()).unwrap_or(false);
        if vivo && r.modelo == chave && r.ctx >= ctx { return Ok(format!("http://127.0.0.1:{}", r.porta)); }
        if let Some(mut c) = r.filho.take() { let _ = c.kill(); let _ = c.wait(); }
        let porta = porta_livre();
        let (_, logs) = self.dirs.get().ok_or("Motor ainda não configurado.")?;
        let _ = std::fs::create_dir_all(logs);
        let log = std::fs::OpenOptions::new().create(true).append(true).open(logs.join("ia.log")).map_err(|e| e.to_string())?;
        let log2 = log.try_clone().map_err(|e| e.to_string())?;
        let mut c = Command::new(&self.exe);
        c.arg("-m").arg(&arq).arg("--host").arg("127.0.0.1").arg("--port").arg(porta.to_string()).arg("-c").arg(ctx.to_string())
            .arg("-t").arg(crate::ia::threads_ideais().to_string()).arg("-np").arg("1").stdin(Stdio::null()).stdout(Stdio::from(log)).stderr(Stdio::from(log2));
        sem_janela(&mut c);
        // se o Karthus morrer, o llama.cpp morre junto (senão ele ficaria ocupando a memória do celular)
        #[cfg(any(target_os = "linux", target_os = "android"))]
        unsafe { use std::os::unix::process::CommandExt; c.pre_exec(|| { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong); Ok(()) }); }
        let mut filho = c.spawn().map_err(|e| format!("Não consegui iniciar o llama.cpp: {}", e))?;
        let t0 = Instant::now();
        loop {
            if ag(3).get(&format!("http://127.0.0.1:{}/health", porta)).call().is_ok() { break; }
            if let Ok(Some(s)) = filho.try_wait() { return Err(format!("O llama.cpp fechou ao carregar o modelo ({}). Veja dados/logs/ia.log (pode faltar memória).", s)); }
            if t0.elapsed() > Duration::from_secs(300) { let _ = filho.kill(); return Err("O modelo demorou demais para carregar.".into()); }
            std::thread::sleep(Duration::from_millis(250));
        }
        r.filho = Some(filho); r.porta = porta; r.modelo = chave; r.ctx = ctx;
        Ok(format!("http://127.0.0.1:{}", porta))
    }

    pub fn aquecer(&self, modelo: &str) -> Result<(), String> { self.garantir(modelo, 2048).map(|_| ()) }

    fn corpo_openai(corpo: &Value, stream: bool) -> Value {
        let o = &corpo["options"];
        let msgs: Vec<Value> = corpo["messages"].as_array().cloned().unwrap_or_default().iter().map(|m| json!({"role": m["role"], "content": m["content"]})).collect();
        let think = match &corpo["think"] { Value::Bool(b) => *b, Value::String(s) => s != "false" && !s.is_empty(), _ => false };
        let mut b = json!({"model": "karthus", "messages": msgs, "stream": stream, "chat_template_kwargs": {"enable_thinking": think}});
        if let Some(t) = o["temperature"].as_f64() { b["temperature"] = json!(t); }
        if let Some(t) = o["top_p"].as_f64() { b["top_p"] = json!(t); }
        if let Some(n) = o["num_predict"].as_i64() { if n > 0 { b["max_tokens"] = json!(n); } }
        b
    }
    fn ctx_pedido(corpo: &Value) -> u32 { corpo["options"]["num_ctx"].as_u64().unwrap_or(2048) as u32 }
    fn modelo_pedido(corpo: &Value) -> String { corpo["model"].as_str().unwrap_or("").to_string() }

    pub fn chat_fluxo(&self, corpo: &Value) -> Result<Box<dyn Read + Send>, String> {
        let u = self.garantir(&Self::modelo_pedido(corpo), Self::ctx_pedido(corpo))?;
        match ag(900).post(&format!("{}/v1/chat/completions", u)).send_json(Self::corpo_openai(corpo, true)) {
            Ok(r) => Ok(Box::new(Sse { r: BufReader::new(Box::new(r.into_reader())), saida: VecDeque::new(), fim: false })),
            Err(ureq::Error::Status(c, r)) => Err(format!("O llama.cpp respondeu {}: {}", c, r.into_string().unwrap_or_default().chars().take(200).collect::<String>())),
            Err(e) => Err(format!("Não consegui falar com o llama.cpp: {}", e)),
        }
    }
    pub fn chat_json(&self, corpo: &Value) -> Result<String, String> {
        let u = self.garantir(&Self::modelo_pedido(corpo), Self::ctx_pedido(corpo))?;
        match ag(1800).post(&format!("{}/v1/chat/completions", u)).send_json(Self::corpo_openai(corpo, false)) {
            Ok(r) => { let j: Value = r.into_json().map_err(|e| e.to_string())?; Ok(j["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()) }
            Err(ureq::Error::Status(c, r)) => Err(format!("O llama.cpp respondeu {}: {}", c, r.into_string().unwrap_or_default().chars().take(200).collect::<String>())),
            Err(e) => Err(format!("Falha ao falar com o llama.cpp: {}", e)),
        }
    }
}

/// Converte o fluxo SSE do llama-server em linhas NDJSON no formato que o Karthus já entende.
struct Sse { r: BufReader<Box<dyn Read + Send>>, saida: VecDeque<u8>, fim: bool }
impl Sse {
    fn linha(&mut self, l: &str) {
        let Some(d) = l.strip_prefix("data:") else { return };
        let d = d.trim();
        if d == "[DONE]" { self.terminar(); return; }
        let Ok(j) = serde_json::from_str::<Value>(d) else { return };
        let dl = &j["choices"][0]["delta"];
        let (c, t) = (dl["content"].as_str().unwrap_or(""), dl["reasoning_content"].as_str().unwrap_or(""));
        if !c.is_empty() || !t.is_empty() {
            let mut m = json!({"role": "assistant", "content": c}); if !t.is_empty() { m["thinking"] = json!(t); }
            self.saida.extend(json!({"message": m, "done": false}).to_string().bytes()); self.saida.push_back(b'\n');
        }
        if !j["choices"][0]["finish_reason"].is_null() { self.terminar(); }
    }
    fn terminar(&mut self) { if !self.fim { self.fim = true; self.saida.extend(json!({"message": {"role": "assistant", "content": ""}, "done": true, "done_reason": "stop"}).to_string().bytes()); self.saida.push_back(b'\n'); } }
}
impl Read for Sse {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if !self.saida.is_empty() { let n = buf.len().min(self.saida.len()); for b in buf.iter_mut().take(n) { *b = self.saida.pop_front().unwrap(); } return Ok(n); }
            if self.fim { return Ok(0); }
            let mut l = String::new();
            match self.r.read_line(&mut l) { Ok(0) => { self.terminar(); } Ok(_) => self.linha(l.trim_end()), Err(e) => return Err(e) }
        }
    }
}

#[allow(dead_code)]
pub fn existe(p: &Path) -> bool { p.is_file() }

#[cfg(test)]
mod testes {
    use super::*;
    #[test] fn sse_para_ndjson() {
        let sse = "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"hm\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"Olá\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
        let mut s = Sse { r: BufReader::new(Box::new(std::io::Cursor::new(sse.as_bytes().to_vec()))), saida: VecDeque::new(), fim: false };
        let mut t = String::new(); s.read_to_string(&mut t).unwrap();
        let ls: Vec<Value> = t.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(ls.len(), 3);
        assert_eq!(ls[0]["message"]["thinking"], "hm"); assert_eq!(ls[1]["message"]["content"], "Olá"); assert_eq!(ls[2]["done"], true);
    }
    #[test] fn corpo() {
        let b = Llama::corpo_openai(&json!({"messages":[{"role":"user","content":"oi","images":["x"]}],"think":false,"options":{"num_predict":50,"temperature":0.3}}), true);
        assert_eq!(b["max_tokens"], 50); assert_eq!(b["chat_template_kwargs"]["enable_thinking"], false); assert!(b["messages"][0].get("images").is_none());
    }
}

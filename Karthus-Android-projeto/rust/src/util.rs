//! Funções de apoio: pastas, JSON, ids seguros, escrita atômica, memória, fluxo HTTP em pedaços.
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const VERSAO: &str = env!("CARGO_PKG_VERSION");

/// Pasta do Karthus: HIRELOD_HOME (testes) ou a pasta do executável.
pub fn base_dir() -> PathBuf {
    if let Ok(p) = std::env::var("HIRELOD_HOME") { if !p.is_empty() { return PathBuf::from(p); } }
    std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())).unwrap_or_else(|| PathBuf::from("."))
}

pub fn agora_s() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }
pub fn agora_ms() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0) }

static SEQ: AtomicU64 = AtomicU64::new(1);
/// Id curto único (tempo + contador + pid), só com letras e números.
pub fn novo_id(prefixo: &str) -> String {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{}{:x}{:x}{:x}", prefixo, agora_ms(), std::process::id() & 0xfff, n & 0xfff)
}

pub fn id_seguro(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').take(48).collect::<String>().to_lowercase()
}

/// Nome de arquivo simples (sem pastas, sem caracteres perigosos).
pub fn nome_arquivo_seguro(s: &str) -> String {
    let base = s.rsplit(['/', '\\']).next().unwrap_or("");
    let l: String = base.chars().map(|c| if c.is_control() || "<>:\"|?*".contains(c) { '_' } else { c }).collect();
    let l = l.trim().trim_start_matches('.').trim().to_string();
    let l: String = l.chars().take(120).collect();
    if l.is_empty() { "arquivo".into() } else { l }
}

pub fn ler_json<T: DeserializeOwned>(p: &Path) -> Option<T> {
    let b = std::fs::read(p).ok()?;
    serde_json::from_slice(&b).ok()
}

pub fn gravar_atomico(p: &Path, dados: &[u8]) -> std::io::Result<()> {
    if let Some(d) = p.parent() { std::fs::create_dir_all(d)?; }
    let tmp = p.with_file_name(format!("{}.tmp{}", p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), std::process::id()));
    { let mut f = std::fs::File::create(&tmp)?; f.write_all(dados)?; f.flush()?; }
    if std::fs::rename(&tmp, p).is_err() { let _ = std::fs::remove_file(p); std::fs::rename(&tmp, p)?; }
    Ok(())
}
pub fn gravar_json(p: &Path, v: &Value) -> std::io::Result<()> { gravar_atomico(p, serde_json::to_string_pretty(v).unwrap_or_default().as_bytes()) }

pub fn sha256_hex(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new(); h.update(b);
    h.finalize().iter().map(|x| format!("{:02x}", x)).collect()
}

pub fn tamanho_humano(b: u64) -> String {
    if b >= 1_000_000_000 { format!("{:.2} GB", b as f64 / 1e9) } else if b >= 1_000_000 { format!("{:.1} MB", b as f64 / 1e6) } else { format!("{} KB", b / 1000) }
}

pub fn log(base: &Path, msg: &str) {
    let d = base.join("dados").join("logs");
    let _ = std::fs::create_dir_all(&d);
    let p = d.join("karthus.log");
    if std::fs::metadata(&p).map(|m| m.len() > 2_000_000).unwrap_or(false) { let _ = std::fs::rename(&p, d.join("karthus.antigo.log")); }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) { let _ = writeln!(f, "[{}] {}", agora_s(), msg); }
}

/// Em Windows: não abre janela preta ao iniciar um programa de linha de comando.
pub fn sem_janela(c: &mut std::process::Command) {
    #[cfg(windows)]
    { use std::os::windows::process::CommandExt; c.creation_flags(0x08000000); }
    #[cfg(not(windows))]
    { let _ = c; }
}

/// (RAM total, RAM livre) em bytes.
pub fn memoria() -> (u64, u64) {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct Ms { len: u32, carga: u32, total: u64, livre: u64, pf_t: u64, pf_l: u64, v_t: u64, v_l: u64, ext: u64 }
        extern "system" { fn GlobalMemoryStatusEx(m: *mut Ms) -> i32; }
        let mut m = Ms { len: std::mem::size_of::<Ms>() as u32, carga: 0, total: 0, livre: 0, pf_t: 0, pf_l: 0, v_t: 0, v_l: 0, ext: 0 };
        if unsafe { GlobalMemoryStatusEx(&mut m) } != 0 { return (m.total, m.livre); }
        (0, 0)
    }
    #[cfg(not(windows))]
    {
        let t = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let pega = |k: &str| t.lines().find(|l| l.starts_with(k)).and_then(|l| l.split_whitespace().nth(1)).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0) * 1024;
        (pega("MemTotal:"), pega("MemAvailable:"))
    }
}
pub fn nucleos() -> usize { std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2) }

/// Envia uma resposta em pedaços (chunked) e dá flush a cada um: o tiny_http segura 1 KB antes de enviar.
pub fn enviar_fluxo(rq: tiny_http::Request, tipo: &str, mut leitor: Box<dyn std::io::Read + Send>) {
    let mut w = rq.into_writer();
    let cab = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nTransfer-Encoding: chunked\r\nCache-Control: no-store\r\nX-Accel-Buffering: no\r\n\r\n", tipo);
    if w.write_all(cab.as_bytes()).is_err() || w.flush().is_err() { return; }
    let mut buf = vec![0u8; 8192];
    loop {
        let n = match leitor.read(&mut buf) { Ok(0) | Err(_) => break, Ok(n) => n };
        if w.write_all(format!("{:x}\r\n", n).as_bytes()).is_err() || w.write_all(&buf[..n]).is_err() || w.write_all(b"\r\n").is_err() || w.flush().is_err() { return; }
    }
    let _ = w.write_all(b"0\r\n\r\n");
    let _ = w.flush();
}

/// Divide linhas de comando do tipo "a b 'c d'" — usado só em testes e utilitários simples.
#[allow(dead_code)]
pub fn dentro_de(base: &Path, p: &Path) -> bool {
    match (std::fs::canonicalize(base), std::fs::canonicalize(p)) { (Ok(b), Ok(p)) => p.starts_with(b), _ => false }
}

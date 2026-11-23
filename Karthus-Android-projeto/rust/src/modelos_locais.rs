//! Acha os modelos .gguf: pasta modelos/, instaladores/modelos/ e (opcional) o armazenamento do Ollama, sem precisar do Ollama.
use crate::estado::Estado;
use crate::util::*;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Arq { pub nome: String, pub caminho: PathBuf, pub tamanho: u64 }

fn gguf_valido(p: &std::path::Path) -> bool {
    use std::io::Read;
    let mut b = [0u8; 4];
    std::fs::File::open(p).and_then(|mut f| f.read_exact(&mut b)).map(|_| &b == b"GGUF").unwrap_or(false)
}

fn minimo() -> u64 { std::env::var("HIRELOD_GGUF_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(50_000_000) }

fn pasta_ollama() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OLLAMA_MODELS") { if !p.is_empty() { return Some(PathBuf::from(p)); } }
    let h = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok()?;
    Some(PathBuf::from(h).join(".ollama").join("models"))
}

pub fn escanear(est: &Estado) -> Vec<Arq> {
    let mut v: Vec<Arq> = Vec::new();
    for d in [est.base.join("modelos"), est.instaladores().join("modelos")] {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                let n = e.file_name().to_string_lossy().to_string();
                if !n.to_lowercase().ends_with(".gguf") { continue; }
                let tam = e.metadata().map(|m| m.len()).unwrap_or(0);
                if tam < minimo() || !gguf_valido(&p) { continue; }
                let nome = n[..n.len() - 5].to_string();
                if v.iter().any(|a| a.nome == nome) { continue; }
                v.push(Arq { nome, caminho: p, tamanho: tam });
            }
        }
    }
    // modelos que o usuário já tem no Ollama (ex.: gemma4): lemos o arquivo direto, o Ollama não precisa estar aberto
    if let Some(raiz) = pasta_ollama() {
        let man = raiz.join("manifests").join("registry.ollama.ai").join("library");
        if let Ok(rd) = std::fs::read_dir(&man) {
            for fam in rd.flatten() {
                let Ok(tags) = std::fs::read_dir(fam.path()) else { continue };
                for t in tags.flatten() {
                    let Some(m): Option<Value> = ler_json(&t.path()) else { continue };
                    let Some(dig) = m["layers"].as_array().and_then(|l| l.iter().find(|x| x["mediaType"].as_str() == Some("application/vnd.ollama.image.model"))).and_then(|x| x["digest"].as_str().map(|s| s.to_string())) else { continue };
                    let blob = raiz.join("blobs").join(dig.replace(':', "-"));
                    let tam = std::fs::metadata(&blob).map(|m| m.len()).unwrap_or(0);
                    if tam < minimo() || !gguf_valido(&blob) { continue; }
                    let nome = format!("{}:{}", fam.file_name().to_string_lossy(), t.file_name().to_string_lossy());
                    if v.iter().any(|a| a.nome == nome) { continue; }
                    v.push(Arq { nome, caminho: blob, tamanho: tam });
                }
            }
        }
    }
    v.sort_by(|a, b| a.nome.cmp(&b.nome));
    v
}

/// Escreve a lista que o motor lê (dados/catalogo-modelos.json).
pub fn atualizar_catalogo(est: &Estado) -> Result<usize, String> {
    let l = escanear(est);
    let j: Vec<Value> = l.iter().map(|a| json!({"nome": a.nome, "caminho": a.caminho.to_string_lossy(), "tamanho": a.tamanho, "template": "auto"})).collect();
    gravar_json(&est.dados.join("catalogo-modelos.json"), &Value::Array(j)).map_err(|e| e.to_string())?;
    Ok(l.len())
}

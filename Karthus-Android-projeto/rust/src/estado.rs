//! Estado compartilhado do programa.
use crate::carga::Carga;
use crate::downloads::Download;
use crate::ia::Cliente;
use crate::usuarios::Repo;
use crate::util::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

pub struct Estado {
    pub base: PathBuf,
    pub dados: PathBuf,
    pub porta: u16,
    pub ollama: Cliente,
    pub repo: Repo,
    pub downloads: Mutex<Vec<Download>>,
    pub carga: Mutex<Carga>,
    pub fila: Mutex<()>,
    pub fila_criar: Mutex<()>,
    pub voz: crate::voz::Voz,
    pub pacotes: Mutex<HashMap<String, crate::atualizar::Pacote>>,
    pub pular: AtomicBool,
    pub sair: AtomicBool,
    pub reiniciar: AtomicBool,
}

impl Estado {
    pub fn novo(base: PathBuf, porta: u16) -> Estado {
        let dados = base.join("dados");
        let _ = std::fs::create_dir_all(&dados);
        for d in ["modelos", "motores", "instaladores", "saidas"] { let _ = std::fs::create_dir_all(base.join(d)); }
        Estado {
            repo: Repo::novo(&dados), base, dados, porta, ollama: Cliente::novo(),
            downloads: Mutex::new(vec![]), carga: Mutex::new(Carga::nova()), fila: Mutex::new(()), fila_criar: Mutex::new(()),
            voz: crate::voz::Voz::novo(), pacotes: Mutex::new(HashMap::new()),
            pular: AtomicBool::new(false), sair: AtomicBool::new(false), reiniciar: AtomicBool::new(false),
        }
    }
    pub fn motores(&self) -> PathBuf { self.base.join("motores") }
    pub fn instaladores(&self) -> PathBuf { self.base.join("instaladores") }
    pub fn saidas(&self) -> PathBuf { self.base.join("saidas") }

    pub fn global(&self) -> Value { ler_json(&self.dados.join("global.json")).unwrap_or_else(|| json!({})) }
    pub fn gravar_global(&self, patch: &Value) -> Result<(), String> {
        let mut g = self.global();
        if let Some(o) = patch.as_object() { for (k, v) in o { if ["modelo_padrao", "idioma_voz"].contains(&k.as_str()) { g[k] = v.clone(); } } }
        gravar_json(&self.dados.join("global.json"), &g).map_err(|e| e.to_string())
    }
    /// Modelo padrão: o escolhido; senão qwen3 pequeno > gemma4 > qualquer um instalado.
    pub fn modelo_padrao(&self) -> String {
        let g = self.global();
        let lista = crate::modelos_locais::escanear(self);
        if let Some(m) = g["modelo_padrao"].as_str() { if lista.iter().any(|a| a.nome == m) { return m.to_string(); } }
        for pref in ["qwen3", "phi-4", "phi4", "smollm", "gemma4", "mistral"] { if let Some(a) = lista.iter().find(|a| a.nome.to_lowercase().contains(pref)) { return a.nome.clone(); } }
        lista.first().map(|a| a.nome.clone()).unwrap_or_default()
    }
}

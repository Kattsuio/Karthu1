//! Registro de tarefas em segundo plano: downloads/instalações (cat "download") e criações (cat "criacao":
//! imagem, vídeo, cortes, arquivos...). A interface consulta /api/downloads a cada segundo.
use crate::carga;
use crate::estado::Estado;
use crate::util::*;
use serde::Serialize;

#[derive(Clone, Serialize, Debug, Default)]
pub struct Download {
    pub id: String,
    pub item: String,
    pub titulo: String,
    pub tipo: String,
    pub cat: String,      // "download" | "criacao"
    pub estado: String,   // fila | ativo | ok | erro
    pub msg: String,
    pub pct: f64,
    pub feito: u64,
    pub total: u64,
    pub vel: f64,
    pub inicio: u64,
    pub fim: u64,
    pub desde_ms: u64,
    pub arquivo: String,  // criação pronta: nome do arquivo em saidas/
    pub eta_s: u64,       // estimativa de entrega (segundos), 0 = desconhecida
    #[serde(skip)] pub ult_ms: u64,
    #[serde(skip)] pub ult_feito: u64,
}

pub fn lista(est: &Estado) -> Vec<Download> { est.downloads.lock().map(|l| l.clone()).unwrap_or_default() }

pub fn ativo_do_item(est: &Estado, item: &str) -> Option<String> {
    est.downloads.lock().ok()?.iter().find(|d| d.item == item && (d.estado == "fila" || d.estado == "ativo")).map(|d| d.id.clone())
}

pub fn novo_cat(est: &Estado, item: &str, titulo: &str, tipo: &str, estado: &str, total: u64, cat: &str) -> String {
    let id = novo_id("t");
    let d = Download { id: id.clone(), item: item.into(), titulo: titulo.into(), tipo: tipo.into(), cat: cat.into(), estado: estado.into(), inicio: agora_s(), desde_ms: agora_ms(), total, ..Default::default() };
    if let Ok(mut l) = est.downloads.lock() {
        l.push(d);
        // mantém no máximo 60 registros (os mais antigos já concluídos saem primeiro)
        while l.len() > 60 { if let Some(i) = l.iter().position(|x| x.estado == "ok" || x.estado == "erro") { l.remove(i); } else { break; } }
    }
    id
}
pub fn novo(est: &Estado, item: &str, titulo: &str, tipo: &str, estado: &str, total: u64) -> String { novo_cat(est, item, titulo, tipo, estado, total, "download") }

pub fn mexer(est: &Estado, id: &str, f: impl FnOnce(&mut Download)) {
    if let Ok(mut l) = est.downloads.lock() { if let Some(d) = l.iter_mut().find(|d| d.id == id) { f(d); } }
}

/// Progresso por bytes (calcula % e velocidade) e repassa à barra do item do carregador, se houver.
pub fn bytes(est: &Estado, id: &str, feito: u64, total: u64, msg: &str) {
    let item = est.downloads.lock().ok().and_then(|l| l.iter().find(|d| d.id == id).map(|d| d.item.clone())).unwrap_or_default();
    let pct = if total > 0 { (feito as f64 * 100.0 / total as f64).clamp(0.0, 99.0) } else { 0.0 };
    if carga::instalavel(&item) { carga::mexer_item(est, &item, |it| { it.pct = pct.max(1.0); it.msg = msg.to_string(); }); }
    mexer(est, id, |d| {
        let agora = agora_ms();
        if d.ult_ms == 0 { d.ult_ms = agora; d.ult_feito = feito; }
        else if agora - d.ult_ms >= 700 {
            let v = (feito.saturating_sub(d.ult_feito)) as f64 * 1000.0 / (agora - d.ult_ms) as f64;
            d.vel = if d.vel == 0.0 { v } else { d.vel * 0.6 + v * 0.4 };
            d.ult_ms = agora; d.ult_feito = feito;
        }
        d.estado = "ativo".into(); d.feito = feito; d.total = total; d.pct = pct; d.msg = msg.to_string();
        d.eta_s = if d.vel > 1.0 && total > feito { ((total - feito) as f64 / d.vel) as u64 } else { 0 };
    });
}

/// Progresso por porcentagem (criações: etapas) com estimativa de entrega.
pub fn pct(est: &Estado, id: &str, pct: f64, msg: &str) {
    mexer(est, id, |d| {
        d.estado = "ativo".into(); d.pct = pct.clamp(0.0, 99.0); d.msg = msg.to_string();
        let dec = (agora_ms().saturating_sub(d.desde_ms)) as f64 / 1000.0;
        d.eta_s = if d.pct >= 3.0 && dec > 2.0 { (dec * (100.0 - d.pct) / d.pct) as u64 } else { 0 };
    });
}

pub fn concluir(est: &Estado, id: &str, r: Result<String, String>) {
    let item = est.downloads.lock().ok().and_then(|l| l.iter().find(|d| d.id == id).map(|d| d.item.clone())).unwrap_or_default();
    mexer(est, id, |d| {
        d.fim = agora_s(); d.vel = 0.0; d.eta_s = 0;
        match &r { Ok(m) => { d.estado = "ok".into(); d.pct = 100.0; d.msg = m.clone(); } Err(e) => { d.estado = "erro".into(); d.msg = e.clone(); } }
    });
    if carga::instalavel(&item) {
        match &r { Ok(_) => carga::rechecar(est, &item), Err(e) => carga::mexer_item(est, &item, |it| { it.estado = "erro".into(); it.msg = e.clone(); }) }
    }
}

pub fn limpar(est: &Estado) { if let Ok(mut l) = est.downloads.lock() { l.retain(|d| d.estado == "ativo" || d.estado == "fila"); } }

pub fn json(est: &Estado) -> serde_json::Value {
    let l = lista(est);
    let ativos = l.iter().any(|d| d.estado == "ativo" || d.estado == "fila");
    serde_json::json!({"itens": l, "ativos": ativos})
}

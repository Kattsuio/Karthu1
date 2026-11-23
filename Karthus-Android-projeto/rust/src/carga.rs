//! Verificação e instalação dos componentes (tela de carregamento e painel "Instalação").
use crate::downloads;
use crate::estado::Estado;
use crate::util::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Item { pub id: String, pub nome: String, pub estado: String, pub msg: String, pub dica: String, pub instalavel: bool, pub critico: bool, pub pct: f64 }
pub struct Carga { pub itens: Vec<Item>, pub rodando: bool, pub concluido: bool, pub atual: String, pub gen: u64 }

const ORDEM: [(&str, &str, bool); 9] = [
    ("dados", "Pastas e dados", true), ("interface", "Interface", true), ("motor_ia", "Motor de IA", true), ("modelo", "Modelo de IA", true), ("aquecer", "Carregar o modelo na memória", false),
    ("voz_ouvir", "Voz: ouvir (Whisper)", false), ("voz_falar", "Voz: falar", false), ("imagem", "Gerador de imagens", false), ("video", "Gerador de vídeos", false),
];

impl Carga {
    pub fn nova() -> Carga {
        Carga { itens: ORDEM.iter().filter(|(id, _, _)| !(so_chat() && ["voz_ouvir", "voz_falar", "imagem", "video"].contains(id))).map(|(id, n, c)| Item { id: id.to_string(), nome: n.to_string(), estado: "espera".into(), msg: String::new(), dica: String::new(), instalavel: false, critico: *c, pct: 0.0 }).collect(), rodando: false, concluido: false, atual: String::new(), gen: 0 }
    }
}

/// No Android (e com HIRELOD_SO_CHAT=1) só o chat é verificado: voz, imagem e vídeo dependem de programas do Windows.
pub fn so_chat() -> bool { cfg!(target_os = "android") || std::env::var("HIRELOD_SO_CHAT").is_ok() }

pub fn instalavel(item: &str) -> bool { ["voz_ouvir", "imagem", "video", "modelo"].contains(&item) }

pub fn mexer_item(est: &Estado, id: &str, f: impl FnOnce(&mut Item)) { if let Ok(mut c) = est.carga.lock() { if let Some(i) = c.itens.iter_mut().find(|i| i.id == id) { f(i); } } }

pub fn achar_arquivo(dir: &Path, prefixo: &str, sufixo: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).find(|p| p.is_file() && p.file_name().map(|n| { let n = n.to_string_lossy().to_lowercase(); n.starts_with(prefixo) && n.ends_with(sufixo) }).unwrap_or(false))
}
fn tem_arquivos(d: &Path) -> bool { std::fs::read_dir(d).map(|r| r.flatten().next().is_some()).unwrap_or(false) }

pub fn whisper_exe(est: &Estado) -> Option<PathBuf> {
    let d = est.motores().join("whisper");
    ["whisper-cli.exe", "main.exe", "whisper-cli", "whisper"].iter().map(|n| d.join(n)).find(|p| p.is_file())
}
pub fn whisper_modelo(est: &Estado) -> Option<PathBuf> { achar_arquivo(&est.motores().join("whisper"), "ggml-", ".bin") }
pub fn sd_exe(est: &Estado) -> Option<PathBuf> { let d = est.motores().join("imagem"); ["sd.exe", "sd"].iter().map(|n| d.join(n)).find(|p| p.is_file()) }
pub fn ffmpeg_exe(est: &Estado) -> Option<PathBuf> { let d = est.motores().join("video"); ["ffmpeg.exe", "ffmpeg"].iter().map(|n| d.join(n)).find(|p| p.is_file()).or_else(|| if std::env::var("HIRELOD_FFMPEG_SISTEMA").is_ok() { Some(PathBuf::from("ffmpeg")) } else { None }) }
pub fn modelo_imagem(est: &Estado) -> Option<PathBuf> {
    let d = est.motores().join("imagem");
    [d.join("modelos"), d.clone()].iter().find_map(|x| std::fs::read_dir(x).ok().and_then(|r| r.flatten().map(|e| e.path()).filter(|p| p.is_file() && { let n = p.to_string_lossy().to_lowercase(); (n.ends_with(".safetensors") || n.ends_with(".gguf") || n.ends_with(".ckpt")) && !n.contains("vae") && !n.contains("lora") }).max_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))))
}
pub fn motor_imagem_pub(est: &Estado) -> Option<(PathBuf, PathBuf)> { Some((sd_exe(est)?, modelo_imagem(est)?)) }

type R = (&'static str, String, String, bool); // estado, msg, dica, instalavel
fn ok(m: &str) -> R { ("ok", m.into(), String::new(), false) }
fn falta(m: &str, d: &str, inst: bool) -> R { ("falta", m.into(), d.into(), inst) }

fn checar(est: &Arc<Estado>, id: &str) -> R {
    match id {
        "dados" => if std::fs::create_dir_all(est.dados.join("tmp")).is_ok() { ok("Pastas prontas.") } else { ("erro", "Não consegui gravar na pasta do Karthus.".into(), "Mova o Karthus para uma pasta onde o senhor possa gravar (ex.: Documentos).".into(), false) },
        "interface" => ok("Interface carregada."),
        "motor_ia" => match est.ollama.iniciar(est) {
            Ok(_) => ok(&format!("Motor de IA nativo pronto{}.", if est.ollama.externo() { " (externo)" } else { "" })),
            Err(e) => ("erro", e, "Reinstale o Karthus: o karthus-ia.exe precisa ficar ao lado do Karthus.exe.".into(), false),
        },
        "modelo" => {
            let n = crate::modelos_locais::atualizar_catalogo(est).unwrap_or(0);
            if n > 0 { let l = crate::modelos_locais::escanear(est); ok(&format!("{} modelo(s): {}", n, l.iter().take(4).map(|a| a.nome.clone()).collect::<Vec<_>>().join(", "))) }
            else { falta("Nenhum modelo de IA encontrado.", "Clique em Instalar para baixar o Qwen3 1.7B (~1,1 GB), ou coloque um arquivo .gguf na pasta modelos. Outros no Catálogo de IAs.", true) }
        }
        "aquecer" => {
            let m = est.modelo_padrao();
            if m.is_empty() { return ("espera", "Aguardando um modelo.".into(), String::new(), false); }
            let est2 = est.clone(); let m2 = m.clone();
            let h = std::thread::spawn(move || est2.ollama.aquecer(&m2));
            let t0 = std::time::Instant::now();
            loop {
                if h.is_finished() { return match h.join() { Ok(Ok(_)) => ok(&format!("{} carregado e pronto.", m)), Ok(Err(e)) => ("erro", e, "O modelo não carregou. Veja se há memória livre ou escolha um modelo menor.".into(), false), Err(_) => ("erro", "Falha interna ao carregar.".into(), String::new(), false) }; }
                if est.pular.swap(false, Ordering::SeqCst) { return ok("Carregando em segundo plano."); }
                mexer_item(est, "aquecer", |i| i.msg = format!("Carregando {} na memória… {}s", m, t0.elapsed().as_secs()));
                std::thread::sleep(Duration::from_millis(250));
            }
        }
        "voz_ouvir" => {
            if whisper_exe(est).is_some() && whisper_modelo(est).is_some() { ok("Whisper pronto.") }
            else if whisper_exe(est).is_none() { falta("Falta o programa do Whisper (whisper-cli.exe).", "Clique em Instalar: baixa o whisper.cpp oficial (~5 MB) e o modelo ggml-base.bin (~150 MB), uma vez só.", true) }
            else { falta("Falta o modelo do Whisper (ggml-*.bin).", "Clique em Instalar para baixar o ggml-base.bin (~150 MB) ou coloque um ggml-*.bin na pasta motores/whisper.", true) }
        }
        "voz_falar" => {
            if std::env::var("HIRELOD_VOZ_FALSA").is_ok() { return ok("Voz de teste."); }
            let n = crate::voz::vozes().len();
            if n > 0 { ok(&format!("{} voz(es) do Windows.", n)) } else { falta("Nenhuma voz do Windows foi encontrada.", "Windows: Configurações > Hora e idioma > Fala > Adicionar vozes. (Opcional)", false) }
        }
        "imagem" => {
            if sd_exe(est).is_some() && modelo_imagem(est).is_some() { ok("Gerador de imagens pronto.") }
            else { falta("O gerador de imagens não está instalado (opcional).", "Clique em Instalar: baixa o sd.exe e o modelo SD-Turbo (~2 GB). Ou adicione seus arquivos em Criações > Motores.", true) }
        }
        "video" => {
            if ffmpeg_exe(est).is_some() { ok("ffmpeg pronto.") } else { falta("Falta o ffmpeg (opcional; vídeos e Cortes usam).", "Clique em Instalar para baixar o ffmpeg (~150 MB).", true) }
        }
        _ => ok(""),
    }
}

fn aplicar(est: &Estado, id: &str, r: R) { mexer_item(est, id, |i| { i.estado = r.0.into(); i.msg = r.1; i.dica = r.2; i.instalavel = r.3; i.pct = if r.0 == "ok" { 100.0 } else { 0.0 }; }); }

/// Verifica tudo (em segunda thread).
pub fn verificar(est: &Arc<Estado>) {
    {
        let mut c = match est.carga.lock() { Ok(c) => c, Err(e) => e.into_inner() };
        if c.rodando { return; }
        c.rodando = true; c.concluido = false; c.gen += 1;
        for i in c.itens.iter_mut() { if i.estado != "instalando" { i.estado = "espera".into(); i.msg.clear(); i.pct = 0.0; } }
    }
    let est = est.clone();
    std::thread::spawn(move || {
        for (id, nome, _) in ORDEM {
            if let Ok(c) = est.carga.lock() { if c.itens.iter().any(|i| i.id == id && i.estado == "instalando") { continue; } }
            if let Ok(mut c) = est.carga.lock() { c.atual = if id == "aquecer" { "carregando o modelo na memória".into() } else { nome.to_string() }; }
            mexer_item(&est, id, |i| i.estado = "verificando".into());
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| checar(&est, id))).unwrap_or(("erro", "Erro interno na verificação.".into(), String::new(), false));
            log(&est.base, &format!("carga {}: {} - {}", id, r.0, r.1));
            aplicar(&est, id, r);
        }
        if let Ok(mut c) = est.carga.lock() { c.rodando = false; c.concluido = true; c.atual.clear(); }
    });
}

/// Reverifica só um item (depois de instalar).
pub fn rechecar(est: &Estado, id: &str) {
    // precisa de Arc: o Estado vive em um Arc global
    if let Some(a) = crate::GLOBAL.get() { let a = a.clone(); let id = id.to_string(); std::thread::spawn(move || { let r = checar(&a, &id); aplicar(&a, &id, r); if id == "modelo" && r_ok(&a, "modelo") { let r2 = checar(&a, "aquecer"); aplicar(&a, "aquecer", r2); } }); }
    let _ = est;
}
fn r_ok(est: &Estado, id: &str) -> bool { est.carga.lock().map(|c| c.itens.iter().any(|i| i.id == id && i.estado == "ok")).unwrap_or(false) }

pub fn json(est: &Estado) -> Value {
    let c = match est.carga.lock() { Ok(c) => c, Err(e) => e.into_inner() };
    let total = c.itens.len() as f64;
    let feitos = c.itens.iter().filter(|i| !["espera", "verificando"].contains(&i.estado.as_str())).count() as f64;
    let pct = if c.concluido { 100.0 } else { feitos * 100.0 / total };
    let crit = c.itens.iter().filter(|i| i.critico && (i.estado == "falta" || i.estado == "erro")).count();
    json!({"rodando": c.rodando, "concluido": c.concluido, "atual": c.atual, "pct": pct, "criticos_com_problema": crit,
           "itens": c.itens.iter().map(|i| json!({"id": i.id, "nome": i.nome, "estado": i.estado, "msg": i.msg, "dica": i.dica, "instalavel": i.instalavel, "critico": i.critico, "pct": i.pct})).collect::<Vec<_>>()})
}
pub fn saude(est: &Estado) -> Value {
    let c = match est.carga.lock() { Ok(c) => c, Err(e) => e.into_inner() };
    json!({"problemas": c.itens.iter().filter(|i| i.estado == "falta" || i.estado == "erro").map(|i| json!({"id": i.id, "nome": i.nome, "msg": i.msg})).collect::<Vec<_>>()})
}

fn copiar_pasta_motor(est: &Estado, tipo: &str) -> Result<(), String> {
    let de = est.instaladores().join("motores").join(tipo);
    let para = est.motores().join(tipo);
    std::fs::create_dir_all(&para).map_err(|e| e.to_string())?;
    let mut n = 0;
    for e in std::fs::read_dir(&de).map_err(|e| e.to_string())?.flatten() {
        let p = e.path();
        if p.is_file() { let dst = para.join(e.file_name()); if !dst.is_file() { std::fs::copy(&p, &dst).map_err(|e| format!("Não consegui copiar {}: {}", p.display(), e))?; } n += 1; }
    }
    if n == 0 { Err("A pasta de instaladores está vazia.".into()) } else { Ok(()) }
}

/// Trabalho de instalação (roda dentro da tarefa em segundo plano).
pub fn instalar_agora(est: &Arc<Estado>, item: &str) -> Result<String, String> {
    let job = downloads::ativo_do_item(est, item).unwrap_or_default();
    let local = |t: &str| tem_arquivos(&est.instaladores().join("motores").join(t));
    match item {
        "voz_ouvir" => {
            if local("whisper") { copiar_pasta_motor(est, "whisper")?; }
            if whisper_exe(est).is_none() { crate::motores_web::instalar_whisper_exe(est, &job)?; }
            if whisper_modelo(est).is_none() { crate::hf::baixar(est, &job, "ggerganov/whisper.cpp", "ggml-base.bin", &est.motores().join("whisper").join("ggml-base.bin"), 147_951_465)?; }
            Ok("Whisper instalado.".into())
        }
        "imagem" => {
            if local("imagem") { copiar_pasta_motor(est, "imagem")?; }
            if sd_exe(est).is_none() { crate::motores_web::instalar_sd(est, &job)?; }
            if modelo_imagem(est).is_none() { crate::motores_web::instalar_modelo_imagem(est, &job)?; }
            Ok("Gerador de imagens instalado.".into())
        }
        "video" => {
            if local("video") { copiar_pasta_motor(est, "video")?; }
            if ffmpeg_exe(est).is_none() { crate::motores_web::instalar_ffmpeg(est, &job)?; }
            Ok("ffmpeg instalado.".into())
        }
        "modelo" => crate::catalogo::baixar_agora(est, &job, "qwen3-1.7b"),
        _ => Err("Esse item não é instalável.".into()),
    }
}

/// Inicia a instalação em segundo plano (um clique só: repetir o clique devolve a mesma tarefa).
pub fn instalar(est: &Arc<Estado>, item: &str) -> Result<String, String> {
    if !instalavel(item) { return Err("Esse item não é instalável.".into()); }
    if let Some(j) = downloads::ativo_do_item(est, item) { return Ok(j); }
    let titulo = ORDEM.iter().find(|(i, _, _)| *i == item).map(|(_, n, _)| format!("Instalando {}", n)).unwrap_or_default();
    let job = downloads::novo(est, item, &titulo, "instalar", "fila", 0);
    mexer_item(est, item, |i| { i.estado = "instalando".into(); i.msg = "Na fila…".into(); i.pct = 1.0; });
    let (e, j, it) = (est.clone(), job.clone(), item.to_string());
    std::thread::spawn(move || {
        let _g = e.fila.lock().unwrap_or_else(|x| x.into_inner());
        downloads::mexer(&e, &j, |d| { d.estado = "ativo".into(); d.msg = "Começando…".into(); d.desde_ms = agora_ms(); });
        mexer_item(&e, &it, |i| i.msg = "Começando…".into());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| instalar_agora(&e, &it))).unwrap_or_else(|_| Err("Erro interno na instalação.".into()));
        log(&e.base, &format!("instalar {}: {:?}", it, r));
        downloads::concluir(&e, &j, r);
    });
    Ok(job)
}

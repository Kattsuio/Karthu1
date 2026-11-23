//! Catálogo de modelos de linguagem: lista embutida + pesquisa online. A IA do Karthus lê esta lista para recomendar.
use crate::downloads;
use crate::estado::Estado;
use crate::hf;
use crate::modelos_locais;
use crate::util::*;
use serde_json::{json, Value};
use std::sync::Arc;

struct Base { id: &'static str, nome: &'static str, familia: &'static str, repo: &'static str, params_b: f64, raciocina: bool, uso: &'static str, desc: &'static str }

const EMBUTIDOS: &[Base] = &[
    Base { id: "qwen3-0.6b", nome: "Qwen3 0.6B", familia: "Qwen", repo: "unsloth/Qwen3-0.6B-GGUF", params_b: 0.6, raciocina: true, uso: "leve", desc: "Minúsculo e rápido; bom para PCs fracos e para testar." },
    Base { id: "qwen3-1.7b", nome: "Qwen3 1.7B", familia: "Qwen", repo: "unsloth/Qwen3-1.7B-GGUF", params_b: 1.7, raciocina: true, uso: "leve", desc: "Leve, raciocina passo a passo. Responde bem em PCs com 8 GB." },
    Base { id: "qwen3-4b", nome: "Qwen3 4B", familia: "Qwen", repo: "unsloth/Qwen3-4B-GGUF", params_b: 4.0, raciocina: true, uso: "geral", desc: "Equilíbrio entre velocidade e qualidade; raciocina e escreve bem em português." },
    Base { id: "qwen3-8b", nome: "Qwen3 8B", familia: "Qwen", repo: "unsloth/Qwen3-8B-GGUF", params_b: 8.2, raciocina: true, uso: "geral", desc: "Mais inteligente; pede 16 GB de RAM e é mais lento em CPU." },
    Base { id: "qwen3-14b", nome: "Qwen3 14B", familia: "Qwen", repo: "unsloth/Qwen3-14B-GGUF", params_b: 14.8, raciocina: true, uso: "geral", desc: "Muito capaz; só para PCs com 24 GB ou mais." },
    Base { id: "phi4-mini", nome: "Phi-4 mini", familia: "Phi", repo: "unsloth/Phi-4-mini-instruct-GGUF", params_b: 3.8, raciocina: false, uso: "geral", desc: "Da Microsoft; forte em lógica e matemática para o tamanho." },
    Base { id: "phi4-mini-reasoning", nome: "Phi-4 mini reasoning", familia: "Phi", repo: "unsloth/Phi-4-mini-reasoning-GGUF", params_b: 3.8, raciocina: true, uso: "raciocínio", desc: "Versão que pensa longamente antes de responder (bom para contas e lógica)." },
    Base { id: "smollm3-3b", nome: "SmolLM3 3B", familia: "SmolLM", repo: "ggml-org/SmolLM3-3B-GGUF", params_b: 3.1, raciocina: true, uso: "leve", desc: "Pequeno, multilíngue, com modo de raciocínio opcional." },
    Base { id: "deepseek-r1-1.5b", nome: "DeepSeek-R1 Distill Qwen 1.5B", familia: "DeepSeek", repo: "bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF", params_b: 1.8, raciocina: true, uso: "raciocínio", desc: "Raciocínio no estilo R1 em tamanho mínimo." },
    Base { id: "deepseek-r1-7b", nome: "DeepSeek-R1 Distill Qwen 7B", familia: "DeepSeek", repo: "bartowski/DeepSeek-R1-Distill-Qwen-7B-GGUF", params_b: 7.6, raciocina: true, uso: "raciocínio", desc: "Raciocínio forte (pensa em voz alta); demora mais para começar a responder." },
    Base { id: "mistral-7b", nome: "Mistral 7B Instruct", familia: "Mistral", repo: "bartowski/Mistral-7B-Instruct-v0.3-GGUF", params_b: 7.2, raciocina: false, uso: "geral", desc: "Clássico e estável, bom para textos e conversa." },
    Base { id: "qwen25-coder-7b", nome: "Qwen2.5 Coder 7B", familia: "Qwen", repo: "bartowski/Qwen2.5-Coder-7B-Instruct-GGUF", params_b: 7.6, raciocina: false, uso: "código", desc: "Especializado em programar." },
];

fn estimar(params_b: f64) -> u64 { (params_b * 0.62e9) as u64 }

fn familia_de(id: &str) -> String {
    let l = id.to_lowercase();
    for (k, v) in [("qwen", "Qwen"), ("phi", "Phi"), ("smol", "SmolLM"), ("deepseek", "DeepSeek"), ("mistral", "Mistral"), ("gemma", "Gemma"), ("llama", "Llama"), ("granite", "Granite"), ("olmo", "OLMo"), ("glm", "GLM"), ("yi", "Yi")] {
        if l.contains(k) { return v.to_string(); }
    }
    "Outra".into()
}

fn slug(s: &str) -> String { s.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' { c } else { '-' }).collect::<String>().trim_matches('-').to_string() }

fn arq_cache(est: &Estado) -> std::path::PathBuf { est.dados.join("catalogo-ia.json") }

fn descobertos(est: &Estado) -> (Vec<Value>, u64) {
    let j = ler_json(&arq_cache(est)).unwrap_or(json!({}));
    (j["itens"].as_array().cloned().unwrap_or_default(), j["quando"].as_u64().unwrap_or(0))
}

/// A lista que a interface (e a própria IA) enxerga.
pub fn lista(est: &Estado) -> Value {
    let (ram, _) = memoria();
    let ram_gb = ram as f64 / 1e9;
    let locais = modelos_locais::escanear(est);
    let (desc, quando) = descobertos(est);
    let padrao = est.modelo_padrao().to_lowercase();
    let mut itens: Vec<Value> = Vec::new();
    let mut ids = std::collections::HashSet::new();
    let mut push = |id: String, nome: String, familia: String, repo: String, params_b: f64, rac: bool, uso: String, d: String, fonte: &str, tam: u64| {
        if !ids.insert(id.clone()) { return; }
        let inst = locais.iter().find(|a| a.nome.to_lowercase() == id.to_lowercase() || a.nome.to_lowercase().starts_with(&format!("{}-", id.to_lowercase())));
        let tam_est = if tam > 0 { tam } else { estimar(params_b) };
        let cabe = ram == 0 || (tam_est as f64 * 1.35) < ram as f64;
        let baixando = downloads::ativo_do_item(est, &format!("modelo:{}", id)).is_some();
        let mut it = json!({"id": id, "nome": nome, "familia": familia, "repo": repo, "params_b": params_b, "raciocina": rac, "uso": uso, "descricao": d, "fonte": fonte,
            "tamanho": tam_est, "cabe": cabe, "instalado": inst.is_some(), "baixando": baixando});
        if let Some(a) = inst { it["nome_local"] = json!(a.nome); it["ativo"] = json!(a.nome.to_lowercase() == padrao); }
        itens.push(it);
    };
    for b in EMBUTIDOS { push(b.id.into(), b.nome.into(), b.familia.into(), b.repo.into(), b.params_b, b.raciocina, b.uso.into(), b.desc.into(), "embutido", 0); }
    for d in desc {
        let repo = d["repo"].as_str().unwrap_or("").to_string();
        if repo.is_empty() { continue; }
        let id = slug(repo.split('/').last().unwrap_or(&repo).trim_end_matches("-GGUF").trim_end_matches("-gguf"));
        let nome = repo.split('/').last().unwrap_or(&repo).replace("-GGUF", "").replace("-gguf", "").replace('-', " ");
        push(id, nome, familia_de(&repo), repo.clone(), d["params_b"].as_f64().unwrap_or(0.0), repo.to_lowercase().contains("reason") || repo.to_lowercase().contains("r1") || repo.to_lowercase().contains("thinking"),
            "descoberto".into(), format!("Encontrado na pesquisa online ({} downloads).", d["downloads"].as_u64().unwrap_or(0)), "descoberto", 0);
    }
    // modelos que o usuário já tem mas não estão no catálogo (ex.: o gemma4 do Ollama)
    for a in &locais {
        let id = a.nome.clone();
        if itens.iter().any(|i| i["nome_local"].as_str() == Some(a.nome.as_str())) { continue; }
        let mut it = json!({"id": id, "nome": a.nome, "familia": familia_de(&a.nome), "repo": "", "params_b": 0.0, "raciocina": false, "uso": "seu", "descricao": "Já está no seu computador.", "fonte": "local",
            "tamanho": a.tamanho, "cabe": true, "instalado": true, "baixando": false, "nome_local": a.nome});
        it["ativo"] = json!(a.nome.to_lowercase() == padrao);
        itens.push(it);
    }
    json!({"itens": itens, "ram_gb": (ram_gb * 10.0).round() / 10.0, "atualizado": quando})
}

/// Pesquisa online e guarda os achados (o "reload" da lista).
pub fn atualizar(est: &Arc<Estado>) -> Result<usize, String> {
    let mut achados: Vec<Value> = Vec::new();
    let mut vistos = std::collections::HashSet::new();
    for q in ["instruct", "reasoning", "chat"] {
        let r = hf::buscar(q, 25)?;
        for m in r {
            let id = m["id"].as_str().unwrap_or("").to_string();
            let tags: Vec<String> = m["tags"].as_array().map(|a| a.iter().filter_map(|t| t.as_str().map(|s| s.to_lowercase())).collect()).unwrap_or_default();
            let texto = m["pipeline_tag"].as_str().map(|p| p == "text-generation").unwrap_or(true);
            let proibido = ["embed", "rerank", "whisper", "vl-", "vision", "mmproj", "lora", "tts", "clip"].iter().any(|p| id.to_lowercase().contains(p));
            if id.is_empty() || !hf::repo_valido(&id) || !texto || proibido || tags.iter().any(|t| t == "image-text-to-text") { continue; }
            if EMBUTIDOS.iter().any(|b| b.repo.eq_ignore_ascii_case(&id)) || !vistos.insert(id.clone()) { continue; }
            achados.push(json!({"repo": id, "downloads": m["downloads"].as_u64().unwrap_or(0), "likes": m["likes"].as_u64().unwrap_or(0)}));
        }
    }
    achados.sort_by_key(|a| std::cmp::Reverse(a["downloads"].as_u64().unwrap_or(0)));
    achados.truncate(30);
    let n = achados.len();
    let agora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    gravar_json(&arq_cache(est), &json!({"quando": agora, "itens": achados})).map_err(|e| e.to_string())?;
    Ok(n)
}

pub fn iniciar_atualizacao(est: &Arc<Estado>) -> String {
    if let Some(id) = downloads::ativo_do_item(est, "catalogo") { return id; }
    let id = downloads::novo(est, "catalogo", "Atualizando o catálogo de IAs", "catalogo", "ativo", 0);
    let (e, j) = (est.clone(), id.clone());
    std::thread::spawn(move || {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| atualizar(&e))).unwrap_or_else(|_| Err("erro interno".into()));
        downloads::concluir(&e, &j, r.map(|n| format!("{} IAs encontradas", n)));
    });
    id
}

fn achar(est: &Estado, id: &str) -> Option<(String, String, String)> {
    // (nome, repo, id)
    let l = lista(est);
    l["itens"].as_array()?.iter().find(|i| i["id"].as_str() == Some(id)).map(|i| (i["nome"].as_str().unwrap_or("").to_string(), i["repo"].as_str().unwrap_or("").to_string(), id.to_string()))
}

/// Baixa o modelo (bloqueante); usado pela tarefa em segundo plano e pela instalação do carregador.
pub fn baixar_agora(est: &Arc<Estado>, job: &str, id: &str) -> Result<String, String> {
    let (nome, repo, id) = achar(est, id).ok_or("Esse modelo não está no catálogo.")?;
    if repo.is_empty() { return Err("Esse modelo já está no computador.".into()); }
    downloads::mexer(est, job, |d| d.msg = "Consultando o Hugging Face…".into());
    let arq = hf::resolver(&repo)?;
    downloads::mexer(est, job, |d| { d.total = arq.tamanho; d.msg = format!("Baixando {}…", arq.nome); });
    let destino = est.base.join("modelos").join(format!("{}.gguf", id));
    hf::baixar(est, job, &repo, &arq.nome, &destino, arq.tamanho)?;
    let _ = modelos_locais::atualizar_catalogo(est);
    crate::carga::rechecar(est, "modelo");
    Ok(format!("{} instalado ({})", nome, tamanho_humano(arq.tamanho)))
}

/// Baixa um modelo do catálogo em segundo plano.
pub fn baixar(est: &Arc<Estado>, id: &str) -> Result<String, String> {
    let item = format!("modelo:{}", id);
    if let Some(j) = downloads::ativo_do_item(est, &item) { return Ok(j); }
    let (nome, repo, id) = achar(est, id).ok_or("Esse modelo não está no catálogo.")?;
    if repo.is_empty() { return Err("Esse modelo já está no computador.".into()); }
    let job = downloads::novo(est, &item, &format!("Baixando {}", nome), "baixar", "fila", 0);
    let (e, j) = (est.clone(), job.clone());
    std::thread::spawn(move || {
        let _g = e.fila.lock().unwrap_or_else(|x| x.into_inner());
        downloads::mexer(&e, &j, |d| { d.estado = "ativo".into(); d.desde_ms = agora_ms(); });
        let r = baixar_agora(&e, &j, &id);
        log(&e.base, &format!("baixar {}: {:?}", id, r));
        downloads::concluir(&e, &j, r);
    });
    Ok(job)
}

pub fn remover(est: &Estado, nome: &str) -> Result<(), String> {
    let n = nome_arquivo_seguro(nome);
    let p = est.base.join("modelos").join(format!("{}.gguf", n.trim_end_matches(".gguf")));
    if !p.is_file() { return Err("Só dá para remover modelos que estão na pasta modelos do Karthus.".into()); }
    std::fs::remove_file(&p).map_err(|e| e.to_string())?;
    let _ = modelos_locais::atualizar_catalogo(est);
    Ok(())
}

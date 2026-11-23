//! Consultor de recursos: lê o que o Karthus tem instalado (IAs de texto, modelos de imagem, VAE, LoRA, upscaler, ffmpeg)
//! e decide qual combinação dá a melhor imagem/vídeo neste computador. Tudo local.
use crate::catalogo;
use crate::estado::Estado;
use crate::util::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Arq { pub caminho: PathBuf, pub nome: String, pub tamanho: u64 }

fn listar(d: &Path, exts: &[&str]) -> Vec<Arq> {
    let mut v = Vec::new();
    if let Ok(rd) = std::fs::read_dir(d) {
        for e in rd.flatten() {
            let p = e.path();
            let n = e.file_name().to_string_lossy().to_string();
            let l = n.to_lowercase();
            if p.is_file() && exts.iter().any(|x| l.ends_with(x)) { v.push(Arq { tamanho: e.metadata().map(|m| m.len()).unwrap_or(0), caminho: p, nome: n }); }
        }
    }
    v.sort_by(|a, b| a.nome.cmp(&b.nome));
    v
}

/// Tipo do arquivo pelo nome/tamanho: base, vae, lora, upscaler.
pub fn classificar(a: &Arq) -> (&'static str, &'static str) {
    let l = a.nome.to_lowercase();
    if l.contains("vae") || l.starts_with("ae.") { return ("vae", ""); }
    if l.contains("lora") { return ("lora", ""); }
    if l.contains("esrgan") || l.contains("upscal") || l.contains("realesr") { return ("upscaler", ""); }
    let fam = if l.contains("flux") { "flux" } else if l.contains("sd3") || l.contains("stable-diffusion-3") { "sd3" }
        else if l.contains("turbo") || l.contains("lightning") || l.contains("lcm") { "turbo" }
        else if l.contains("xl") || l.contains("sdxl") { "sdxl" }
        else if a.tamanho > 5_500_000_000 { "sdxl" } else if l.contains("sd15") || l.contains("1.5") || l.contains("1-5") || l.contains("v1-5") { "sd15" }
        else if a.tamanho > 0 && a.tamanho < 700_000_000 { "pequeno" } else { "sd15" };
    ("base", fam)
}

fn pasta_img(est: &Estado) -> PathBuf { est.motores().join("imagem") }

pub fn inventario(est: &Estado) -> Value {
    let (ram, livre) = memoria();
    let dir = pasta_img(est);
    let todos: Vec<Arq> = [listar(&dir.join("modelos"), &[".safetensors", ".gguf", ".ckpt"]), listar(&dir, &[".safetensors", ".gguf", ".pth", ".ckpt"]), listar(&dir.join("vae"), &[".safetensors", ".pth", ".ckpt"]), listar(&dir.join("lora"), &[".safetensors"]), listar(&dir.join("upscaler"), &[".pth", ".bin", ".gguf", ".safetensors"])].concat();
    let imgs: Vec<Value> = todos.iter().map(|a| { let (t, f) = classificar(a); json!({"nome": a.nome, "tipo": t, "familia": f, "tamanho": a.tamanho}) }).collect();
    let cat = catalogo::lista(est);
    let llms: Vec<Value> = cat["itens"].as_array().map(|a| a.iter().filter(|i| i["instalado"] == json!(true)).map(|i| json!({"nome": i["nome_local"], "familia": i["familia"], "raciocina": i["raciocina"], "tamanho": i["tamanho"], "ativo": i["ativo"]})).collect()).unwrap_or_default();
    let m = est.motores();
    let tem = |p: PathBuf| p.is_file();
    json!({
        "hardware": {"ram_gb": (ram as f64 / 1e8).round() / 10.0, "ram_livre_gb": (livre as f64 / 1e8).round() / 10.0, "nucleos": nucleos()},
        "llms": llms,
        "imagem": {"sd": tem(dir.join("sd.exe")) || tem(dir.join("sd")), "arquivos": imgs},
        "video": {"ffmpeg": tem(m.join("video").join("ffmpeg.exe")) || tem(m.join("video").join("ffmpeg"))},
        "voz": {"whisper": !listar(&m.join("whisper"), &[".bin"]).is_empty()},
        "perfil": ler_json(&est.dados.join("imagem-perfil.json")).unwrap_or(Value::Null),
    })
}

/// Decide a melhor configuração de imagem/vídeo para este PC e explica o porquê.
pub fn decidir_imagem(est: &Estado) -> Value {
    let inv = inventario(est);
    let ram = inv["hardware"]["ram_gb"].as_f64().unwrap_or(0.0);
    let arqs = inv["imagem"]["arquivos"].as_array().cloned().unwrap_or_default();
    let mut motivos: Vec<String> = Vec::new();
    if inv["imagem"]["sd"] != json!(true) { return json!({"ok": false, "erro": "O gerador de imagens (sd.exe) não está instalado.", "motivos": ["Adicione o sd.exe em Motores > Imagem."]}); }
    // pontuação: qualidade da família, limitada pelo que cabe na RAM (modelo + ~2 GB de folga, e a IA de texto é descarregada)
    let pontos = |f: &str| match f { "flux" => 5, "sd3" => 4, "sdxl" => 4, "sd15" => 3, "turbo" => 2, _ => 1 };
    let mut bases: Vec<&Value> = arqs.iter().filter(|a| a["tipo"] == "base").collect();
    let cabe = |a: &Value| ram == 0.0 || (a["tamanho"].as_f64().unwrap_or(0.0) / 1e9) * 1.25 + 2.0 < ram;
    let (cabem, nao): (Vec<&Value>, Vec<&Value>) = bases.drain(..).partition(|a| cabe(a));
    for a in &nao { motivos.push(format!("{} ({}) é grande demais para {} GB de RAM e foi deixado de fora.", a["nome"].as_str().unwrap_or(""), a["familia"].as_str().unwrap_or(""), ram)); }
    let melhor = cabem.iter().max_by_key(|a| (pontos(a["familia"].as_str().unwrap_or("")), a["tamanho"].as_u64().unwrap_or(0))).cloned();
    let Some(base) = melhor else { return json!({"ok": false, "erro": "Nenhum modelo de imagem utilizável foi encontrado em Motores > Imagem.", "motivos": motivos}); };
    let fam = base["familia"].as_str().unwrap_or("sd15");
    motivos.push(format!("Modelo principal: {} — família {}, a melhor qualidade entre os que cabem neste PC.", base["nome"].as_str().unwrap_or(""), fam));
    let acha = |t: &str, f: &dyn Fn(&str) -> bool| arqs.iter().find(|a| a["tipo"] == t && f(&a["nome"].as_str().unwrap_or("").to_lowercase())).and_then(|a| a["nome"].as_str().map(|s| s.to_string()));
    let vae = acha("vae", &|n| if fam == "sdxl" { n.contains("xl") || !n.contains("sd15") } else if fam == "sd15" { !n.contains("xl") } else { true });
    if let Some(v) = &vae { motivos.push(format!("VAE {}: deixa as cores e os rostos mais nítidos.", v)); }
    let up = acha("upscaler", &|_| true);
    if let Some(u) = &up { motivos.push(format!("Ampliador {}: aumenta a imagem final sem perder detalhe (mais lento).", u)); }
    let lora = arqs.iter().any(|a| a["tipo"] == "lora");
    if lora { motivos.push("Há LoRAs instalados: ficam disponíveis na pasta lora (use <lora:nome:0.8> no pedido).".into()); }
    let (l, a, passos) = match fam { "flux" => (1024, 1024, 20), "sd3" => (1024, 1024, 28), "sdxl" => (1024, 1024, 25), "turbo" => (512, 512, 6), "pequeno" => (512, 512, 20), _ => (512, 512, 25) };
    let (l, a) = if ram > 0.0 && ram < 10.0 && l > 768 { (768, 768) } else { (l, a) };
    if l < 1024 && fam != "sd15" && fam != "turbo" && fam != "pequeno" { motivos.push("Resolução reduzida para 768 por pouca RAM.".into()); }
    let cpu = inv["hardware"]["nucleos"].as_u64().unwrap_or(2);
    if cpu < 6 { motivos.push(format!("Com {} núcleos a geração será lenta; reduza passos se quiser mais rapidez.", cpu)); }
    let perfil = json!({"ok": true, "modelo": base["nome"], "familia": fam, "vae": vae, "upscaler": up, "largura": l, "altura": a, "passos": passos, "lora_dir": lora, "motivos": motivos,
        "video": {"ffmpeg": inv["video"]["ffmpeg"], "cenas": if ram >= 16.0 { 4 } else { 3 }, "tamanho": if ram >= 16.0 { [768, 432] } else { [512, 288] }}});
    perfil
}

pub fn aplicar_imagem(est: &Estado) -> Value {
    let p = decidir_imagem(est);
    if p["ok"] == json!(true) { let _ = gravar_json(&est.dados.join("imagem-perfil.json"), &p); }
    p
}

/// Perfil gravado, com o caminho do modelo e os argumentos extras do sd.exe.
pub struct Perfil { pub modelo: Option<PathBuf>, pub extra: Vec<String>, pub largura: u32, pub altura: u32, pub passos: u32 }
pub fn perfil(est: &Estado) -> Option<Perfil> {
    let p: Value = ler_json(&est.dados.join("imagem-perfil.json"))?;
    if p["ok"] != json!(true) { return None; }
    let d = pasta_img(est);
    let achar = |sub: &str, nome: &str| [d.join(sub).join(nome), d.join(nome)].into_iter().find(|x| x.is_file());
    let modelo = p["modelo"].as_str().and_then(|n| achar("modelos", n));
    let mut extra = Vec::new();
    if let Some(v) = p["vae"].as_str().and_then(|n| achar("vae", n)) { extra.push("--vae".into()); extra.push(v.to_string_lossy().to_string()); }
    if let Some(u) = p["upscaler"].as_str().and_then(|n| achar("upscaler", n)) { extra.push("--upscale-model".into()); extra.push(u.to_string_lossy().to_string()); }
    if p["lora_dir"] == json!(true) && d.join("lora").is_dir() { extra.push("--lora-model-dir".into()); extra.push(d.join("lora").to_string_lossy().to_string()); }
    Some(Perfil { modelo, extra, largura: p["largura"].as_u64().unwrap_or(512) as u32, altura: p["altura"].as_u64().unwrap_or(512) as u32, passos: p["passos"].as_u64().unwrap_or(20) as u32 })
}

/// Texto que a IA recebe no chat quando o pedido é sobre recursos ("o que tenho?", "melhore minhas imagens", "baixe uma IA melhor").
pub fn contexto_para_ia(est: &Estado) -> String {
    let inv = inventario(est);
    let dec = decidir_imagem(est);
    let cat = catalogo::lista(est);
    let sug: Vec<String> = cat["itens"].as_array().map(|a| a.iter().filter(|i| i["instalado"] != json!(true) && i["cabe"] == json!(true) && i["fonte"] != json!("local")).take(6).map(|i| format!("- {} (id {}, {}, ~{} GB): {}", i["nome"].as_str().unwrap_or(""), i["id"].as_str().unwrap_or(""), i["uso"].as_str().unwrap_or(""), (i["tamanho"].as_f64().unwrap_or(0.0) / 1e8).round() / 10.0, i["descricao"].as_str().unwrap_or(""))).collect()).unwrap_or_default();
    format!("INVENTÁRIO REAL DO COMPUTADOR (lido agora pelo Karthus; não invente nada além disto):\n{}\n\nDECISÃO AUTOMÁTICA DE IMAGEM/VÍDEO:\n{}\n\nIAs de texto que cabem e ainda não estão instaladas:\n{}\n\nSe o usuário pedir, você pode oferecer estes botões escrevendo exatamente: [[baixar:ID]] para baixar uma IA do catálogo, [[usar:NOME]] para trocar a IA ativa, [[aplicar-imagem]] para aplicar a decisão de imagem/vídeo. Explique em português simples e só ofereça o que está na lista.",
        serde_json::to_string_pretty(&inv).unwrap_or_default(), serde_json::to_string_pretty(&dec).unwrap_or_default(), if sug.is_empty() { "(nenhuma)".into() } else { sug.join("\n") })
}

/// O pedido fala de recursos/instalação/melhoria?
pub fn fala_de_recursos(t: &str) -> bool {
    let l = t.to_lowercase();
    let gat = ["recurso", "melhor imagem", "melhorar imagem", "melhor vídeo", "melhor video", "melhorar vídeo", "adicion", "instal", "baixar uma ia", "baixe uma ia", "outra ia", "catálogo", "catalogo", "o que eu tenho", "o que tenho instalado", "modelo de imagem", "modelos de imagem", "qualidade da imagem", "que ia"];
    gat.iter().any(|g| l.contains(g)) && ["imagem", "imagens", "vídeo", "video", "ia", "modelo", "recurso", "motor"].iter().any(|k| l.split(|c: char| !c.is_alphanumeric()).any(|w| w == *k))
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test] fn classifica() {
        let a = |n: &str, t: u64| Arq { caminho: PathBuf::new(), nome: n.into(), tamanho: t };
        assert_eq!(classificar(&a("sdxl_vae.safetensors", 300)), ("vae", ""));
        assert_eq!(classificar(&a("flux1-dev-q4.gguf", 6_000_000_000)), ("base", "flux"));
        assert_eq!(classificar(&a("sd_turbo.safetensors", 2_000_000_000)), ("base", "turbo"));
        assert_eq!(classificar(&a("RealESRGAN_x4.pth", 60_000_000)), ("upscaler", ""));
        assert_eq!(classificar(&a("v1-5-pruned.safetensors", 4_000_000_000)), ("base", "sd15"));
    }
    #[test] fn intencao() {
        assert!(fala_de_recursos("quero melhorar a imagem, instale recursos novos"));
        assert!(!fala_de_recursos("qual a capital da França?"));
    }
}

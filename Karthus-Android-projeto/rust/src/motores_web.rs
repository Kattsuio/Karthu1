//! Baixa os motores de imagem e vídeo: sd.exe (stable-diffusion.cpp), um modelo de imagem pequeno (SD-Turbo) e o ffmpeg.
//! Os nomes dos arquivos das versões mudam, então o Karthus consulta a lista de arquivos da versão mais nova e escolhe o certo.
use crate::downloads;
use crate::estado::Estado;
use crate::hf;
use crate::util::*;
use serde_json::{json, Value};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn api_base() -> String { std::env::var("HIRELOD_GH").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "https://api.github.com".into()) }
fn teste() -> bool { std::env::var("HIRELOD_GH").is_ok() }

#[derive(Clone, Debug)]
pub struct Ativo { pub nome: String, pub url: String, pub tamanho: u64 }

pub fn assets(repo: &str, tag: &str) -> Result<Vec<Ativo>, String> {
    let rota = if tag.is_empty() { "latest".to_string() } else { format!("tags/{}", tag) };
    let url = format!("{}/repos/{}/releases/{}", api_base(), repo, rota);
    let ag = ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(10)).timeout(Duration::from_secs(30)).user_agent("Karthus/6.1").build();
    let j: Value = match ag.get(&url).set("Accept", "application/vnd.github+json").call() {
        Ok(r) => r.into_json().map_err(|e| format!("Resposta estranha do GitHub: {}", e))?,
        Err(ureq::Error::Status(403, _)) => return Err("O GitHub recusou a consulta agora (limite de acessos). Tente de novo em alguns minutos.".into()),
        Err(ureq::Error::Status(c, _)) => return Err(format!("O GitHub respondeu {} ao consultar {}.", c, repo)),
        Err(e) => return Err(format!("Sem conexão com o GitHub: {}", e)),
    };
    Ok(j["assets"].as_array().map(|a| a.iter().filter_map(|x| Some(Ativo { nome: x["name"].as_str()?.to_string(), url: x["browser_download_url"].as_str()?.to_string(), tamanho: x["size"].as_u64().unwrap_or(0) })).collect()).unwrap_or_default())
}


/// Procura nas últimas versões (não só na "latest": algumas versões saem sem os programas para Windows).
/// Prefere versões estáveis; se nenhuma tiver o pacote, aceita as pré-lançamentos oficiais.
pub fn achar_em_releases(repo: &str, escolher: &dyn Fn(&[Ativo]) -> Option<Ativo>) -> Result<Ativo, String> {
    let url = format!("{}/repos/{}/releases?per_page=30", api_base(), repo);
    let ag = ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(10)).timeout(Duration::from_secs(30)).user_agent("Karthus/6.1").build();
    let j: Value = match ag.get(&url).set("Accept", "application/vnd.github+json").call() {
        Ok(r) => r.into_json().map_err(|e| format!("Resposta estranha do GitHub: {}", e))?,
        Err(ureq::Error::Status(403, _)) => return Err("O GitHub recusou a consulta agora (limite de acessos). Tente de novo em alguns minutos.".into()),
        Err(ureq::Error::Status(c, _)) => return Err(format!("O GitHub respondeu {} ao consultar {}.", c, repo)),
        Err(e) => return Err(format!("Sem conexão com o GitHub: {}", e)),
    };
    let lista = j.as_array().cloned().unwrap_or_default();
    for pre in [false, true] {
        for r in lista.iter().filter(|r| r["draft"] != json!(true) && (r["prerelease"] == json!(true)) == pre) {
            let ativos: Vec<Ativo> = r["assets"].as_array().map(|a| a.iter().filter_map(|x| Some(Ativo { nome: x["name"].as_str()?.to_string(), url: x["browser_download_url"].as_str()?.to_string(), tamanho: x["size"].as_u64().unwrap_or(0) })).collect()).unwrap_or_default();
            if let Some(a) = escolher(&ativos) { return Ok(a); }
        }
    }
    Err("sem pacote".into())
}

/// Escolhe o pacote do sd.exe para CPU no Windows (o mais compatível que o PC aguente: avx2).
pub fn escolher_sd(a: &[Ativo]) -> Option<Ativo> {
    let ok: Vec<&Ativo> = a.iter().filter(|x| { let l = x.nome.to_lowercase(); l.ends_with(".zip") && l.contains("win") && !["cuda", "cudart", "vulkan", "rocm", "hip", "opencl", "sycl", "arm", "noavx", "src"].iter().any(|p| l.contains(p)) }).collect();
    for pref in ["avx2", "avx512", "avx"] { if let Some(x) = ok.iter().find(|x| x.nome.to_lowercase().contains(pref)) { return Some((*x).clone()); } }
    ok.first().map(|x| (*x).clone())
}
/// Pacote do whisper.cpp para Windows, CPU (o nome mudou entre versões: aceita whisper-bin-x64.zip e parecidos).
pub fn escolher_whisper(a: &[Ativo]) -> Option<Ativo> {
    a.iter().find(|x| { let l = x.nome.to_lowercase(); l.ends_with(".zip") && l.contains("bin") && l.contains("x64") && !["cublas", "cuda", "blas", "vulkan", "openvino", "src"].iter().any(|p| l.contains(p)) }).cloned()
}
pub fn instalar_whisper_exe(est: &Estado, job: &str) -> Result<(), String> {
    let dir = est.motores().join("whisper");
    if crate::carga::whisper_exe(est).is_some() { return Ok(()); }
    let a = achar_em_releases("ggml-org/whisper.cpp", &escolher_whisper).map_err(|e| if e == "sem pacote" { "Não achei o pacote do whisper.cpp para Windows nas versões recentes. Baixe o whisper-bin-x64.zip em github.com/ggml-org/whisper.cpp/releases e use Criar > Motores.".to_string() } else { e })?;
    let feitos = baixar_zip(est, job, &a, &dir, &|n| { let l = n.to_lowercase(); let base = l.rsplit('/').next().unwrap_or(""); if base.ends_with(".exe") || base.ends_with(".dll") { Some(base.to_string()) } else { None } })?;
    if !feitos.iter().any(|n| n == "whisper-cli.exe" || n == "main.exe") { return Err(format!("O pacote {} não trouxe o whisper-cli.exe (veio: {}).", a.nome, feitos.join(", "))); }
    Ok(())
}
pub fn escolher_ffmpeg(a: &[Ativo]) -> Option<Ativo> {
    a.iter().find(|x| { let l = x.nome.to_lowercase(); l.ends_with("win64-gpl.zip") && !l.contains("shared") && !l.contains("lgpl") }).cloned()
}

fn url_confiavel(u: &str) -> bool { teste() || u.starts_with("https://github.com/") }

/// Baixa o zip e tira só os arquivos úteis (.exe/.dll) para a pasta de destino.
fn baixar_zip(est: &Estado, job: &str, a: &Ativo, destino: &Path, quer: &dyn Fn(&str) -> Option<String>) -> Result<Vec<String>, String> {
    if !url_confiavel(&a.url) { return Err("O endereço do arquivo não é do GitHub oficial; recusei por segurança.".into()); }
    let zip = est.dados.join("tmp").join(&nome_arquivo_seguro(&a.nome));
    downloads::mexer(est, job, |d| d.msg = format!("Baixando {}…", a.nome));
    hf::baixar_url(est, job, &a.url, &zip, a.tamanho)?;
    downloads::mexer(est, job, |d| { d.msg = "Extraindo…".into(); });
    let f = std::fs::File::open(&zip).map_err(|e| e.to_string())?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| format!("O arquivo baixado está corrompido: {}", e))?;
    std::fs::create_dir_all(destino).map_err(|e| e.to_string())?;
    let mut feitos = Vec::new();
    for i in 0..z.len() {
        let mut e = z.by_index(i).map_err(|e| e.to_string())?;
        if e.is_dir() { continue; }
        let Some(nome) = e.enclosed_name().map(|p| p.to_string_lossy().replace('\\', "/")) else { continue };
        let Some(saida) = quer(&nome) else { continue };
        let dst = destino.join(nome_arquivo_seguro(&saida));
        let parte = dst.with_extension("parte");
        let mut o = std::fs::File::create(&parte).map_err(|e| format!("Não consegui gravar {}: {}", saida, e))?;
        std::io::copy(&mut (&mut e).take(1_500_000_000), &mut o).map_err(|e| format!("Falha ao extrair {}: {}", saida, e))?;
        drop(o);
        let _ = std::fs::remove_file(&dst);
        std::fs::rename(&parte, &dst).map_err(|e| e.to_string())?;
        feitos.push(saida);
    }
    let _ = std::fs::remove_file(&zip);
    Ok(feitos)
}

pub fn instalar_sd(est: &Estado, job: &str) -> Result<(), String> {
    let dir = est.motores().join("imagem");
    if dir.join("sd.exe").is_file() { return Ok(()); }
    let a = achar_em_releases("leejet/stable-diffusion.cpp", &escolher_sd).map_err(|e| if e == "sem pacote" { "Não achei o pacote do sd.exe para Windows nas versões recentes do stable-diffusion.cpp.".to_string() } else { e })?;
    let feitos = baixar_zip(est, job, &a, &dir, &|n| { let l = n.to_lowercase(); let base = l.rsplit('/').next().unwrap_or(""); if base.ends_with(".exe") || base.ends_with(".dll") { Some(base.to_string()) } else { None } })?;
    // versões novas chamam o programa de sd-cli.exe: guarda também como sd.exe
    if !dir.join("sd.exe").is_file() {
        let cli = feitos.iter().find(|n| n.starts_with("sd-cli") || n == &"sd.exe").cloned();
        match cli { Some(c) => { std::fs::copy(dir.join(&c), dir.join("sd.exe")).map_err(|e| e.to_string())?; } None => return Err(format!("O pacote {} não trouxe um sd.exe (veio: {}).", a.nome, feitos.join(", "))) }
    }
    Ok(())
}

/// SD-Turbo quantizado (~2 GB): gera uma imagem em poucos passos, bom para PC sem placa de vídeo forte.
pub fn instalar_modelo_imagem(est: &Estado, job: &str) -> Result<(), String> {
    let md = est.motores().join("imagem").join("modelos");
    let tem = ["safetensors", "gguf", "ckpt"].iter().any(|e| crate::carga::achar_arquivo(&md, "", &format!(".{}", e)).is_some());
    if tem { return Ok(()); }
    downloads::mexer(est, job, |d| d.msg = "Baixando o modelo de imagem SD-Turbo (~2 GB)…".into());
    hf::baixar(est, job, "Green-Sky/SD-Turbo-GGUF", "sd_turbo-f16-q8_0.gguf", &md.join("sd_turbo-f16-q8_0.gguf"), 2_020_000_000)
}

pub fn instalar_ffmpeg(est: &Estado, job: &str) -> Result<(), String> {
    let dir = est.motores().join("video");
    if dir.join("ffmpeg.exe").is_file() { return Ok(()); }
    let lista = assets("BtbN/FFmpeg-Builds", "latest")?;
    let a = escolher_ffmpeg(&lista).ok_or("Não achei o pacote do ffmpeg para Windows.")?;
    let feitos = baixar_zip(est, job, &a, &dir, &|n| if n.to_lowercase().ends_with("/bin/ffmpeg.exe") || n.to_lowercase() == "ffmpeg.exe" { Some("ffmpeg.exe".into()) } else { None })?;
    if feitos.is_empty() { return Err("O pacote do ffmpeg não trouxe o ffmpeg.exe.".into()); }
    Ok(())
}


// ------------------------------------------------------------------------------------------ vozes em português (Piper, offline)
/// (id, nome, pasta no Hugging Face rhasspy/piper-voices, nome do arquivo, tamanho aproximado)
pub const VOZES_PIPER: [(&str, &str, &str, &str, u64); 4] = [
    ("faber", "Faber (masculina, pt-BR)", "pt/pt_BR/faber/medium", "pt_BR-faber-medium", 63_000_000),
    ("cadu", "Cadu (masculina, pt-BR)", "pt/pt_BR/cadu/medium", "pt_BR-cadu-medium", 63_000_000),
    ("jeff", "Jeff (masculina, pt-BR)", "pt/pt_BR/jeff/medium", "pt_BR-jeff-medium", 63_000_000),
    ("edresson", "Edresson (masculina, pt-BR, leve)", "pt/pt_BR/edresson/low", "pt_BR-edresson-low", 63_000_000),
];
pub fn pasta_piper(est: &Estado) -> PathBuf { est.motores().join("voz").join("piper") }
pub fn piper_exe(est: &Estado) -> Option<PathBuf> { let p = pasta_piper(est).join("piper.exe"); if p.is_file() { Some(p) } else { let q = pasta_piper(est).join("piper"); if q.is_file() { Some(q) } else { None } } }
pub fn voz_piper_arquivo(est: &Estado, id: &str) -> Option<PathBuf> {
    let (_, _, _, nome, _) = VOZES_PIPER.iter().find(|v| v.0 == id)?;
    let p = pasta_piper(est).join("vozes").join(format!("{}.onnx", nome));
    if p.is_file() && p.with_extension("onnx.json").is_file() { Some(p) } else { None }
}

fn instalar_piper_exe(est: &Estado, job: &str) -> Result<(), String> {
    if piper_exe(est).is_some() { return Ok(()); }
    let a = Ativo { nome: "piper_windows_amd64.zip".into(), url: "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_windows_amd64.zip".into(), tamanho: 22_000_000 };
    if !url_confiavel(&a.url) { return Err("Endereço não confiável.".into()); }
    let zip = est.dados.join("tmp").join("piper.zip");
    downloads::mexer(est, job, |d| d.msg = "Baixando o motor de voz (Piper)…".into());
    hf::baixar_url(est, job, &a.url, &zip, a.tamanho)?;
    downloads::mexer(est, job, |d| d.msg = "Extraindo…".into());
    let f = std::fs::File::open(&zip).map_err(|e| e.to_string())?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| format!("O arquivo baixado está corrompido: {}", e))?;
    let destino = pasta_piper(est);
    std::fs::create_dir_all(&destino).map_err(|e| e.to_string())?;
    for i in 0..z.len() {
        let mut e = z.by_index(i).map_err(|e| e.to_string())?;
        if e.is_dir() { continue; }
        let Some(rel) = e.enclosed_name() else { continue };
        let comps: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
        let comps = if comps.first().map(|c| c == "piper").unwrap_or(false) { comps[1..].to_vec() } else { comps };
        if comps.is_empty() { continue; }
        let dst = comps.iter().fold(destino.clone(), |p, c| p.join(c));
        if !dst.starts_with(&destino) { continue; }
        if let Some(pp) = dst.parent() { std::fs::create_dir_all(pp).map_err(|e| e.to_string())?; }
        let mut o = std::fs::File::create(&dst).map_err(|e| format!("Não consegui gravar {}: {}", dst.display(), e))?;
        std::io::copy(&mut (&mut e).take(400_000_000), &mut o).map_err(|e| e.to_string())?;
    }
    let _ = std::fs::remove_file(&zip);
    if piper_exe(est).is_none() { return Err("O pacote do Piper não trouxe o piper.exe.".into()); }
    Ok(())
}

/// Instala o Piper (uma vez) e a voz escolhida.
pub fn instalar_voz_piper(est: &Estado, job: &str, id: &str) -> Result<(), String> {
    let (_, _, pasta, nome, tam) = *VOZES_PIPER.iter().find(|v| v.0 == id).ok_or("Voz desconhecida.")?;
    if voz_piper_arquivo(est, id).is_some() && piper_exe(est).is_some() { return Ok(()); }
    instalar_piper_exe(est, job)?;
    let dir = pasta_piper(est).join("vozes");
    downloads::mexer(est, job, |d| d.msg = format!("Baixando a voz {}…", nome));
    let base = format!("{}/rhasspy/piper-voices/resolve/main/{}/{}", hf::base(), pasta, nome);
    hf::baixar_url(est, job, &format!("{}.onnx.json", base), &dir.join(format!("{}.onnx.json", nome)), 6_000)?;
    hf::baixar_url(est, job, &format!("{}.onnx", base), &dir.join(format!("{}.onnx", nome)), tam)?;
    sanear_json_voz(&dir.join(format!("{}.onnx.json", nome)));
    Ok(())
}

/// O Piper para Windows (2023) só aceita fonemas de UM caractere no .onnx.json; as vozes novas trazem
/// chaves como "aɪ". Remove essas chaves (o motor fala os dois sons em sequência). Devolve quantas tirou.
pub fn sanear_json_voz(p: &Path) -> usize {
    let Ok(t) = std::fs::read_to_string(p) else { return 0 };
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&t) else { return 0 };
    let mut n = 0;
    for campo in ["phoneme_id_map", "phoneme_map"] {
        if let Some(o) = v.get_mut(campo).and_then(|x| x.as_object_mut()) {
            let ruins: Vec<String> = o.keys().filter(|k| k.chars().count() != 1).cloned().collect();
            n += ruins.len();
            for k in ruins { o.remove(&k); }
        }
    }
    if n > 0 { let _ = std::fs::write(p, serde_json::to_string(&v).unwrap_or(t)); }
    n
}

#[allow(dead_code)]
fn _usa(_: PathBuf) {}

#[cfg(test)]
mod testes {
    use super::*;
    fn a(n: &str) -> Ativo { Ativo { nome: n.into(), url: format!("https://github.com/x/{}", n), tamanho: 1 } }
    #[test] fn escolhe_sd() {
        let l = vec![a("sd-master-1-bin-Linux-Ubuntu-x86_64.zip"), a("sd-master-1-bin-win-cuda12-x64.zip"), a("sd-master-1-bin-win-vulkan-x64.zip"), a("sd-master-1-bin-win-avx512-x64.zip"), a("sd-master-1-bin-win-avx2-x64.zip"), a("sd-master-1-bin-win-noavx-x64.zip"), a("cudart-sd-bin-win-cu12-x64.zip")];
        assert_eq!(escolher_sd(&l).unwrap().nome, "sd-master-1-bin-win-avx2-x64.zip");
        assert!(escolher_sd(&[a("sd-bin-linux.zip")]).is_none());
    }
    #[test] fn saneia_voz() {
        let d = std::env::temp_dir().join("sv_teste.json");
        std::fs::write(&d, r#"{"phoneme_id_map":{"a":[1],"aɪ":[161],"ɪ":[2]},"audio":{"sample_rate":22050}}"#).unwrap();
        assert_eq!(sanear_json_voz(&d), 1);
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&d).unwrap()).unwrap();
        assert!(v["phoneme_id_map"].get("aɪ").is_none() && v["phoneme_id_map"].get("a").is_some() && v["audio"]["sample_rate"] == 22050);
        assert_eq!(sanear_json_voz(&d), 0);
    }
    #[test] fn escolhe_whisper() {
        let l = vec![a("whisper-cublas-12.4.0-bin-x64.zip"), a("whisper-blas-bin-x64.zip"), a("whisper-bin-Win32.zip"), a("whisper-bin-x64.zip")];
        assert_eq!(escolher_whisper(&l).unwrap().nome, "whisper-bin-x64.zip");
    }
    #[test] fn escolhe_ffmpeg() {
        let l = vec![a("ffmpeg-master-latest-win64-gpl-shared.zip"), a("ffmpeg-master-latest-win64-lgpl.zip"), a("ffmpeg-master-latest-win64-gpl.zip"), a("ffmpeg-master-latest-linux64-gpl.tar.xz")];
        assert_eq!(escolher_ffmpeg(&l).unwrap().nome, "ffmpeg-master-latest-win64-gpl.zip");
    }

    #[test] fn procura_em_varias_versoes() {
        let srv = tiny_http::Server::http("127.0.0.1:0").unwrap(); let porta = srv.server_addr().to_ip().unwrap().port();
        std::thread::spawn(move || { for rq in srv.incoming_requests() { let b = json!([
            {"tag_name": "v1.9.5", "prerelease": false, "draft": false, "assets": [{"name": "source.zip", "browser_download_url": "https://github.com/x/source.zip", "size": 1}]},
            {"tag_name": "b5454", "prerelease": true, "draft": false, "assets": [{"name": "whisper-bin-x64.zip", "browser_download_url": "https://github.com/x/pre.zip", "size": 1}]},
            {"tag_name": "v1.9.2", "prerelease": false, "draft": false, "assets": [{"name": "whisper-blas-bin-x64.zip", "browser_download_url": "https://github.com/x/blas.zip", "size": 1}, {"name": "whisper-bin-x64.zip", "browser_download_url": "https://github.com/x/ok.zip", "size": 1}]}]);
            let _ = rq.respond(tiny_http::Response::from_string(b.to_string())); } });
        std::env::set_var("HIRELOD_GH", format!("http://127.0.0.1:{}", porta));
        let a = achar_em_releases("ggml-org/whisper.cpp", &escolher_whisper).unwrap();
        assert_eq!(a.url, "https://github.com/x/ok.zip");
    }
}

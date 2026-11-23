//! Cortes: pega um vídeo longo, transcreve (whisper local), escolhe os melhores trechos e corta com ffmpeg.
//! 100% local e gratuito: sem chave, sem tokens, sem internet depois dos motores instalados.
use crate::carga;
use crate::downloads;
use crate::estado::Estado;
use crate::util::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Seg { pub ini: f64, pub fim: f64, pub texto: String }

fn tempo_srt(t: &str) -> Option<f64> {
    let t = t.trim().replace(',', ".");
    let p: Vec<&str> = t.split(':').collect();
    if p.len() != 3 { return None; }
    Some(p[0].parse::<f64>().ok()? * 3600.0 + p[1].parse::<f64>().ok()? * 60.0 + p[2].parse::<f64>().ok()?)
}
pub fn ler_srt(s: &str) -> Vec<Seg> {
    let mut v = Vec::new();
    for bloco in s.replace("\r\n", "\n").split("\n\n") {
        let l: Vec<&str> = bloco.lines().filter(|x| !x.trim().is_empty()).collect();
        let Some(i) = l.iter().position(|x| x.contains("-->")) else { continue };
        let Some((a, b)) = l[i].split_once("-->") else { continue };
        let (Some(a), Some(b)) = (tempo_srt(a), tempo_srt(b)) else { continue };
        let texto = l[i + 1..].join(" ").trim().to_string();
        if b > a && !texto.is_empty() { v.push(Seg { ini: a, fim: b, texto }); }
    }
    v
}
fn fmt_srt(t: f64) -> String { let ms = (t * 1000.0).round() as u64; format!("{:02}:{:02}:{:02},{:03}", ms / 3600000, ms / 60000 % 60, ms / 1000 % 60, ms % 1000) }
pub fn escrever_srt(segs: &[Seg], deslocar: f64) -> String {
    segs.iter().enumerate().map(|(i, s)| format!("{}\n{} --> {}\n{}\n\n", i + 1, fmt_srt((s.ini - deslocar).max(0.0)), fmt_srt((s.fim - deslocar).max(0.0)), s.texto)).collect()
}

const GANCHOS: [&str; 18] = ["segredo", "erro", "nunca", "sempre", "dinheiro", "verdade", "importante", "cuidado", "atenção", "incrível", "como", "por que", "porque", "dica", "resultado", "problema", "solução", "gratuito"];
fn pontuar(segs: &[Seg]) -> f64 {
    if segs.is_empty() { return 0.0; }
    let dur = (segs.last().unwrap().fim - segs[0].ini).max(1.0);
    let texto: String = segs.iter().map(|s| s.texto.to_lowercase()).collect::<Vec<_>>().join(" ");
    let palavras = texto.split_whitespace().count() as f64;
    let ganchos = GANCHOS.iter().filter(|g| texto.contains(*g)).count() as f64;
    let perguntas = texto.matches('?').count() as f64;
    let exclam = texto.matches('!').count() as f64;
    let ritmo = (palavras / dur).min(3.5); // fala densa = trecho com conteúdo
    ritmo * 2.0 + ganchos * 0.8 + perguntas * 0.6 + exclam * 0.4
}

/// Janelas candidatas que começam e terminam em frases inteiras, com duração entre min e max.
pub fn candidatos(segs: &[Seg], min: f64, max: f64) -> Vec<(f64, f64, f64)> {
    let mut v = Vec::new();
    for i in 0..segs.len() {
        let mut j = i;
        while j < segs.len() && segs[j].fim - segs[i].ini <= max { j += 1; }
        if j == i { continue; }
        let fim_idx = j - 1;
        let dur = segs[fim_idx].fim - segs[i].ini;
        if dur >= min { v.push((segs[i].ini, segs[fim_idx].fim, pontuar(&segs[i..=fim_idx]))); }
    }
    v
}
/// Escolhe os n melhores sem sobreposição.
pub fn escolher(segs: &[Seg], n: usize, min: f64, max: f64) -> Vec<(f64, f64)> {
    let mut c = candidatos(segs, min, max);
    c.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    let mut r: Vec<(f64, f64)> = Vec::new();
    for (a, b, _) in c { if r.len() >= n { break; } if r.iter().all(|(x, y)| b <= *x || a >= *y) { r.push((a, b)); } }
    r.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    r
}

fn duracao(ff: &Path, video: &Path) -> Option<f64> {
    let mut c = Command::new(ff); c.arg("-hide_banner").arg("-i").arg(video).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()); sem_janela(&mut c);
    let o = c.output().ok()?; let s = String::from_utf8_lossy(&o.stderr).to_string();
    let p = s.find("Duration:")?; tempo_srt(s[p + 9..].split(',').next()?.trim())
}

fn ffmpeg(ff: &Path, args: &[String], cwd: &Path) -> Result<(), String> {
    let mut c = Command::new(ff); c.args(["-y", "-hide_banner", "-loglevel", "error", "-nostdin"]).args(args).current_dir(cwd).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()); sem_janela(&mut c);
    let o = c.output().map_err(|e| format!("ffmpeg: {}", e))?;
    if o.status.success() { Ok(()) } else { Err(format!("ffmpeg: {}", String::from_utf8_lossy(&o.stderr).lines().last().unwrap_or("falhou").chars().take(200).collect::<String>())) }
}

/// Opcional: a IA local escolhe os trechos mais interessantes (texto curto, sem enviar nada para fora).
fn escolher_com_ia(est: &Estado, modelo: &str, segs: &[Seg], n: usize, min: f64, max: f64) -> Option<Vec<(f64, f64)>> {
    if modelo.is_empty() || est.ollama.versao().is_err() { return None; }
    let mut t = String::new();
    for s in segs { let l = format!("[{:.0}-{:.0}] {}\n", s.ini, s.fim, s.texto); if t.len() + l.len() > 5500 { break; } t += &l; }
    let c = json!({"model": modelo, "stream": false, "think": false, "keep_alive": 7200, "options": {"num_ctx": 4096, "num_predict": 300, "temperature": 0.2},
        "messages": [{"role": "system", "content": format!("Você escolhe os {} trechos mais interessantes de uma transcrição para virarem cortes curtos ({}–{} segundos cada). Responda SOMENTE com JSON: [{{\"ini\":segundos,\"fim\":segundos}}]. Use os tempos da transcrição.", n, min as u32, max as u32)}, {"role": "user", "content": t}]});
    let r = est.ollama.chat_json(&c).ok()?;
    let (a, b) = (r.find('[')?, r.rfind(']')?);
    let v: Vec<Value> = serde_json::from_str(&r[a..=b]).ok()?;
    let fim_total = segs.last()?.fim;
    let mut out: Vec<(f64, f64)> = Vec::new();
    for x in v { let (i, f) = (x["ini"].as_f64()?, x["fim"].as_f64()?); if f - i >= min * 0.6 && f - i <= max * 1.3 && f <= fim_total + 1.0 && i >= 0.0 && out.iter().all(|(p, q)| f <= *p || i >= *q) { out.push((i, f)); } }
    if out.is_empty() { None } else { out.truncate(n); out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap()); Some(out) }
}

/// Estimativa de entrega em segundos: áudio + transcrição (~0,35x do vídeo) + cortes.
pub fn estimativa(dur: f64, n: usize) -> u64 { (20.0 + dur * 0.4 + n as f64 * 15.0) as u64 }

pub struct Opcoes { pub quantidade: usize, pub min: f64, pub max: f64, pub vertical: bool, pub legendas: bool, pub usar_ia: bool }

pub fn iniciar(est: &Arc<Estado>, uid: &str, video: &str, o: Opcoes, modelo: String) -> Result<String, String> {
    let ff = carga::ffmpeg_exe(est).ok_or("O ffmpeg não está instalado. Abra o ícone vermelho > Instalar (vídeo).")?;
    let wx = carga::whisper_exe(est).ok_or("O transcritor (whisper) não está instalado. Abra o ícone vermelho > Instalar (voz).")?;
    let wm = carga::whisper_modelo(est).ok_or("O modelo do transcritor não está instalado.")?;
    let video = PathBuf::from(video.trim().trim_matches('"'));
    if !video.is_file() { return Err("Não achei esse arquivo de vídeo.".into()); }
    let ext = video.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    if !["mp4", "mkv", "mov", "avi", "webm", "m4v", "flv", "wmv", "mp3", "wav", "m4a"].contains(&ext.as_str()) { return Err("Formato não suportado. Use mp4, mkv, mov, avi ou webm.".into()); }
    let o = Opcoes { quantidade: o.quantidade.clamp(1, 10), min: o.min.clamp(10.0, 120.0), max: o.max.clamp(15.0, 180.0).max(o.min + 5.0), ..o };
    let nome = video.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or("video".into());
    let job = downloads::novo_cat(est, &format!("cortes:{}", novo_id("c")), &format!("Cortes: {}", nome.chars().take(36).collect::<String>()), "cortes", "fila", 0, "criacao");
    let (e, j, u) = (est.clone(), job.clone(), uid.to_string());
    std::thread::spawn(move || {
        let _g = e.fila_criar.lock().unwrap_or_else(|x| x.into_inner());
        downloads::mexer(&e, &j, |d| { d.estado = "ativo".into(); d.desde_ms = agora_ms(); });
        let work = e.dados.join("tmp").join(novo_id("cut"));
        let r = (|| -> Result<usize, String> {
            std::fs::create_dir_all(&work).map_err(|x| x.to_string())?;
            let dur = duracao(&ff, &video).ok_or("Não consegui ler a duração do vídeo.")?;
            let est_s = estimativa(dur, o.quantidade);
            downloads::mexer(&e, &j, |d| d.eta_s = est_s);
            downloads::pct(&e, &j, 3.0, &format!("Extraindo o áudio (entrega estimada em ~{} min)…", (est_s + 59) / 60));
            ffmpeg(&ff, &["-i".into(), video.to_string_lossy().to_string(), "-vn".into(), "-ac".into(), "1".into(), "-ar".into(), "16000".into(), "audio.wav".into()], &work)?;
            downloads::pct(&e, &j, 10.0, "Transcrevendo a fala (local)…");
            let mut c = Command::new(&wx);
            c.arg("-m").arg(&wm).arg("-f").arg(work.join("audio.wav")).args(["-l", "pt", "-osrt", "-np", "-of"]).arg(work.join("fala")).arg("-t").arg(crate::ia::threads_ideais().to_string());
            c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()); sem_janela(&mut c);
            // transcrição pode levar minutos: acompanha com o relógio para a barra andar
            let mut f = c.spawn().map_err(|x| format!("Não consegui rodar o whisper: {}", x))?;
            let ini = std::time::Instant::now(); let alvo = (dur * 0.35).max(8.0);
            let st = loop {
                if let Some(s) = f.try_wait().map_err(|x| x.to_string())? { break s; }
                let fr = (ini.elapsed().as_secs_f64() / alvo).min(0.97);
                downloads::pct(&e, &j, 10.0 + fr * 55.0, "Transcrevendo a fala (local)…");
                std::thread::sleep(std::time::Duration::from_millis(500));
            };
            let srt = std::fs::read_to_string(work.join("fala.srt")).unwrap_or_default();
            if !st.success() && srt.is_empty() { return Err("A transcrição falhou.".into()); }
            let segs = ler_srt(&srt);
            if segs.is_empty() { return Err("Não encontrei fala no vídeo para cortar.".into()); }
            downloads::pct(&e, &j, 68.0, "Escolhendo os melhores trechos…");
            let trechos = if o.usar_ia { escolher_com_ia(&e, &modelo, &segs, o.quantidade, o.min, o.max) } else { None }.unwrap_or_else(|| escolher(&segs, o.quantidade, o.min, o.max));
            let trechos = if trechos.is_empty() { vec![(0.0, dur.min(o.max))] } else { trechos };
            let n = trechos.len();
            for (i, (a, b)) in trechos.iter().enumerate() {
                downloads::pct(&e, &j, 70.0 + i as f64 * 29.0 / n as f64, &format!("Cortando {} de {}…", i + 1, n));
                let dentro: Vec<Seg> = segs.iter().filter(|s| s.fim > *a && s.ini < *b).cloned().collect();
                let mut vf: Vec<String> = Vec::new();
                if o.vertical { vf.push("crop=ih*9/16:ih,scale=1080:1920".into()); }
                if o.legendas && !dentro.is_empty() {
                    std::fs::write(work.join("leg.srt"), escrever_srt(&dentro, *a)).map_err(|x| x.to_string())?;
                    vf.push(format!("subtitles=leg.srt:force_style='FontSize={},Outline=2,Alignment=2,MarginV={}'", if o.vertical { 14 } else { 20 }, if o.vertical { 120 } else { 30 }));
                }
                let saida = format!("corte-{}-{:02}.mp4", novo_id(""), i + 1);
                let mut args = vec!["-ss".into(), format!("{:.2}", a), "-i".into(), video.to_string_lossy().to_string(), "-t".into(), format!("{:.2}", b - a)];
                if !vf.is_empty() { args.push("-vf".into()); args.push(vf.join(",")); }
                args.extend(["-c:v".into(), "libx264".into(), "-preset".into(), "veryfast".into(), "-crf".into(), "22".into(), "-c:a".into(), "aac".into(), "-movflags".into(), "+faststart".into(), e.saidas().join(&saida).to_string_lossy().to_string()]);
                ffmpeg(&ff, &args, &work)?;
                let resumo: String = dentro.iter().map(|s| s.texto.as_str()).collect::<Vec<_>>().join(" ").chars().take(90).collect();
                crate::criar::registrar(&e, &u, "corte", &saida, &format!("Corte {} · {}", i + 1, nome.chars().take(24).collect::<String>()), json!({"ini": a, "fim": b, "trecho": resumo}));
                downloads::mexer(&e, &j, |d| d.arquivo = saida.clone());
            }
            Ok(n)
        })();
        let _ = std::fs::remove_dir_all(&work);
        log(&e.base, &format!("cortes: {:?}", r));
        downloads::concluir(&e, &j, r.map(|n| format!("{} cortes prontos", n)));
    });
    Ok(job)
}

#[cfg(test)]
mod testes {
    use super::*;
    fn exemplo() -> Vec<Seg> { (0..40).map(|i| Seg { ini: i as f64 * 5.0, fim: i as f64 * 5.0 + 4.5, texto: if i / 8 == 2 { "o segredo importante: como evitar esse erro?".into() } else { "ok".into() } }).collect() }
    #[test] fn srt() {
        let s = ler_srt("1\n00:00:01,000 --> 00:00:03,500\nolá mundo\n\n2\n00:01:00,000 --> 00:01:02,000\ntchau\n");
        assert_eq!(s.len(), 2); assert!((s[1].ini - 60.0).abs() < 1e-6);
        assert!(escrever_srt(&s, 1.0).contains("00:00:00,000 --> 00:00:02,500"));
    }
    #[test] fn escolhe_sem_sobrepor_e_acha_o_melhor() {
        let r = escolher(&exemplo(), 2, 20.0, 40.0);
        assert_eq!(r.len(), 2); assert!(r[0].1 <= r[1].0 || r[1].1 <= r[0].0);
        assert!(r.iter().any(|(a, b)| *a <= 85.0 && *b >= 75.0), "{:?}", r);
        for (a, b) in &r { assert!(b - a >= 20.0 && b - a <= 40.0); }
    }
    #[test] fn eta() { assert!(estimativa(600.0, 5) > estimativa(60.0, 5)); }
}

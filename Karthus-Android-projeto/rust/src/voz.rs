//! Voz: ouvir (microfone + detecção de fala + Whisper) e falar (vozes do Windows).
//! O microfone só registra trechos que parecem fala humana; ao terminar uma frase (pausa), transcreve na hora.
use crate::carga;
use crate::estado::Estado;
use crate::util::*;
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const TAXA: u32 = 16000;
const QUADRO: usize = 320; // 20 ms a 16 kHz

// ------------------------------------------------------------------------------------------ detector de fala
/// Filtro biquad passa-faixa simples (RBJ).
#[derive(Clone)]
struct Biquad { b0: f32, b1: f32, b2: f32, a1: f32, a2: f32, x1: f32, x2: f32, y1: f32, y2: f32 }
impl Biquad {
    fn passa_faixa(f0: f32, q: f32, fs: f32) -> Biquad {
        let w = 2.0 * std::f32::consts::PI * f0 / fs; let (s, c) = (w.sin(), w.cos()); let al = s / (2.0 * q);
        let a0 = 1.0 + al;
        Biquad { b0: al / a0, b1: 0.0, b2: -al / a0, a1: -2.0 * c / a0, a2: (1.0 - al) / a0, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }
    fn p(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1; self.x1 = x; self.y2 = self.y1; self.y1 = y; y
    }
}

pub struct Detector {
    banda: Biquad,
    piso: f32,              // nível do ruído de fundo (aprendido)
    pre: std::collections::VecDeque<Vec<f32>>, // 300 ms antes do início
    seg: Vec<f32>,
    falando: bool,
    seguidos: u32,
    silencio: u32,
    rms_voz: Vec<f32>,
    quadros_seg: u32,
    pub nivel: f32,
    pub pausa_ms: u32,
}

impl Detector {
    pub fn novo() -> Detector {
        Detector { banda: Biquad::passa_faixa(1000.0, 0.45, TAXA as f32), piso: 0.004, pre: Default::default(), seg: vec![], falando: false, seguidos: 0, silencio: 0, rms_voz: vec![], quadros_seg: 0, nivel: 0.0, pausa_ms: 800 }
    }
    pub fn em_fala(&self) -> bool { self.falando }

    /// Processa 1 quadro de 20 ms. Devolve um trecho de fala completo quando a pessoa termina de falar.
    pub fn quadro(&mut self, q: &[f32]) -> Option<Vec<f32>> {
        let n = q.len().max(1) as f32;
        let rms = (q.iter().map(|x| x * x).sum::<f32>() / n).sqrt();
        let faixa = (q.iter().map(|&x| { let y = self.banda.p(x); y * y }).sum::<f32>() / n).sqrt();
        let razao = if rms > 1e-6 { faixa / rms } else { 0.0 };           // energia na faixa da voz (≈300–3400 Hz)
        let cruz = q.windows(2).filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0)).count() as f32 / n;
        self.nivel = self.nivel * 0.7 + rms * 0.3;
        let limiar = (self.piso * 3.2).max(0.012);
        let voz = rms > limiar && razao > 0.30 && cruz < 0.35;
        if !voz && !self.falando { self.piso = (self.piso * 0.97 + rms * 0.03).clamp(0.0005, 0.05); } // aprende o ruído
        if self.falando && !voz { self.piso = (self.piso * 0.995 + rms * 0.005).clamp(0.0005, 0.05); }

        if !self.falando {
            self.pre.push_back(q.to_vec()); if self.pre.len() > 15 { self.pre.pop_front(); }
            self.seguidos = if voz { self.seguidos + 1 } else { 0 };
            if self.seguidos >= 4 {
                self.falando = true; self.silencio = 0; self.quadros_seg = 0; self.rms_voz.clear();
                self.seg = self.pre.drain(..).flatten().collect();
                self.rms_voz.push(rms);
            }
            return None;
        }
        self.seg.extend_from_slice(q); self.quadros_seg += 1;
        if voz { self.silencio = 0; self.rms_voz.push(rms); } else { self.silencio += 1; }
        let maximo = self.quadros_seg > 25 * 50;
        if self.silencio * 20 >= self.pausa_ms || maximo {
            self.falando = false; self.seguidos = 0; self.pre.clear();
            let seg = std::mem::take(&mut self.seg);
            let rv = std::mem::take(&mut self.rms_voz);
            return if parece_fala(&rv, self.quadros_seg.saturating_sub(self.silencio)) { Some(seg) } else { None };
        }
        None
    }

    /// Fecha o trecho em andamento (ao parar de ouvir).
    pub fn fechar(&mut self) -> Option<Vec<f32>> {
        if !self.falando { return None; }
        self.falando = false;
        let seg = std::mem::take(&mut self.seg); let rv = std::mem::take(&mut self.rms_voz);
        if parece_fala(&rv, self.quadros_seg) { Some(seg) } else { None }
    }
}

/// Um trecho só vale se durou o bastante e o volume oscila como sílabas (um zumbido constante ou um clique não é fala).
fn parece_fala(rms_voz: &[f32], quadros: u32) -> bool {
    if quadros < 18 || rms_voz.len() < 12 { return false; }       // menos de ~0,35 s de voz
    let m = rms_voz.iter().sum::<f32>() / rms_voz.len() as f32;
    if m <= 0.0 { return false; }
    let v = rms_voz.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / rms_voz.len() as f32;
    let cv = v.sqrt() / m;
    let ocupacao = rms_voz.len() as f32 / quadros.max(1) as f32;
    cv > 0.10 && ocupacao > 0.35
}

pub fn escrever_wav(p: &std::path::Path, amostras: &[f32]) -> Result<(), String> {
    let spec = hound::WavSpec { channels: 1, sample_rate: TAXA, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
    let mut w = hound::WavWriter::create(p, spec).map_err(|e| e.to_string())?;
    for &s in amostras { w.write_sample((s.clamp(-1.0, 1.0) * 32767.0) as i16).map_err(|e| e.to_string())?; }
    w.finalize().map_err(|e| e.to_string())
}

// ------------------------------------------------------------------------------------------ Whisper
/// Frases fantasma que o Whisper inventa quando só há silêncio/ruído.
fn lixo_whisper(t: &str) -> bool {
    let l = t.to_lowercase();
    let l = l.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ').trim().to_string();
    l.is_empty() || ["blank_audio", "legendas pela comunidade", "amara.org", "obrigado por assistir", "inscreva-se", "música", "musica", "applause", "silence"].iter().any(|p| l.contains(p))
}

pub fn transcrever(est: &Estado, wav: &std::path::Path) -> Result<String, String> {
    let exe = carga::whisper_exe(est).ok_or("Falta o Whisper. Abra o ícone vermelho > Instalar.")?;
    let modelo = carga::whisper_modelo(est).ok_or("Falta o modelo do Whisper (ggml-*.bin).")?;
    let mut c = Command::new(&exe);
    c.arg("-m").arg(&modelo).arg("-f").arg(wav).args(["-l", "pt", "-nt", "-np", "-t"]).arg(crate::ia::threads_ideais().to_string())
        .current_dir(exe.parent().unwrap_or(std::path::Path::new("."))).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    sem_janela(&mut c);
    let o = c.output().map_err(|e| format!("Não consegui rodar o Whisper: {}", e))?;
    let t = String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ");
    Ok(if lixo_whisper(&t) { String::new() } else { t })
}

// ------------------------------------------------------------------------------------------ estado da voz
#[derive(Default)]
pub struct Interno { ouvindo: bool, nivel: f32, fala: bool, pendentes: u32, novos: Vec<String>, todos: Vec<String>, erro: String }

pub struct Voz { pub i: Arc<Mutex<Interno>>, parar: Arc<AtomicBool>, falando: Arc<AtomicBool>, filho: Arc<Mutex<Option<Child>>>, vozes_cache: Mutex<Option<Vec<Value>>> }

impl Voz {
    pub fn novo() -> Voz { Voz { i: Arc::new(Mutex::new(Interno::default())), parar: Arc::new(AtomicBool::new(true)), falando: Arc::new(AtomicBool::new(false)), filho: Arc::new(Mutex::new(None)), vozes_cache: Mutex::new(None) } }
    pub fn ia_falando(&self) -> bool { self.falando.load(Ordering::SeqCst) }

    pub fn estado(&self) -> Value {
        let mut i = self.i.lock().unwrap_or_else(|e| e.into_inner());
        let novos = std::mem::take(&mut i.novos);
        json!({"ouvindo": i.ouvindo, "nivel": (i.nivel * 12.0).min(1.0), "fala": i.fala, "pendentes": i.pendentes, "textos": novos, "erro": i.erro, "ia_falando": self.ia_falando()})
    }

    /// Começa a ouvir (contínuo, com detecção de fala). Cada frase vira texto assim que a pessoa faz uma pausa.
    pub fn gravar(&self, est: &Arc<Estado>) -> Result<(), String> {
        if carga::whisper_exe(est).is_none() || carga::whisper_modelo(est).is_none() { return Err("Falta o Whisper para ouvir. Clique no ícone vermelho e em Instalar (voz: ouvir).".into()); }
        if self.i.lock().map(|i| i.ouvindo).unwrap_or(false) { return Ok(()); }
        { let mut i = self.i.lock().unwrap_or_else(|e| e.into_inner()); *i = Interno { ouvindo: true, ..Default::default() }; }
        self.parar.store(false, Ordering::SeqCst);
        let (est2, i2, parar, falando) = (est.clone(), self.i.clone(), self.parar.clone(), self.falando.clone());
        let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();
        // captura
        let captura = iniciar_captura(tx, self.parar.clone());
        if let Err(e) = captura { self.parar.store(true, Ordering::SeqCst); if let Ok(mut i) = self.i.lock() { i.ouvindo = false; } return Err(e); }
        std::thread::spawn(move || {
            let mut det = Detector::novo();
            let mut resto: Vec<f32> = Vec::new();
            let enviar = |seg: Vec<f32>| {
                let (e, i) = (est2.clone(), i2.clone());
                if let Ok(mut x) = i.lock() { x.pendentes += 1; }
                std::thread::spawn(move || {
                    let p = e.dados.join("tmp").join(format!("{}.wav", novo_id("v")));
                    let r = escrever_wav(&p, &seg).and_then(|_| transcrever(&e, &p));
                    let _ = std::fs::remove_file(&p);
                    if let Ok(mut x) = i.lock() {
                        x.pendentes = x.pendentes.saturating_sub(1);
                        match r { Ok(t) if !t.is_empty() => { x.novos.push(t.clone()); x.todos.push(t); } Ok(_) => {} Err(er) => x.erro = er }
                    }
                });
            };
            loop {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(bloco) => {
                        if falando.load(Ordering::SeqCst) { resto.clear(); continue; } // não escuta a própria voz
                        resto.extend_from_slice(&bloco);
                        while resto.len() >= QUADRO {
                            let q: Vec<f32> = resto.drain(..QUADRO).collect();
                            if let Some(seg) = det.quadro(&q) { enviar(seg); }
                        }
                        if let Ok(mut x) = i2.lock() { x.nivel = det.nivel; x.fala = det.em_fala(); }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(_) => break,
                }
                if parar.load(Ordering::SeqCst) { break; }
            }
            if let Some(seg) = det.fechar() { enviar(seg); }
            if let Ok(mut x) = i2.lock() { x.ouvindo = false; x.fala = false; x.nivel = 0.0; }
        });
        Ok(())
    }

    /// Para de ouvir, espera as transcrições terminarem e devolve o que ainda não foi lido pela tela.
    pub fn parar_e_ler(&self) -> String {
        self.parar.store(true, Ordering::SeqCst);
        let t0 = Instant::now();
        loop {
            let (o, p) = { let i = self.i.lock().unwrap_or_else(|e| e.into_inner()); (i.ouvindo, i.pendentes) };
            if (!o && p == 0) || t0.elapsed() > Duration::from_secs(40) { break; }
            std::thread::sleep(Duration::from_millis(80));
        }
        let mut i = self.i.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut i.novos).join(" ")
    }

    // ---- falar
    pub fn falar(&self, texto: &str, voz: Option<&str>) -> Result<(), String> {
        self.calar();
        let texto: String = texto.chars().take(6000).collect();
        if texto.trim().is_empty() { return Ok(()); }
        if std::env::var("HIRELOD_VOZ_FALSA").is_ok() {
            self.falando.store(true, Ordering::SeqCst);
            let f = self.falando.clone(); std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(400)); f.store(false, Ordering::SeqCst); });
            return Ok(());
        }
        #[cfg(not(windows))]
        { let _ = voz; return Err("Falar em voz alta só funciona no Windows.".into()); }
        #[cfg(windows)]
        {
            let vs = self.vozes();
            if vs.is_empty() { return Err("Nenhuma voz do Windows foi encontrada. Instale em Configurações > Hora e idioma > Fala.".into()); }
            let alvo = voz.unwrap_or("").to_lowercase();
            let nome = vs.iter().find(|v| !alvo.is_empty() && v["nome"].as_str().unwrap_or("").to_lowercase() == alvo)
                .or_else(|| vs.iter().find(|v| v["idioma"].as_str().unwrap_or("").to_lowercase().starts_with("pt-br")))
                .or_else(|| vs.iter().find(|v| v["idioma"].as_str().unwrap_or("").to_lowercase().starts_with("pt")))
                .and_then(|v| v["nome"].as_str().map(|s| s.to_string()));
            let sel = nome.map(|n| format!("$s.SelectVoice('{}');", n.replace('\'', ""))).unwrap_or_default();
            let script = format!("Add-Type -AssemblyName System.Speech; $s=New-Object System.Speech.Synthesis.SpeechSynthesizer; {} $s.Rate=1; $s.Speak($env:HIRELOD_TEXTO)", sel);
            let mut c = Command::new("powershell.exe");
            c.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script]).env("HIRELOD_TEXTO", &texto).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
            sem_janela(&mut c);
            let filho = c.spawn().map_err(|e| format!("Não consegui abrir a voz do Windows: {}", e))?;
            if let Ok(mut f) = self.filho.lock() { *f = Some(filho); }
            self.falando.store(true, Ordering::SeqCst);
            let (f2, fh) = (self.falando.clone(), self.filho.clone());
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(150));
                let acabou = fh.lock().map(|mut f| match f.as_mut() { Some(c) => c.try_wait().map(|s| s.is_some()).unwrap_or(true), None => true }).unwrap_or(true);
                if acabou { f2.store(false, Ordering::SeqCst); break; }
            });
            Ok(())
        }
    }
    /// Fala com uma voz neural do Piper (offline): gera um .wav e toca. Para na hora se chamarem calar().
    pub fn falar_piper(&self, exe: std::path::PathBuf, modelo: std::path::PathBuf, texto: &str, tmp: std::path::PathBuf) -> Result<(), String> {
        self.calar();
        let texto: String = texto.chars().take(6000).collect();
        if texto.trim().is_empty() { return Ok(()); }
        if std::env::var("HIRELOD_VOZ_FALSA").is_ok() { return self.falar(&texto, None); }
        let _ = std::fs::create_dir_all(&tmp);
        let wav = tmp.join(format!("{}.wav", novo_id("fala")));
        let logp = tmp.join("piper.log");
        let mut c = Command::new(&exe);
        c.arg("--model").arg(&modelo).arg("--output_file").arg(&wav);
        if let Some(d) = exe.parent() {
            c.current_dir(d);
            let ed = d.join("espeak-ng-data");
            if ed.is_dir() { c.arg("--espeak_data").arg(ed); }
        }
        let erro = std::fs::File::create(&logp).map(Stdio::from).unwrap_or_else(|_| Stdio::null());
        c.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(erro);
        sem_janela(&mut c);
        let mut filho = c.spawn().map_err(|e| format!("Não consegui abrir o Piper ({}): {}", exe.display(), e))?;
        if let Some(mut si) = filho.stdin.take() { use std::io::Write; let _ = si.write_all(texto.replace('\n', " ").as_bytes()); let _ = si.write_all(b"\n"); }
        if let Ok(mut f) = self.filho.lock() { *f = Some(filho); }
        self.falando.store(true, Ordering::SeqCst);
        // gera a fala aqui (assim qualquer erro do Piper aparece para o senhor)
        let ini = std::time::Instant::now();
        let status = loop {
            std::thread::sleep(Duration::from_millis(60));
            if !self.falando.load(Ordering::SeqCst) { let _ = std::fs::remove_file(&wav); return Ok(()); }
            let r = self.filho.lock().ok().and_then(|mut f| f.as_mut().map(|c| c.try_wait()));
            match r { Some(Ok(Some(st))) => break st, Some(Ok(None)) => {}, _ => break std::process::ExitStatus::default() }
            if ini.elapsed() > Duration::from_secs(90) { self.calar(); return Err("O Piper demorou demais para gerar a fala.".into()); }
        };
        let tam = std::fs::metadata(&wav).map(|m| m.len()).unwrap_or(0);
        if !status.success() || tam < 100 {
            self.falando.store(false, Ordering::SeqCst);
            let _ = std::fs::remove_file(&wav);
            let l = std::fs::read_to_string(&logp).unwrap_or_default();
            let ult = l.lines().rev().find(|x| !x.trim().is_empty()).unwrap_or("sem mensagem").chars().take(200).collect::<String>();
            return Err(format!("O Piper não gerou a voz (código {}): {}", status.code().map(|c| c.to_string()).unwrap_or("?".into()), ult));
        }
        let (falando, fh) = (self.falando.clone(), self.filho.clone());
        std::thread::spawn(move || {
            #[cfg(windows)]
            {
                let mut p = Command::new("powershell.exe");
                p.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", "$p=New-Object Media.SoundPlayer $env:HIRELOD_WAV; $p.PlaySync()"]).env("HIRELOD_WAV", &wav).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
                sem_janela(&mut p);
                if let Ok(ch) = p.spawn() {
                    if let Ok(mut f) = fh.lock() { *f = Some(ch); }
                    loop {
                        std::thread::sleep(Duration::from_millis(100));
                        if !falando.load(Ordering::SeqCst) { break; }
                        let fim = fh.lock().map(|mut f| match f.as_mut() { Some(c) => c.try_wait().map(|s| s.is_some()).unwrap_or(true), None => true }).unwrap_or(true);
                        if fim { break; }
                    }
                }
            }
            #[cfg(not(windows))]
            { let _ = &fh; }
            let _ = std::fs::remove_file(&wav);
            falando.store(false, Ordering::SeqCst);
        });
        Ok(())
    }
    pub fn calar(&self) {
        if let Ok(mut f) = self.filho.lock() { if let Some(mut c) = f.take() { let _ = c.kill(); let _ = c.wait(); } }
        self.falando.store(false, Ordering::SeqCst);
    }
    pub fn vozes(&self) -> Vec<Value> {
        if let Ok(c) = self.vozes_cache.lock() { if let Some(v) = c.as_ref() { return v.clone(); } }
        let v: Vec<Value> = listar_vozes().into_iter().filter(|x| x["idioma"].as_str().unwrap_or("").to_lowercase().starts_with("pt")).collect();
        if let Ok(mut c) = self.vozes_cache.lock() { if !v.is_empty() { *c = Some(v.clone()); } }
        v
    }
}

/// Vozes instaladas (SAPI do Windows, via PowerShell).
fn listar_vozes() -> Vec<Value> {
    if std::env::var("HIRELOD_VOZ_FALSA").is_ok() { return vec![json!({"nome": "Voz de teste", "idioma": "pt-BR"})]; }
    #[cfg(windows)]
    {
        let mut c = Command::new("powershell.exe");
        c.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", "Add-Type -AssemblyName System.Speech; $s=New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.GetInstalledVoices() | Where-Object {$_.Enabled} | ForEach-Object { $_.VoiceInfo.Name + '|' + $_.VoiceInfo.Culture.Name }"]).stdin(Stdio::null());
        sem_janela(&mut c);
        if let Ok(o) = c.output() { return String::from_utf8_lossy(&o.stdout).lines().filter_map(|l| { let (n, i) = l.trim().split_once('|')?; Some(json!({"nome": n, "idioma": i})) }).collect(); }
    }
    vec![]
}
pub fn vozes() -> Vec<Value> { listar_vozes() }

// ------------------------------------------------------------------------------------------ captura do microfone
#[cfg(windows)]
fn iniciar_captura(tx: std::sync::mpsc::Sender<Vec<f32>>, parar: Arc<AtomicBool>) -> Result<(), String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let (ok_tx, ok_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let Some(dev) = host.default_input_device() else { let _ = ok_tx.send(Err("Não achei nenhum microfone. Conecte um e confira a permissão de microfone do Windows.".into())); return; };
        let cfg = match dev.default_input_config() { Ok(c) => c, Err(e) => { let _ = ok_tx.send(Err(format!("Microfone indisponível: {}", e))); return; } };
        let canais = cfg.channels() as usize; let taxa = cfg.sample_rate().0 as f32;
        let passo = taxa / TAXA as f32;
        let mut frac = 0.0f32;
        let mut ant = 0.0f32;
        let mut processa = move |mono: &[f32], tx: &std::sync::mpsc::Sender<Vec<f32>>| {
            // reamostragem linear para 16 kHz
            let mut out = Vec::with_capacity((mono.len() as f32 / passo) as usize + 2);
            for &x in mono { while frac < 1.0 { out.push(ant + (x - ant) * frac); frac += passo; } frac -= 1.0; ant = x; }
            if !out.is_empty() { let _ = tx.send(out); }
        };
        let tx2 = tx.clone();
        let erro = |e| eprintln!("microfone: {}", e);
        let stream = match cfg.sample_format() {
            cpal::SampleFormat::F32 => dev.build_input_stream(&cfg.clone().into(), move |d: &[f32], _| { let m: Vec<f32> = d.chunks(canais).map(|c| c.iter().sum::<f32>() / canais as f32).collect(); processa(&m, &tx2); }, erro, None),
            cpal::SampleFormat::I16 => dev.build_input_stream(&cfg.clone().into(), move |d: &[i16], _| { let m: Vec<f32> = d.chunks(canais).map(|c| c.iter().map(|&s| s as f32 / 32768.0).sum::<f32>() / canais as f32).collect(); processa(&m, &tx2); }, erro, None),
            cpal::SampleFormat::U16 => dev.build_input_stream(&cfg.clone().into(), move |d: &[u16], _| { let m: Vec<f32> = d.chunks(canais).map(|c| c.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).sum::<f32>() / canais as f32).collect(); processa(&m, &tx2); }, erro, None),
            f => { let _ = ok_tx.send(Err(format!("Formato de microfone não suportado: {:?}", f))); return; }
        };
        match stream { Ok(s) => { if let Err(e) = s.play() { let _ = ok_tx.send(Err(format!("Não consegui ligar o microfone: {}", e))); return; } let _ = ok_tx.send(Ok(())); while !parar.load(Ordering::SeqCst) { std::thread::sleep(Duration::from_millis(100)); } drop(s); }
                       Err(e) => { let _ = ok_tx.send(Err(format!("Não consegui abrir o microfone: {}", e))); } }
    });
    ok_rx.recv_timeout(Duration::from_secs(8)).map_err(|_| "O microfone demorou demais para responder.".to_string())?
}

/// Fora do Windows (testes): lê um arquivo WAV em tempo real, como se fosse o microfone.
#[cfg(not(windows))]
fn iniciar_captura(tx: std::sync::mpsc::Sender<Vec<f32>>, parar: Arc<AtomicBool>) -> Result<(), String> {
    let p = std::env::var("HIRELOD_VOZ_ARQUIVO").map_err(|_| "O microfone só funciona no Windows.".to_string())?;
    let mut r = hound::WavReader::open(&p).map_err(|e| format!("WAV de teste: {}", e))?;
    let amostras: Vec<f32> = r.samples::<i16>().filter_map(|s| s.ok()).map(|s| s as f32 / 32768.0).collect();
    std::thread::spawn(move || {
        for bloco in amostras.chunks(1600) {
            if parar.load(Ordering::SeqCst) { return; }
            if tx.send(bloco.to_vec()).is_err() { return; }
            std::thread::sleep(Duration::from_millis(20));
        }
        while !parar.load(Ordering::SeqCst) { std::thread::sleep(Duration::from_millis(50)); }
    });
    Ok(())
}

#[cfg(test)]
pub mod testes {
    use super::*;
    /// Sinal parecido com fala: harmônicas de 120 Hz com envelope silábico (~4 Hz) + ruído.
    pub fn fala(seg: f32, amp: f32, semente: u32) -> Vec<f32> {
        let n = (seg * TAXA as f32) as usize; let mut s = semente.max(1);
        (0..n).map(|i| {
            let t = i as f32 / TAXA as f32;
            let env = 0.55 + 0.45 * (2.0 * std::f32::consts::PI * 4.0 * t).sin().abs();
            let mut v = 0.0; for h in 1..=14 { let f = 120.0 * h as f32; let formante = (-((f - 700.0) / 500.0).powi(2)).exp() + 0.6 * (-((f - 1800.0) / 700.0).powi(2)).exp(); v += formante * (2.0 * std::f32::consts::PI * f * t).sin() / h as f32; }
            s = s.wrapping_mul(1664525).wrapping_add(1013904223); let r = ((s >> 9) as f32 / 4194304.0 - 1.0) * 0.004;
            v * env * amp + r
        }).collect()
    }
    pub fn ruido(seg: f32, amp: f32) -> Vec<f32> { let n = (seg * TAXA as f32) as usize; let mut s = 7u32; (0..n).map(|_| { s = s.wrapping_mul(1664525).wrapping_add(1013904223); ((s >> 9) as f32 / 4194304.0 - 1.0) * amp }).collect() }
    fn rodar(sinal: &[f32]) -> Vec<usize> {
        let mut d = Detector::novo(); let mut out = vec![];
        for q in sinal.chunks(QUADRO) { if q.len() == QUADRO { if let Some(s) = d.quadro(q) { out.push(s.len()); } } }
        if let Some(s) = d.fechar() { out.push(s.len()); }
        out
    }
    #[test] fn so_ruido_nao_grava() { assert!(rodar(&ruido(5.0, 0.01)).is_empty()); }
    #[test] fn duas_frases_separadas_por_pausa() {
        let mut s = ruido(1.0, 0.004); s.extend(fala(2.0, 0.25, 1)); s.extend(ruido(1.5, 0.004)); s.extend(fala(1.5, 0.25, 2)); s.extend(ruido(1.5, 0.004));
        let r = rodar(&s); assert_eq!(r.len(), 2, "trechos: {:?}", r);
        assert!(r[0] as f32 / TAXA as f32 > 1.8 && (r[0] as f32 / TAXA as f32) < 3.5);
    }
    #[test] fn zumbido_constante_nao_e_fala() {
        let s: Vec<f32> = (0..(4 * TAXA as usize)).map(|i| 0.2 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / TAXA as f32).sin()).collect();
        assert!(rodar(&s).is_empty());
    }
    #[test] fn clique_curto_nao_e_fala() { let mut s = ruido(1.0, 0.004); s.extend(fala(0.12, 0.3, 3)); s.extend(ruido(1.5, 0.004)); assert!(rodar(&s).is_empty()); }
    #[test] fn pausa_curta_no_meio_nao_corta() {
        let mut s = ruido(0.5, 0.004); s.extend(fala(1.2, 0.25, 4)); s.extend(ruido(0.4, 0.004)); s.extend(fala(1.2, 0.25, 5)); s.extend(ruido(1.5, 0.004));
        assert_eq!(rodar(&s).len(), 1);
    }
    #[test] fn filtra_lixo() { assert!(lixo_whisper("[BLANK_AUDIO]")); assert!(lixo_whisper(" Legendas pela comunidade Amara.org")); assert!(!lixo_whisper("Oi, tudo bem?")); }
}

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod atualizar; mod carga; mod catalogo; mod cortes; mod criar; mod downloads; mod estado; mod hf; mod ia; mod llama; mod modelos_locais;
mod motores_web; mod orquestra; mod recursos; mod server; mod usuarios; mod util; mod voz; mod web;

use std::sync::atomic::Ordering;
use std::sync::{Arc, OnceLock};

pub static GLOBAL: OnceLock<Arc<estado::Estado>> = OnceLock::new();

fn porta_do_servidor() -> u16 { std::env::var("HIRELOD_PORTA").ok().and_then(|p| p.parse().ok()).unwrap_or(8765) }

fn main() {
    let base = util::base_dir();
    let _ = std::fs::create_dir_all(&base);
    atualizar::limpar_velhos(&base);
    let est = Arc::new(estado::Estado::novo(base.clone(), porta_do_servidor()));
    let _ = GLOBAL.set(est.clone());
    if let Err(e) = server::iniciar(est.clone()) {
        util::log(&base, &format!("servidor: {}", e));
        #[cfg(windows)] { janela::aviso("Karthus", &format!("{}\n\nO Karthus já está aberto? Feche-o e tente de novo.", e)); }
        eprintln!("{}", e);
        return;
    }
    // Motor de IA + verificação em segundo plano (a tela de carregamento mostra o andamento)
    {
        let e0 = est.clone();
        std::thread::spawn(move || { let _ = modelos_locais::atualizar_catalogo(&e0); });
        let e = est.clone();
        std::thread::spawn(move || {
            let r = e.ollama.iniciar(&e);
            if let Err(er) = &r { util::log(&e.base, &format!("motor: {}", er)); }
            carga::verificar(&e);
        });
    }
    #[cfg(windows)] { janela::preparar_webview(&est); janela::rodar(est.clone()); }
    #[cfg(not(windows))] {
        println!("Karthus {} em http://127.0.0.1:{}", util::VERSAO, est.porta);
        while !est.sair.load(Ordering::SeqCst) { std::thread::sleep(std::time::Duration::from_millis(200)); }
    }
    est.ollama.parar();
    if est.reiniciar.load(Ordering::SeqCst) {
        if let Ok(exe) = std::env::current_exe() { let mut c = std::process::Command::new(exe); util::sem_janela(&mut c); let _ = c.spawn(); }
    }
}

#[cfg(windows)]
mod janela {
    use super::*;
    use tao::dpi::LogicalSize;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::window::{Icon, ResizeDirection, WindowBuilder};
    use wry::{WebContext, WebViewBuilder};

    #[derive(Debug, Clone)]
    enum Ev { Arrastar, Minimizar, Maximizar, Fechar, Borda(ResizeDirection) }

    static LOADER: &[u8] = include_bytes!("../embutido/WebView2Loader.dll");
    extern "C" { static mut karthus_wv2_path: [u16; 600]; }
    /// A WebView2Loader.dll vai dentro do .exe: extrai (uma vez) para dados/bin e diz ao carregador onde ela está.
    pub fn preparar_webview(est: &estado::Estado) {
        let dst = est.dados.join("bin").join("WebView2Loader.dll");
        let ok = std::fs::read(&dst).map(|b| b == LOADER).unwrap_or(false);
        if !ok { if let Some(p) = dst.parent() { let _ = std::fs::create_dir_all(p); } let _ = util::gravar_atomico(&dst, LOADER); }
        let w: Vec<u16> = dst.to_string_lossy().encode_utf16().collect();
        if w.len() < 599 { unsafe { let p = std::ptr::addr_of_mut!(karthus_wv2_path) as *mut u16; for (i, c) in w.iter().enumerate() { *p.add(i) = *c; } *p.add(w.len()) = 0; } }
    }
    pub fn aviso(t: &str, m: &str) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
        let w = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        unsafe { MessageBoxW(std::ptr::null_mut(), w(m).as_ptr(), w(t).as_ptr(), MB_OK | MB_ICONERROR); }
    }

    fn icone() -> Option<Icon> {
        let dec = png::Decoder::new(std::io::Cursor::new(include_bytes!("../assets/espiral.png").as_slice()));
        let mut r = dec.read_info().ok()?;
        let mut buf = vec![0; r.output_buffer_size()];
        let i = r.next_frame(&mut buf).ok()?;
        let rgba: Vec<u8> = match i.color_type {
            png::ColorType::Rgba => buf[..i.buffer_size()].to_vec(),
            png::ColorType::Rgb => buf[..i.buffer_size()].chunks(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
            _ => return None };
        Icon::from_rgba(rgba, i.width, i.height).ok()
    }

    fn direcao(s: &str) -> Option<ResizeDirection> {
        Some(match s { "n" => ResizeDirection::North, "s" => ResizeDirection::South, "e" => ResizeDirection::East, "w" => ResizeDirection::West,
            "ne" => ResizeDirection::NorthEast, "nw" => ResizeDirection::NorthWest, "se" => ResizeDirection::SouthEast, "sw" => ResizeDirection::SouthWest, _ => return None })
    }

    pub fn rodar(est: Arc<estado::Estado>) {
        let el = EventLoopBuilder::<Ev>::with_user_event().build();
        let proxy = el.create_proxy();
        // Sem barra do sistema: a interface desenha a própria barra (arrastar, minimizar, maximizar, fechar).
        let mut wb = WindowBuilder::new().with_title("Karthus").with_inner_size(LogicalSize::new(1180.0, 760.0)).with_min_inner_size(LogicalSize::new(760.0, 520.0)).with_decorations(false);
        if let Some(i) = icone() { wb = wb.with_window_icon(Some(i)); }
        let janela = match wb.build(&el) { Ok(w) => w, Err(e) => { aviso("Karthus", &format!("Não consegui abrir a janela: {}", e)); return; } };
        let mut ctx = WebContext::new(Some(est.dados.join("webview")));
        let url = format!("http://127.0.0.1:{}/carregando", est.porta);
        let p2 = proxy.clone();
        let wv = WebViewBuilder::with_web_context(&mut ctx).with_background_color((13, 13, 13, 255)).with_url(&url).with_ipc_handler(move |req| {
            let b = req.body().as_str();
            let ev = match b { "drag" => Ev::Arrastar, "min" => Ev::Minimizar, "max" => Ev::Maximizar, "close" => Ev::Fechar, x if x.starts_with("resize:") => match direcao(&x[7..]) { Some(d) => Ev::Borda(d), None => return }, _ => return };
            let _ = p2.send_event(ev);
        }).build(&janela);
        let wv = match wv { Ok(w) => w, Err(e) => { aviso("Karthus", &format!("Não consegui iniciar o WebView2 (o componente de tela do Windows): {}", e)); return; } };
        let _ = wv;
        el.run(move |ev, _, fluxo| {
            *fluxo = ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_millis(250));
            if est.sair.load(Ordering::SeqCst) || est.reiniciar.load(Ordering::SeqCst) && false { *fluxo = ControlFlow::Exit; }
            match ev {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => { *fluxo = ControlFlow::Exit; }
                Event::UserEvent(Ev::Arrastar) => { let _ = janela.drag_window(); }
                Event::UserEvent(Ev::Minimizar) => janela.set_minimized(true),
                Event::UserEvent(Ev::Maximizar) => janela.set_maximized(!janela.is_maximized()),
                Event::UserEvent(Ev::Fechar) => { *fluxo = ControlFlow::Exit; }
                Event::UserEvent(Ev::Borda(d)) => { let _ = janela.drag_resize_window(d); }
                _ => {}
            }
        });
    }
}

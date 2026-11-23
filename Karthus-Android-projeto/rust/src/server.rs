//! Servidor local (127.0.0.1) que entrega a interface e a API.
use crate::estado::Estado;
use crate::util::*;
use crate::{atualizar, carga, catalogo, cortes, criar, downloads, modelos_locais, orquestra, recursos};
use include_dir::{include_dir, Dir};
use serde_json::{json, Value};
use std::io::{Cursor, Read};
use std::sync::Arc;
use tiny_http::{Header, Method, Request, Response, Server};

static UI: Dir = include_dir!("$CARGO_MANIFEST_DIR/ui");

fn h(k: &str, v: &str) -> Header { Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap() }
fn resp_json(rq: Request, code: u16, v: &Value) {
    let _ = rq.respond(Response::from_data(v.to_string().into_bytes()).with_status_code(code).with_header(h("Content-Type", "application/json; charset=utf-8")).with_header(h("Cache-Control", "no-store")));
}
fn erro(rq: Request, code: u16, m: &str) { resp_json(rq, code, &json!({"erro": m})); }
fn tipo_de(n: &str) -> &'static str {
    match n.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "html" => "text/html; charset=utf-8", "js" => "text/javascript; charset=utf-8", "css" => "text/css; charset=utf-8", "json" => "application/json",
        "png" => "image/png", "jpg" | "jpeg" => "image/jpeg", "gif" => "image/gif", "svg" => "image/svg+xml", "ico" => "image/x-icon", "webp" => "image/webp",
        "mp4" => "video/mp4", "webm" => "video/webm", "mp3" => "audio/mpeg", "wav" => "audio/wav", "txt" | "md" | "csv" | "py" | "rs" | "sql" | "yaml" | "yml" | "xml" | "ts" | "c" | "cpp" | "cs" | "go" | "java" | "toml" => "text/plain; charset=utf-8",
        _ => "application/octet-stream" }
}
fn query(u: &str, k: &str) -> Option<String> {
    let q = u.split_once('?')?.1;
    q.split('&').find_map(|p| { let (a, b) = p.split_once('=')?; if a == k { Some(decodificar(b)) } else { None } })
}
fn decodificar(s: &str) -> String {
    let b = s.as_bytes(); let mut o = Vec::new(); let mut i = 0;
    while i < b.len() { if b[i] == b'%' && i + 2 < b.len() + 0 && i + 2 <= b.len() - 1 { if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) { o.push(v); i += 3; continue; } } o.push(if b[i] == b'+' { b' ' } else { b[i] }); i += 1; }
    String::from_utf8_lossy(&o).to_string()
}
fn corpo(rq: &mut Request, max: usize) -> Result<Vec<u8>, String> {
    let mut v = Vec::new();
    rq.as_reader().take(max as u64 + 1).read_to_end(&mut v).map_err(|e| e.to_string())?;
    if v.len() > max { return Err("Arquivo grande demais.".into()); }
    Ok(v)
}
fn corpo_json(rq: &mut Request) -> Value { corpo(rq, 4_000_000).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(json!({})) }
fn cab(rq: &Request, n: &str) -> String { rq.headers().iter().find(|x| x.field.as_str().as_str().eq_ignore_ascii_case(n)).map(|x| x.value.as_str().to_string()).unwrap_or_default() }

fn servir_ui(rq: Request, caminho: &str, est: &Estado) {
    let n = caminho.trim_start_matches('/');
    let n = if n.is_empty() { "index.html" } else if n == "carregando" { "carregando.html" } else { n };
    if n.contains("..") || n.contains('\\') { return erro(rq, 404, "não encontrado"); }
    let disco = est.base.join("ui").join(n);
    let dados: Option<Vec<u8>> = if disco.is_file() && dentro_de(&est.base.join("ui"), &disco) { std::fs::read(&disco).ok() } else { UI.get_file(n).map(|f| f.contents().to_vec()) };
    match dados {
        Some(d) => { let _ = rq.respond(Response::from_data(d).with_header(h("Content-Type", tipo_de(n))).with_header(h("Cache-Control", "no-store"))); }
        None => erro(rq, 404, "não encontrado"),
    }
}

/// Ajusta o pedido de chat para responder rápido: sem pensar por padrão, contexto pequeno, limite de resposta, modelo aquecido.
pub fn otimizar(c: &mut Value) {
    let msgs: Vec<Value> = c["messages"].as_array().cloned().unwrap_or_default();
    let ultimo = msgs.iter().rev().find(|m| m["role"] == "user").and_then(|m| m["content"].as_str()).unwrap_or("").trim().to_lowercase();
    let saudacao = ultimo.chars().count() <= 24 && ["oi", "olá", "ola", "bom dia", "boa tarde", "boa noite", "e aí", "e ai", "opa", "hey", "hello", "tudo bem", "obrigado", "valeu"].iter().any(|g| ultimo.starts_with(g));
    let pensar = c["think"] == json!(true);
    if saudacao && !pensar {
        // cumprimento: só o essencial (persona + última mensagem), resposta curtíssima
        let mut m2: Vec<Value> = msgs.iter().filter(|m| m["role"] == "system").take(1).cloned().collect();
        if let Some(s) = m2.first_mut() { let t: String = s["content"].as_str().unwrap_or("").chars().take(500).collect(); s["content"] = json!(t); }
        if let Some(u) = msgs.iter().rev().find(|m| m["role"] == "user") { m2.push(u.clone()); }
        c["messages"] = json!(m2);
    }
    let total: usize = c["messages"].as_array().map(|a| a.iter().map(|m| m["content"].as_str().map(|s| s.chars().count()).unwrap_or(0)).sum()).unwrap_or(0);
    let precisa = total / 3 + if pensar { 3000 } else { 900 };
    let ctx = if precisa <= 1024 { 1024 } else if precisa <= 2048 { 2048 } else if precisa <= 4096 { 4096 } else if precisa <= 8192 { 8192 } else { 16384 };
    if !c["options"].is_object() { c["options"] = json!({}); }
    let o = &mut c["options"];
    if o["num_ctx"].is_null() { o["num_ctx"] = json!(ctx); }
    if o["num_predict"].is_null() { o["num_predict"] = json!(if saudacao { 90 } else if pensar { 3000 } else { 1200 }); }
    if o["temperature"].is_null() { o["temperature"] = json!(if saudacao { 0.5 } else { 0.7 }); }
    if o["repeat_penalty"].is_null() { o["repeat_penalty"] = json!(1.1); }
    c["think"] = json!(pensar);
    c["keep_alive"] = json!(7200);
    c["stream"] = json!(true);
}

fn pronto_texto(rq: Request, texto: &str, tarefa: Option<(&str, &str)>) {
    let mut v = json!({"message": {"role": "assistant", "content": texto}, "done": false});
    if let Some((job, tipo)) = tarefa { v["tarefa"] = json!(job); v["tipo"] = json!(tipo); }
    let mut s = v.to_string(); s.push('\n');
    s.push_str(&json!({"done": true}).to_string()); s.push('\n');
    enviar_fluxo(rq, "application/x-ndjson", Box::new(Cursor::new(s.into_bytes())));
}

fn chat(est: &Arc<Estado>, mut rq: Request, uid: &str) {
    let mut c = corpo_json(&mut rq);
    if c["messages"].as_array().map(|a| a.is_empty()).unwrap_or(true) { return erro(rq, 400, "Mensagem vazia."); }
    let ultimo = c["messages"].as_array().and_then(|a| a.iter().rev().find(|m| m["role"] == "user")).and_then(|m| m["content"].as_str()).unwrap_or("").to_string();
    let mut modelo = c["model"].as_str().unwrap_or("").to_string();
    if modelo.is_empty() { modelo = est.modelo_padrao(); c["model"] = json!(modelo); }
    if modelo.is_empty() { return erro(rq, 400, "Nenhuma IA de texto instalada. Abra o ícone vermelho para instalar."); }
    // Pedido de criação? Vira tarefa em segundo plano na aba Criações.
    if let Some((tipo, assunto)) = criar::intencao(&ultimo) {
        let r = match tipo {
            "imagem" => criar::imagem(est, uid, &assunto, 512, 512, 20),
            "video" => criar::video(est, uid, &assunto, 4, 3, 512, 512),
            "arquivo" => criar::arquivo(est, uid, &assunto, &modelo),
            _ => criar::baixar_arquivo(est, uid, &assunto),
        };
        let (t, tf) = match &r {
            Ok(j) => (match tipo { "imagem" => "Sim, senhor. Estou criando a imagem em segundo plano. Aparece aqui quando ficar pronta (e na aba Criações). Pode continuar conversando.", "video" => "Sim, senhor. Estou criando o vídeo em segundo plano. Aparece aqui quando ficar pronto (e na aba Criações). Pode continuar conversando.", "arquivo" => "Sim, senhor. Estou escrevendo o arquivo em segundo plano. Já entrego aqui para baixar.", _ => "Sim, senhor. Baixando em segundo plano; já entrego aqui." }.to_string(), Some(j.clone())),
            Err(e) => (format!("Não consegui iniciar, senhor: {} Posso tentar novamente quando quiser.", e), None) };
        return pronto_texto(rq, &t, tf.as_deref().map(|j| (j, tipo)));
    }
    // identidade: o nome da IA é Karthus (qualquer modelo por baixo)
    {
        let persona = est.repo.config(uid)["persona"].as_str().unwrap_or("").trim().chars().take(600).collect::<String>();
        let mut base = "Você é o Karthus, um assistente de IA pessoal que roda 100% no computador do usuário. Seu nome é Karthus (nunca diga que é Gemma, Llama, Qwen, Karthus ou outro nome). Responda em português do Brasil, direto e claro, e trate o usuário por 'senhor'.".to_string();
        if !persona.is_empty() { base.push_str(&format!(" Instruções do usuário: {}", persona)); }
        if let Some(a) = c["messages"].as_array_mut() { if !a.iter().any(|m| m["role"] == "system") { a.insert(0, json!({"role": "system", "content": base})); } }
    }
    let web = c["web"] == json!(true);
    let fluxo: Vec<String> = c["fluxo"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let recursos = recursos::fala_de_recursos(&ultimo);
    if let Some(o) = c.as_object_mut() { o.remove("web"); o.remove("fluxo"); }
    otimizar(&mut c);
    // Caminho rápido: sem extras, vai direto para o motor.
    if !web && fluxo.len() < 2 && !recursos {
        match est.ollama.chat_fluxo(&c) { Ok(r) => enviar_fluxo(rq, "application/x-ndjson", r), Err(e) => resp_json(rq, 502, &json!({"erro": e})) }
        return;
    }
    let canal = orquestra::iniciar(est.clone(), orquestra::Pedido { corpo: c, web, fluxo, recursos, pergunta: ultimo });
    enviar_fluxo(rq, "application/x-ndjson", Box::new(canal));
}

fn escolher_arquivo(titulo: &str) -> Result<String, String> {
    #[cfg(windows)] {
        let ps = format!("Add-Type -AssemblyName System.Windows.Forms; $f=New-Object System.Windows.Forms.OpenFileDialog; $f.Title='{}'; $f.Filter='Vídeos|*.mp4;*.mkv;*.mov;*.avi;*.webm;*.m4v|Todos|*.*'; if($f.ShowDialog() -eq 'OK'){{[Console]::OutputEncoding=[Text.Encoding]::UTF8; $f.FileName}}", titulo.replace('\'', ""));
        let mut c = std::process::Command::new("powershell"); c.args(["-NoProfile", "-STA", "-Command", &ps]); sem_janela(&mut c);
        let o = c.output().map_err(|e| e.to_string())?;
        return Ok(String::from_utf8_lossy(&o.stdout).trim().to_string());
    }
    #[cfg(not(windows))] { let _ = titulo; Err("Seletor de arquivos só existe no Windows; cole o caminho.".into()) }
}

fn ps(script: &str, nome: &str) -> Result<String, String> {
    #[cfg(windows)] {
        let mut c = std::process::Command::new("powershell"); c.args(["-NoProfile", "-STA", "-Command", &format!("[Console]::OutputEncoding=[Text.Encoding]::UTF8; Add-Type -AssemblyName System.Windows.Forms; {}", script)]).env("HIRELOD_NOME", nome); sem_janela(&mut c);
        let o = c.output().map_err(|e| e.to_string())?; return Ok(String::from_utf8_lossy(&o.stdout).trim().to_string());
    }
    #[cfg(not(windows))] { let _ = (script, nome); Err("Escolher a pasta só funciona no Windows.".into()) }
}
fn dialogo_salvar(nome: &str) -> Result<Option<String>, String> {
    let r = ps("$f=New-Object System.Windows.Forms.SaveFileDialog; $f.FileName=$env:HIRELOD_NOME; $f.InitialDirectory=[Environment]::GetFolderPath('MyDocuments'); $f.Title='Salvar em…'; if($f.ShowDialog() -eq 'OK'){$f.FileName}", nome)?;
    Ok(if r.is_empty() { None } else { Some(r) })
}
fn dialogo_pasta() -> Result<Option<String>, String> {
    let r = ps("$f=New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description='Escolha a pasta para salvar'; if($f.ShowDialog() -eq 'OK'){$f.SelectedPath}", "")?;
    Ok(if r.is_empty() { None } else { Some(r) })
}
fn abrir_pasta(p: &std::path::Path) {
    #[cfg(windows)] { let mut c = std::process::Command::new("explorer"); if p.is_file() { c.arg("/select,"); } c.arg(p); sem_janela(&mut c); let _ = c.spawn(); }
    #[cfg(not(windows))] { let _ = p; }
}

fn rota(est: &Arc<Estado>, mut rq: Request) {
    let url = rq.url().to_string();
    let caminho = url.split('?').next().unwrap_or("/").to_string();
    let metodo = rq.method().clone();
    // Só aceita o próprio computador (protege contra páginas externas e DNS rebinding)
    let host = cab(&rq, "Host");
    let hn = host.rsplit_once(':').map(|x| x.0).unwrap_or(&host).to_lowercase();
    if hn != "127.0.0.1" && hn != "localhost" { return erro(rq, 403, "acesso negado"); }
    if !caminho.starts_with("/api/") {
        if metodo != Method::Get { return erro(rq, 405, "método não permitido"); }
        return servir_ui(rq, &caminho, est);
    }
    if metodo != Method::Get && cab(&rq, "X-Hirelod") != "1" { return erro(rq, 403, "pedido recusado"); }
    let uid = id_seguro(&cab(&rq, "X-Usuario"));
    let uid_ok = !uid.is_empty() && est.repo.existe(&uid);
    let seg: Vec<&str> = caminho.trim_start_matches("/api/").split('/').collect();
    let m = metodo.clone();
    macro_rules! precisa_usuario { () => { if !uid_ok { return erro(rq, 401, "Escolha um usuário."); } } }
    match (m, seg.as_slice()) {
        (Method::Get, ["estado"]) => resp_json(rq, 200, &json!({"versao": VERSAO, "ollama": est.ollama.versao().map(|v| format!("Karthus IA {}", v)).unwrap_or_else(|e| e)})),
        (Method::Get, ["usuarios"]) => resp_json(rq, 200, &json!(est.repo.listar())),
        (Method::Post, ["usuarios"]) => { let b = corpo_json(&mut rq); match est.repo.criar(b["nome"].as_str().unwrap_or(""), b["cor"].as_str().unwrap_or("")) { Ok(u) => resp_json(rq, 200, &u), Err(e) => erro(rq, 400, &e) } }
        (Method::Put, ["usuarios", id]) => { let b = corpo_json(&mut rq); match est.repo.atualizar(id, &b) { Ok(u) => resp_json(rq, 200, &u), Err(e) => erro(rq, 400, &e) } }
        (Method::Delete, ["usuarios", id]) => match est.repo.remover(id) { Ok(_) => resp_json(rq, 200, &json!({"ok": true})), Err(e) => erro(rq, 400, &e) },
        (Method::Get, ["config"]) => { precisa_usuario!(); resp_json(rq, 200, &est.repo.config(&uid)) }
        (Method::Put, ["config"]) => { precisa_usuario!(); let b = corpo_json(&mut rq); match est.repo.gravar_config(&uid, &b) { Ok(c) => resp_json(rq, 200, &c), Err(e) => erro(rq, 400, &e) } }
        (Method::Get, ["conversas"]) => { precisa_usuario!(); resp_json(rq, 200, &json!(est.repo.conversas(&uid))) }
        (Method::Get, ["conversas", id]) => { precisa_usuario!(); match est.repo.ler_conversa(&uid, id) { Some(c) => resp_json(rq, 200, &c), None => erro(rq, 404, "não encontrada") } }
        (Method::Put, ["conversas", id]) => { precisa_usuario!(); let b = corpo_json(&mut rq); match est.repo.gravar_conversa(&uid, id, &b) { Ok(_) => resp_json(rq, 200, &json!({"ok": true})), Err(e) => erro(rq, 400, &e) } }
        (Method::Delete, ["conversas", id]) => { precisa_usuario!(); est.repo.apagar_conversa(&uid, id); resp_json(rq, 200, &json!({"ok": true})) }
        (Method::Post, ["chat"]) => { precisa_usuario!(); chat(est, rq, &uid) }
        (Method::Get, ["modelos"]) => resp_json(rq, 200, &json!(modelos_locais::escanear(est).iter().map(|a| json!({"nome": a.nome, "tamanho": a.tamanho})).collect::<Vec<_>>())),
        (Method::Get, ["global"]) => { let mut g = est.global(); g["modelo_padrao"] = json!(est.modelo_padrao()); resp_json(rq, 200, &g) }
        (Method::Put, ["global"]) => { let b = corpo_json(&mut rq); match est.gravar_global(&b) { Ok(_) => resp_json(rq, 200, &est.global()), Err(e) => erro(rq, 400, &e) } }
        (Method::Get, ["saude"]) => resp_json(rq, 200, &carga::saude(est)),
        (Method::Get, ["carga"]) => resp_json(rq, 200, &carga::json(est)),
        (Method::Post, ["carga", "verificar"]) => { carga::verificar(est); resp_json(rq, 200, &json!({"ok": true})) }
        (Method::Post, ["carga", "pular"]) => { est.pular.store(true, std::sync::atomic::Ordering::SeqCst); resp_json(rq, 200, &json!({"ok": true})) }
        (Method::Post, ["instalar", item]) => match carga::instalar(est, item) { Ok(j) => resp_json(rq, 200, &json!({"ok": true, "download": j})), Err(e) => erro(rq, 400, &e) },
        (Method::Get, ["downloads"]) => resp_json(rq, 200, &downloads::json(est)),
        (Method::Post, ["downloads", "limpar"]) => { downloads::limpar(est); resp_json(rq, 200, &json!({"ok": true})) }
        (Method::Get, ["catalogo"]) => resp_json(rq, 200, &catalogo::lista(est)),
        (Method::Post, ["catalogo", "atualizar"]) => resp_json(rq, 200, &json!({"ok": true, "download": catalogo::iniciar_atualizacao(est)})),
        (Method::Post, ["catalogo", "baixar"]) => { let b = corpo_json(&mut rq); match catalogo::baixar(est, b["id"].as_str().unwrap_or("")) { Ok(j) => resp_json(rq, 200, &json!({"ok": true, "download": j})), Err(e) => erro(rq, 400, &e) } }
        (Method::Post, ["catalogo", "usar"]) => { let b = corpo_json(&mut rq); match est.gravar_global(&json!({"modelo_padrao": b["nome"]})) { Ok(_) => resp_json(rq, 200, &json!({"ok": true})), Err(e) => erro(rq, 400, &e) } }
        (Method::Delete, ["catalogo", nome]) => match catalogo::remover(est, &decodificar(nome)) { Ok(_) => resp_json(rq, 200, &json!({"ok": true})), Err(e) => erro(rq, 400, &e) },
        (Method::Get, ["recursos"]) => resp_json(rq, 200, &recursos::inventario(est)),
        (Method::Post, ["recursos", "aplicar-imagem"]) => resp_json(rq, 200, &recursos::aplicar_imagem(est)),
        // ---- criações
        (Method::Get, ["criacoes"]) => { precisa_usuario!(); resp_json(rq, 200, &json!(est.repo.criacoes(&uid))) }
        (Method::Delete, ["criacoes", arq]) => { precisa_usuario!(); let a = nome_arquivo_seguro(&decodificar(arq)); let ok = est.repo.del_criacao(&uid, &a); if ok { let outros = est.repo.listar().iter().any(|u| u["id"].as_str().map(|i| i != uid && est.repo.criacoes(i).iter().any(|c| c["arq"] == json!(a))).unwrap_or(false)); if !outros { let _ = std::fs::remove_file(est.saidas().join(&a)); } } resp_json(rq, 200, &json!({"ok": ok})) }
        (Method::Post, ["criar", "imagem"]) | (Method::Post, ["imagem"]) => { precisa_usuario!(); let b = corpo_json(&mut rq); match criar::imagem(est, &uid, b["prompt"].as_str().unwrap_or(""), b["largura"].as_u64().unwrap_or(512) as u32, b["altura"].as_u64().unwrap_or(512) as u32, b["passos"].as_u64().unwrap_or(20) as u32) { Ok(j) => resp_json(rq, 200, &json!({"job": j})), Err(e) => erro(rq, 400, &e) } }
        (Method::Post, ["criar", "video"]) | (Method::Post, ["video"]) => { precisa_usuario!(); let b = corpo_json(&mut rq); match criar::video(est, &uid, b["prompt"].as_str().unwrap_or(""), b["cenas"].as_u64().unwrap_or(4) as u32, b["segundos"].as_u64().unwrap_or(3) as u32, b["largura"].as_u64().unwrap_or(512) as u32, b["altura"].as_u64().unwrap_or(512) as u32) { Ok(j) => resp_json(rq, 200, &json!({"job": j})), Err(e) => erro(rq, 400, &e) } }
        (Method::Post, ["criar", "arquivo"]) => { precisa_usuario!(); let b = corpo_json(&mut rq); let mut mo = b["modelo"].as_str().unwrap_or("").to_string(); if mo.is_empty() { mo = est.modelo_padrao(); } match criar::arquivo(est, &uid, b["pedido"].as_str().unwrap_or(""), &mo) { Ok(j) => resp_json(rq, 200, &json!({"job": j})), Err(e) => erro(rq, 400, &e) } }
        (Method::Post, ["criar", "download"]) => { precisa_usuario!(); let b = corpo_json(&mut rq); match criar::baixar_arquivo(est, &uid, b["url"].as_str().unwrap_or("")) { Ok(j) => resp_json(rq, 200, &json!({"job": j})), Err(e) => erro(rq, 400, &e) } }
        (Method::Post, ["criar", "cortes"]) => { precisa_usuario!(); let b = corpo_json(&mut rq);
            let o = cortes::Opcoes { quantidade: b["quantidade"].as_u64().unwrap_or(3) as usize, min: b["min"].as_f64().unwrap_or(30.0), max: b["max"].as_f64().unwrap_or(60.0), vertical: b["vertical"] == json!(true), legendas: b["legendas"] != json!(false), usar_ia: b["usar_ia"] != json!(false) };
            match cortes::iniciar(est, &uid, b["video"].as_str().unwrap_or(""), o, est.modelo_padrao()) { Ok(j) => resp_json(rq, 200, &json!({"job": j})), Err(e) => erro(rq, 400, &e) } }
        (Method::Get, ["imagem", job]) => { let d = downloads::lista(est).into_iter().find(|d| d.id == *job); match d { Some(d) => resp_json(rq, 200, &json!({"estado": match d.estado.as_str() { "ok" => "pronto", "erro" => "erro", _ => "rodando" }, "pct": d.pct, "msg": d.msg, "arquivo": d.arquivo})), None => erro(rq, 404, "tarefa não encontrada") } }
        (Method::Get, ["saida", arq]) => {
            let a = nome_arquivo_seguro(&decodificar(arq)); let p = est.saidas().join(&a);
            if !p.is_file() { return erro(rq, 404, "não encontrado"); }
            match std::fs::File::open(&p) { Ok(f) => { let tam = f.metadata().map(|m| m.len()).unwrap_or(0); let mut r = Response::new(tiny_http::StatusCode(200), vec![h("Content-Type", tipo_de(&a)), h("Cache-Control", "private, max-age=3600")], f, Some(tam as usize), None); if query(&url, "baixar").is_some() { r.add_header(h("Content-Disposition", &format!("attachment; filename=\"{}\"", a))); } let _ = rq.respond(r); } Err(_) => erro(rq, 404, "não encontrado") }
        }
        (Method::Post, ["salvar-como"]) => {
            let b = corpo_json(&mut rq); let nome = nome_arquivo_seguro(b["nome"].as_str().unwrap_or("arquivo.txt"));
            let pasta = est.base.join("documentos"); let _ = std::fs::create_dir_all(&pasta);
            let dest = pasta.join(&nome);
            let r = if let Some(s) = b["saida"].as_str() { std::fs::copy(est.saidas().join(nome_arquivo_seguro(s)), &dest).map(|_| ()) } else { std::fs::write(&dest, b["texto"].as_str().unwrap_or("")) };
            match r { Ok(_) => resp_json(rq, 200, &json!({"caminho": dest.to_string_lossy()})), Err(e) => erro(rq, 400, &e.to_string()) }
        }
        (Method::Post, ["salvar-dialogo"]) => {
            let b = corpo_json(&mut rq); let a = nome_arquivo_seguro(b["arq"].as_str().unwrap_or("")); let orig = est.saidas().join(&a);
            if !orig.is_file() { return erro(rq, 404, "Arquivo não encontrado."); }
            match dialogo_salvar(&a) { Ok(Some(dest)) => match std::fs::copy(&orig, &dest) { Ok(_) => resp_json(rq, 200, &json!({"caminho": dest})), Err(e) => erro(rq, 400, &format!("Não consegui salvar: {}", e)) }, Ok(None) => resp_json(rq, 200, &json!({"cancelado": true})), Err(e) => erro(rq, 400, &e) }
        }
        (Method::Post, ["salvar-pasta"]) => {
            let b = corpo_json(&mut rq);
            match dialogo_pasta() { Ok(Some(pasta)) => { let mut n = 0; for x in b["arqs"].as_array().cloned().unwrap_or_default() { let a = nome_arquivo_seguro(x.as_str().unwrap_or("")); let o = est.saidas().join(&a); if o.is_file() && std::fs::copy(&o, std::path::Path::new(&pasta).join(&a)).is_ok() { n += 1; } } resp_json(rq, 200, &json!({"pasta": pasta, "copiados": n})) }, Ok(None) => resp_json(rq, 200, &json!({"cancelado": true})), Err(e) => erro(rq, 400, &e) }
        }
        (Method::Post, ["abrir-pasta"]) => { let b = corpo_json(&mut rq); let p = match b["arq"].as_str() { Some(a) => est.saidas().join(nome_arquivo_seguro(a)), None => est.saidas() }; abrir_pasta(&p); resp_json(rq, 200, &json!({"ok": true})) }
        (Method::Post, ["escolher-arquivo"]) => match escolher_arquivo("Escolha o vídeo") { Ok(c) => resp_json(rq, 200, &json!({"caminho": c})), Err(e) => erro(rq, 400, &e) },
        (Method::Post, ["motores", "adicionar"]) => {
            let tipo = query(&url, "tipo").unwrap_or_default(); let nome = nome_arquivo_seguro(&query(&url, "nome").unwrap_or_default());
            let sub = match tipo.as_str() { "voz" | "whisper" => "whisper", "imagem" => "imagem", "video" => "video", "modelo" => "modelos", _ => return erro(rq, 400, "tipo inválido") };
            let dir = est.motores().join(sub); let _ = std::fs::create_dir_all(&dir);
            let dest = dir.join(&nome);
            let mut tmp = dest.clone(); tmp.set_extension("parte");
            let r = (|| -> Result<(), String> { let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?; std::io::copy(&mut rq.as_reader(), &mut f).map_err(|e| e.to_string())?; std::fs::rename(&tmp, &dest).map_err(|e| e.to_string()) })();
            match r { Ok(_) => { carga::verificar(est); resp_json(rq, 200, &json!({"ok": true})) } Err(e) => { let _ = std::fs::remove_file(&tmp); erro(rq, 400, &e) } }
        }
        // ---- voz
        (Method::Post, ["voz", "gravar"]) => match est.voz.gravar(est) { Ok(_) => resp_json(rq, 200, &json!({"ok": true})), Err(e) => erro(rq, 400, &e) },
        (Method::Get, ["voz", "estado"]) => resp_json(rq, 200, &est.voz.estado()),
        (Method::Post, ["voz", "parar"]) => resp_json(rq, 200, &json!({"texto": est.voz.parar_e_ler()})),
        (Method::Post, ["voz", "falar"]) => {
            let b = corpo_json(&mut rq);
            let escolhida = b["voz"].as_str().map(String::from).unwrap_or_else(|| if uid_ok { est.repo.config(&uid)["voz_nome"].as_str().unwrap_or("").to_string() } else { String::new() });
            let texto = b["texto"].as_str().unwrap_or("");
            let r = if let Some(id) = escolhida.strip_prefix("piper:") {
                match (crate::motores_web::piper_exe(est), crate::motores_web::voz_piper_arquivo(est, id)) {
                    (Some(e), Some(m)) => { crate::motores_web::sanear_json_voz(&m.with_extension("onnx.json")); est.voz.falar_piper(e, m, texto, est.dados.join("tmp")).or_else(|er| { log(&est.base, &er); let _ = est.voz.falar(texto, None); Err(er) }) }
                    _ => est.voz.falar(texto, None) }
            } else { est.voz.falar(texto, if escolhida.is_empty() { None } else { Some(escolhida.as_str()) }) };
            match r { Ok(_) => resp_json(rq, 200, &json!({"ok": true})), Err(e) => erro(rq, 400, &e) } }
        (Method::Get, ["voz", "catalogo"]) => resp_json(rq, 200, &json!(crate::motores_web::VOZES_PIPER.iter().map(|v| json!({"id": format!("piper:{}", v.0), "nome": v.1, "instalada": crate::motores_web::voz_piper_arquivo(est, v.0).is_some() && crate::motores_web::piper_exe(est).is_some(), "baixando": downloads::ativo_do_item(est, &format!("voz:{}", v.0)).is_some()})).collect::<Vec<_>>())),
        (Method::Post, ["voz", "baixar"]) => { let b = corpo_json(&mut rq); let id = b["id"].as_str().unwrap_or("").trim_start_matches("piper:").to_string();
            let Some(v) = crate::motores_web::VOZES_PIPER.iter().find(|v| v.0 == id) else { return erro(rq, 400, "Voz desconhecida."); };
            let item = format!("voz:{}", id);
            if let Some(j) = downloads::ativo_do_item(est, &item) { return resp_json(rq, 200, &json!({"ok": true, "download": j})); }
            let job = downloads::novo(est, &item, &format!("Voz: {}", v.1), "voz", "fila", v.4);
            let (e, j) = (est.clone(), job.clone());
            std::thread::spawn(move || { let _g = e.fila.lock().unwrap_or_else(|x| x.into_inner()); downloads::mexer(&e, &j, |d| { d.estado = "ativo".into(); d.desde_ms = agora_ms(); }); downloads::concluir(&e, &j, crate::motores_web::instalar_voz_piper(&e, &j, &id).map(|_| "Voz instalada".to_string())); });
            resp_json(rq, 200, &json!({"ok": true, "download": job})) }
        (Method::Post, ["voz", "calar"]) => { est.voz.calar(); resp_json(rq, 200, &json!({"ok": true})) }
        (Method::Get, ["voz", "vozes"]) => { let mut v = est.voz.vozes(); for x in crate::motores_web::VOZES_PIPER.iter().filter(|x| crate::motores_web::voz_piper_arquivo(est, x.0).is_some() && crate::motores_web::piper_exe(est).is_some()) { v.push(json!({"nome": format!("piper:{}", x.0), "rotulo": x.1, "idioma": "pt-BR"})); } resp_json(rq, 200, &json!(v)) }
        // ---- atualização
        (Method::Post, ["atualizar", "inspecionar"]) => match corpo(&mut rq, 400_000_000).and_then(|b| atualizar::inspecionar(est, &b)) { Ok(v) => resp_json(rq, 200, &v), Err(e) => erro(rq, 400, &e) },
        (Method::Post, ["atualizar", "aplicar"]) => { let b = corpo_json(&mut rq); match atualizar::aplicar(est, b["token"].as_str().unwrap_or("")) { Ok(v) => resp_json(rq, 200, &v), Err(e) => erro(rq, 400, &e) } }
        (Method::Post, ["sair"]) => { est.sair.store(true, std::sync::atomic::Ordering::SeqCst); resp_json(rq, 200, &json!({"ok": true})) }
        _ => erro(rq, 404, "rota desconhecida"),
    }
}

pub fn iniciar(est: Arc<Estado>) -> Result<(), String> {
    let s = Server::http(("127.0.0.1", est.porta)).map_err(|e| format!("Porta {} ocupada: {}", est.porta, e))?;
    let s = Arc::new(s);
    std::thread::spawn(move || {
        for rq in s.incoming_requests() {
            let e = est.clone();
            std::thread::spawn(move || { let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rota(&e, rq))); });
        }
    });
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test] fn otimiza_oi() {
        let mut c = json!({"messages": [{"role": "system", "content": "persona"}, {"role": "user", "content": "x".repeat(3000)}, {"role": "assistant", "content": "y"}, {"role": "user", "content": "oi"}]});
        otimizar(&mut c);
        assert_eq!(c["messages"].as_array().unwrap().len(), 2); assert_eq!(c["options"]["num_ctx"], 1024); assert_eq!(c["options"]["num_predict"], 90); assert_eq!(c["think"], false);
    }
    #[test] fn ctx_cresce() { let mut c = json!({"messages": [{"role": "user", "content": "a".repeat(9000)}]}); otimizar(&mut c); assert!(c["options"]["num_ctx"].as_u64().unwrap() >= 4096); }
    #[test] fn consulta() { assert_eq!(query("/a?tipo=voz&nome=a%20b.bin", "nome").unwrap(), "a b.bin"); }
}

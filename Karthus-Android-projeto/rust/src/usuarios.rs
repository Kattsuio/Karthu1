//! Usuários, configurações, conversas e lista de criações. Tudo em arquivos JSON dentro de dados/usuarios/<id>/.
use crate::util::*;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct Repo { pub raiz: PathBuf }

fn so_texto(v: &Value, max: usize) -> String { v.as_str().unwrap_or("").trim().chars().take(max).collect() }

impl Repo {
    pub fn novo(dados: &std::path::Path) -> Repo { Repo { raiz: dados.join("usuarios") } }
    fn dir(&self, id: &str) -> PathBuf { self.raiz.join(id_seguro(id)) }
    pub fn existe(&self, id: &str) -> bool { !id_seguro(id).is_empty() && self.dir(id).join("perfil.json").is_file() }

    pub fn listar(&self) -> Vec<Value> {
        let mut v: Vec<Value> = std::fs::read_dir(&self.raiz).map(|r| r.flatten().filter_map(|e| ler_json::<Value>(&e.path().join("perfil.json"))).collect()).unwrap_or_default();
        v.sort_by_key(|u| u["criado"].as_u64().unwrap_or(0));
        v
    }
    pub fn criar(&self, nome: &str, cor: &str) -> Result<Value, String> {
        let nome = nome.trim().chars().take(40).collect::<String>();
        if nome.is_empty() { return Err("Digite um nome.".into()); }
        if self.listar().len() >= 12 { return Err("O limite é de 12 usuários.".into()); }
        let cor = if cor.starts_with('#') && cor.len() == 7 && cor[1..].chars().all(|c| c.is_ascii_hexdigit()) { cor.to_string() } else { "#d4202f".into() };
        let id = novo_id("u");
        let p = json!({"id": id, "nome": nome, "cor": cor, "criado": agora_ms()});
        gravar_json(&self.dir(&id).join("perfil.json"), &p).map_err(|e| e.to_string())?;
        Ok(p)
    }
    pub fn atualizar(&self, id: &str, patch: &Value) -> Result<Value, String> {
        let f = self.dir(id).join("perfil.json");
        let mut p: Value = ler_json(&f).ok_or("Usuário não encontrado.")?;
        if let Some(n) = patch.get("nome") { let n = so_texto(n, 40); if n.is_empty() { return Err("Digite um nome.".into()); } p["nome"] = json!(n); }
        if let Some(c) = patch["cor"].as_str() { if c.starts_with('#') && c.len() == 7 { p["cor"] = json!(c); } }
        gravar_json(&f, &p).map_err(|e| e.to_string())?;
        Ok(p)
    }
    pub fn remover(&self, id: &str) -> Result<(), String> {
        if !self.existe(id) { return Err("Usuário não encontrado.".into()); }
        let lixo = self.raiz.parent().unwrap_or(&self.raiz).join("lixeira");
        std::fs::create_dir_all(&lixo).map_err(|e| e.to_string())?;
        std::fs::rename(self.dir(id), lixo.join(format!("{}-{}", id_seguro(id), agora_s()))).map_err(|e| e.to_string())
    }

    pub fn config(&self, id: &str) -> Value { ler_json(&self.dir(id).join("config.json")).unwrap_or_else(|| json!({"tema": "claro", "acento": "branco"})) }
    pub fn gravar_config(&self, id: &str, patch: &Value) -> Result<Value, String> {
        let mut c = self.config(id);
        const CHAVES: [&str; 9] = ["tema", "acento", "persona", "modelo", "pensar", "voz_responder", "voz_nome", "escala", "rapido"];
        if let Some(o) = patch.as_object() { for (k, v) in o { if CHAVES.contains(&k.as_str()) { c[k] = v.clone(); } } }
        gravar_json(&self.dir(id).join("config.json"), &c).map_err(|e| e.to_string())?;
        Ok(c)
    }

    fn pasta_conv(&self, id: &str) -> PathBuf { self.dir(id).join("conversas") }
    pub fn conversas(&self, id: &str) -> Vec<Value> {
        let mut v: Vec<Value> = std::fs::read_dir(self.pasta_conv(id)).map(|r| r.flatten().filter_map(|e| {
            let c: Value = ler_json(&e.path())?;
            Some(json!({"id": c["id"], "titulo": c["titulo"], "atualizado": c["atualizado"], "fixada": c["fixada"].as_bool().unwrap_or(false), "mensagens": c["mensagens"].as_array().map(|a| a.len()).unwrap_or(0)}))
        }).collect()).unwrap_or_default();
        v.sort_by_key(|c| std::cmp::Reverse(c["atualizado"].as_u64().unwrap_or(0)));
        v
    }
    pub fn ler_conversa(&self, uid: &str, cid: &str) -> Option<Value> { ler_json(&self.pasta_conv(uid).join(format!("{}.json", id_seguro(cid)))) }
    pub fn gravar_conversa(&self, uid: &str, cid: &str, c: &Value) -> Result<(), String> {
        let cid = id_seguro(cid);
        if cid.is_empty() { return Err("Id de conversa inválido.".into()); }
        let mut c = c.clone();
        c["id"] = json!(cid); c["atualizado"] = json!(agora_ms());
        gravar_json(&self.pasta_conv(uid).join(format!("{}.json", cid)), &c).map_err(|e| e.to_string())
    }
    pub fn apagar_conversa(&self, uid: &str, cid: &str) { let _ = std::fs::remove_file(self.pasta_conv(uid).join(format!("{}.json", id_seguro(cid)))); }

    // ---- criações (tudo que o Karthus gerou para este usuário)
    fn arq_cri(&self, id: &str) -> PathBuf { self.dir(id).join("criacoes.json") }
    pub fn criacoes(&self, id: &str) -> Vec<Value> { ler_json(&self.arq_cri(id)).unwrap_or_default() }
    pub fn add_criacao(&self, id: &str, c: Value) {
        let mut l = self.criacoes(id);
        l.insert(0, c);
        l.truncate(500);
        let _ = gravar_json(&self.arq_cri(id), &Value::Array(l));
    }
    pub fn del_criacao(&self, id: &str, arq: &str) -> bool {
        let mut l = self.criacoes(id);
        let n = l.len();
        l.retain(|c| c["arq"].as_str() != Some(arq));
        let ok = l.len() != n;
        if ok { let _ = gravar_json(&self.arq_cri(id), &Value::Array(l)); }
        ok
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test] fn ciclo() {
        let d = std::env::temp_dir().join(format!("hl-test-{}", novo_id("x")));
        let r = Repo::novo(&d);
        let u = r.criar("Davi", "#2f6fed").unwrap();
        let id = u["id"].as_str().unwrap().to_string();
        assert!(r.existe(&id) && !r.existe("../x"));
        r.gravar_config(&id, &json!({"tema": "escuro", "lixo": 1})).unwrap();
        assert_eq!(r.config(&id)["tema"], "escuro"); assert!(r.config(&id).get("lixo").is_none());
        r.gravar_conversa(&id, "c1", &json!({"titulo": "oi", "mensagens": []})).unwrap();
        assert_eq!(r.conversas(&id).len(), 1);
        r.add_criacao(&id, json!({"arq": "a.png"})); assert_eq!(r.criacoes(&id).len(), 1); assert!(r.del_criacao(&id, "a.png"));
        r.remover(&id).unwrap(); assert!(r.listar().is_empty());
        let _ = std::fs::remove_dir_all(d);
    }
}

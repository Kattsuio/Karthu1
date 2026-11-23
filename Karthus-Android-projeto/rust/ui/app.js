'use strict';
/* Karthus 4.0 - interface. Tudo conversa com o programa em Rust pelo endereço local. */
const $ = (s, e = document) => e.querySelector(s);
const $$ = (s, e = document) => [...e.querySelectorAll(s)];
const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const P = d => '<svg viewBox="0 0 24 24">' + d + '</svg>';
const IC = {
  plus: P('<path d="M12 5v14M5 12h14"/>'), search: P('<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>'),
  lado: P('<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M9 4v16"/>'),
  novo: P('<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/>'),
  cri: P('<path d="M12 3l1.9 4.6L19 9.5l-4.1 2.9L16 18l-4-2.8L8 18l1.1-5.6L5 9.5l5.1-1.9z"/><path d="M19 3v4M17 5h4"/>'),
  img: P('<rect x="3" y="3" width="18" height="18" rx="3"/><circle cx="9" cy="9" r="1.6"/><path d="m21 15-5-5L5 21"/>'),
  cfg: P('<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/>'),
  mic: P('<rect x="9" y="3" width="6" height="12" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3"/>'),
  onda: P('<path d="M4 10v4M8 6v12M12 3v18M16 7v10M20 10v4"/>'),
  seta: P('<path d="M12 19V5M5 12l7-7 7 7"/>'), parar: P('<rect x="6" y="6" width="12" height="12" rx="2" fill="currentColor"/>'),
  copiar: P('<rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15V6a2 2 0 0 1 2-2h9"/>'),
  baixar: P('<path d="M12 3v12M7 10l5 5 5-5M5 21h14"/>'), refazer: P('<path d="M3 12a9 9 0 0 1 15.5-6.2L21 8M21 3v5h-5M21 12a9 9 0 0 1-15.5 6.2L3 16M3 21v-5h5"/>'),
  falar: P('<path d="M11 5 6 9H3v6h3l5 4zM15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13"/>'),
  lixo: P('<path d="M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3"/>'), mais: P('<circle cx="5" cy="12" r="1.2"/><circle cx="12" cy="12" r="1.2"/><circle cx="19" cy="12" r="1.2"/>'),
  x: P('<path d="M6 6l12 12M18 6 6 18"/>'), clip: P('<path d="m21 12-8.5 8.5a5 5 0 0 1-7-7L14 5a3.500 3.500 0 0 1 5 5l-8.500 8.500a2 2 0 0 1-3-3L15 8"/>'),
  pensar: P('<path d="M9 18h6M10 21h4M12 3a6 6 0 0 0-4 10.500c.7.7 1 1.500 1 2.500h6c0-1 .3-1.800 1-2.500A6 6 0 0 0 12 3z"/>'),
  fixar: P('<path d="M12 17v5M9 3h6l-1 7 3 3H7l3-3z"/>'), nome: P('<path d="M12 20h9M16.500 3.500a2.100 2.100 0 0 1 3 3L7 19l-4 1 1-4z"/>'),
  baixo: P('<path d="m6 9 6 6 6-6"/>'), usuario: P('<circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/>'), edit: P('<path d="M12 20h9M16.500 3.500a2.100 2.100 0 0 1 3 3L7 19l-4 1 1-4z"/>'),
  texto: P('<path d="M4 6h16M4 12h16M4 18h10"/>'), arq: P('<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/>'),
};
const S = { usuario: null, usuarios: [], cfg: {}, convs: [], conv: null, modelos: [], modeloPadrao: '', saude: null, gerando: false, abort: null, gens: new Map(), obs: new Set(), anexos: [], gravando: false, vozModo: false, global: {} };
const H = () => ({ 'X-Hirelod': '1', 'Content-Type': 'application/json', 'X-Usuario': S.usuario ? S.usuario.id : '' });

async function api(m, u, corpo, opc = {}) {
  const r = await fetch(u, { method: m, headers: opc.cru ? { 'X-Hirelod': '1', 'X-Usuario': S.usuario ? S.usuario.id : '' } : H(), body: m === 'GET' || m === 'DELETE' ? undefined : (opc.cru ? corpo : JSON.stringify(corpo || {})) });
  let j = null; try { j = await r.json(); } catch (_) {}
  if (!r.ok) { const e = new Error((j && j.erro) || ('Erro ' + r.status)); e.status = r.status; throw e; }
  return j;
}
let avisoT;
function aviso(t, ms = 4500) { const a = $('#aviso'); a.textContent = t; a.classList.add('on'); clearTimeout(avisoT); avisoT = setTimeout(() => a.classList.remove('on'), ms); }
const iniciais = n => (n || '?').trim().slice(0, 1).toUpperCase();
const avatar = (u, cls = '') => `<span class="av ${cls}" style="background:${esc(u.cor)}">${esc(iniciais(u.nome))}</span>`;

/* ---------------------------------------------------------------- markdown */
function inline(t) {
  t = esc(t);
  t = t.replace(/`([^`\n]+)`/g, '<code>$1</code>');
  t = t.replace(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>').replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, '$1<em>$2</em>');
  return t;
}
const NOME_ARQ = /^\s*(?:#|\/\/|--|;|<!--|\/\*|REM)\s*(?:arquivo|file|nome)?\s*[:=]?\s*([\w][\w .\-\/\\]{0,80}\.[A-Za-z0-9]{1,8})\s*(?:-->|\*\/)?\s*$/i;
function codigo(lang, txt, i) {
  const m = NOME_ARQ.exec(txt.split('\n')[0] || '');
  const nome = m ? m[1].trim() : '';
  return `<div class="cod" data-i="${i}"><div class="cod-topo"><b>${esc(nome || lang || 'código')}</b><span class="esp"></span><button data-a="copiar-cod">${IC.copiar} Copiar</button>${(nome || txt.split('\n').length > 3) ? `<button data-a="baixar-cod" data-n="${esc(nome)}">${IC.baixar} Baixar</button>` : ''}</div><pre><code>${esc(txt.replace(/\n$/, ''))}</code></pre></div>`;
}
function md(src) {
  const blocos = [];
  src = String(src || '').replace(/\r\n/g, '\n');
  // blocos de código (também os ainda abertos durante a resposta)
  src = src.replace(/```([\w+#.-]*)[ \t]*\n([\s\S]*?)(?:```|$)/g, (_, l, c) => { blocos.push([l, c]); return `\u0000${blocos.length - 1}\u0000`; });
  const linhas = src.split('\n'); const out = []; let lista = null, par = [];
  const fechaP = () => { if (par.length) { out.push('<p>' + inline(par.join('\n')).replace(/\n/g, '<br>') + '</p>'); par = []; } };
  const fechaL = () => { if (lista) { out.push(`</${lista}>`); lista = null; } };
  for (let i = 0; i < linhas.length; i++) {
    const l = linhas[i]; let m;
    if ((m = /^\u0000(\d+)\u0000$/.exec(l.trim()))) { fechaP(); fechaL(); out.push(codigo(blocos[+m[1]][0], blocos[+m[1]][1], +m[1])); continue; }
    if (/^\s*$/.test(l)) { fechaP(); fechaL(); continue; }
    if ((m = /^(#{1,3})\s+(.*)$/.exec(l))) { fechaP(); fechaL(); out.push(`<h${m[1].length}>${inline(m[2])}</h${m[1].length}>`); continue; }
    if (/^\s*([-*_])\1{2,}\s*$/.test(l)) { fechaP(); fechaL(); out.push('<hr>'); continue; }
    if ((m = /^\s*>\s?(.*)$/.exec(l))) { fechaP(); fechaL(); out.push(`<blockquote>${inline(m[1])}</blockquote>`); continue; }
    if (/^\s*\|.*\|\s*$/.test(l) && /^\s*\|?[\s:|-]+\|?\s*$/.test(linhas[i + 1] || '') && /-/.test(linhas[i + 1] || '')) {
      fechaP(); fechaL();
      const cel = r => r.trim().replace(/^\||\|$/g, '').split('|').map(c => c.trim());
      let h = '<table><thead><tr>' + cel(l).map(c => `<th>${inline(c)}</th>`).join('') + '</tr></thead><tbody>'; i += 2;
      for (; i < linhas.length && /^\s*\|.*\|\s*$/.test(linhas[i]); i++) h += '<tr>' + cel(linhas[i]).map(c => `<td>${inline(c)}</td>`).join('') + '</tr>';
      i--; out.push(h + '</tbody></table>'); continue;
    }
    if ((m = /^\s*([-*•])\s+(.*)$/.exec(l))) { fechaP(); if (lista !== 'ul') { fechaL(); out.push('<ul>'); lista = 'ul'; } out.push(`<li>${inline(m[2])}</li>`); continue; }
    if ((m = /^\s*(\d+)[.)]\s+(.*)$/.exec(l))) { fechaP(); if (lista !== 'ol') { fechaL(); out.push('<ol>'); lista = 'ol'; } out.push(`<li>${inline(m[2])}</li>`); continue; }
    fechaL(); par.push(l);
  }
  fechaP(); fechaL();
  return out.join('');
}
const textoLimpo = t => String(t || '').replace(/```[\s\S]*?(```|$)/g, ' (código) ').replace(/[*_`#>|]/g, '').replace(/\s+/g, ' ').trim();

/* ---------------------------------------------------------------- usuários */
async function telaUsuarios(editando = false) {
  S.usuarios = await api('GET', '/api/usuarios');
  let e = $('#entrada'); if (!e) { e = document.createElement('div'); e.id = 'entrada'; document.body.appendChild(e); }
  if (!S.usuarios.length) return formUsuario(true);
  e.className = editando ? 'editando' : '';
  e.innerHTML = `<h1>Quem está usando o Karthus?</h1><div class="grade">${S.usuarios.map(u => `<div style="position:relative"><button class="card-u" data-u="${esc(u.id)}">${avatar(u)}<b>${esc(u.nome)}</b></button><button class="x" data-del="${esc(u.id)}" title="Remover ${esc(u.nome)}" style="position:absolute;top:2px;right:6px;width:26px;height:26px;border-radius:50%;background:var(--bg3);display:${editando ? 'inline-flex' : 'none'};align-items:center;justify-content:center">${IC.x}</button></div>`).join('')}
    <button class="card-u novo" id="novoU"><span class="av">${IC.plus}</span><b>Adicionar</b></button></div>
    <button class="bs" id="editU">${editando ? 'Concluir' : 'Gerenciar usuários'}</button>`;
  $$('[data-u]', e).forEach(b => b.onclick = () => entrar(S.usuarios.find(u => u.id === b.dataset.u)));
  $('#novoU', e).onclick = () => formUsuario(false);
  $('#editU', e).onclick = () => telaUsuarios(!editando);
  $$('[data-del]', e).forEach(b => b.onclick = async () => {
    const u = S.usuarios.find(x => x.id === b.dataset.del);
    if (!await confirmar(`Remover ${u.nome}? As conversas dele vão para a pasta "lixeira" do Karthus (não são apagadas).`)) return;
    try { await api('DELETE', '/api/usuarios/' + u.id); } catch (er) { aviso(er.message); }
    telaUsuarios(true);
  });
}
const CORES = ['#d4202f', '#2f6fed', '#1f9d6b', '#e08a00', '#8b4fd6', '#c2307a', '#0f8fa0', '#5b6270'];
function formUsuario(primeiro) {
  const e = $('#entrada'); let cor = CORES[S.usuarios.length % CORES.length];
  e.className = '';
  e.innerHTML = `<h1>${primeiro ? 'Bem-vindo ao Karthus. Qual é o seu nome?' : 'Novo usuário'}</h1><div class="form-u"><span class="av" id="prev" style="width:84px;height:84px;font-size:36px;background:${cor}">?</span>
    <input type="text" id="nomeU" maxlength="40" placeholder="Nome" style="font-size:18px;padding:10px 14px;width:min(320px,80vw);text-align:center" autofocus>
    <div class="cores">${CORES.map(c => `<button data-c="${c}" style="background:${c}" class="${c === cor ? 'on' : ''}"></button>`).join('')}</div>
    <div style="display:flex;gap:10px"><button class="bp" id="okU">Criar e entrar</button>${primeiro ? '' : '<button class="bs" id="cancU">Cancelar</button>'}</div><div id="erroU" style="color:var(--red)"></div></div>`;
  const n = $('#nomeU', e);
  n.oninput = () => { $('#prev', e).textContent = iniciais(n.value); };
  $$('[data-c]', e).forEach(b => b.onclick = () => { cor = b.dataset.c; $$('[data-c]', e).forEach(x => x.classList.toggle('on', x === b)); $('#prev', e).style.background = cor; });
  const criar = async () => {
    try { const u = await api('POST', '/api/usuarios', { nome: n.value, cor }); S.usuarios.push(u); entrar(u); }
    catch (er) { $('#erroU', e).textContent = er.message; }
  };
  $('#okU', e).onclick = criar; n.onkeydown = ev => { if (ev.key === 'Enter') criar(); };
  if (!primeiro) $('#cancU', e).onclick = () => telaUsuarios();
  n.focus();
}
async function entrar(u) {
  S.usuario = u; sessionStorage.setItem('hl.u', u.id);
  const e = $('#entrada'); if (e) e.remove(); reanimar($('#app'), 'entra');
  S.cfg = await api('GET', '/api/config'); aplicarTema();
  S.convs = await api('GET', '/api/conversas');
  desenharPerfil(); desenharLista(); novaConversa();
  carregarModelos(); atualizarSaude(); atualizarPensar();
}
function aplicarTema() {
  if (typeof aplicarEscala === 'function') aplicarEscala();
  const t = S.cfg.tema === 'auto' ? (matchMedia('(prefers-color-scheme: dark)').matches ? 'escuro' : 'claro') : (S.cfg.tema || 'claro');
  document.documentElement.dataset.tema = t; document.documentElement.dataset.acento = ['vermelho', 'azul'].includes(S.cfg.acento) ? S.cfg.acento : 'branco';
}
async function salvarCfg(p) { Object.assign(S.cfg, p); aplicarTema(); atualizarPensar(); try { await api('PUT', '/api/config', p); } catch (e) { aviso(e.message); } }

/* ---------------------------------------------------------------- barra lateral */
function desenharPerfil() { $('#perfilBt').innerHTML = `${avatar(S.usuario)}<span><b>${esc(S.usuario.nome)}</b><small>Trocar usuário ou configurar</small></span>`; }
function desenharLista(filtro = '') {
  const f = filtro.trim().toLowerCase();
  const l = S.convs.filter(c => !f || (c.titulo || '').toLowerCase().includes(f)).sort((a, b) => (b.fixada ? 1 : 0) - (a.fixada ? 1 : 0) || b.atualizado - a.atualizado);
  $('#lista').innerHTML = l.length ? l.map(c => `<div class="conv ${S.conv && S.conv.id === c.id ? 'on' : ''}" data-id="${esc(c.id)}"><button class="t" title="${esc(c.titulo)}">${c.fixada ? '📌 ' : ''}${esc(c.titulo || 'Nova conversa')}</button>${S.gens.has(c.id) ? '<span class="pontos3" title="Respondendo em segundo plano"><i></i><i></i><i></i></span>' : ''}<button class="mais" data-m="${esc(c.id)}">${IC.mais}</button></div>`).join('') : '<div class="vazio">' + (f ? 'Nada encontrado.' : 'Nenhuma conversa ainda.') + '</div>';
}
$('#lista').addEventListener('click', ev => {
  const m = ev.target.closest('[data-m]'), c = ev.target.closest('.conv');
  if (m) return menuConversa(m, m.dataset.m);
  if (c) abrirConversa(c.dataset.id);
});
function menu(ancora, itens, alinharDireita = false) {
  $$('.menu').forEach(x => x.remove());
  const m = document.createElement('div'); m.className = 'menu';
  m.innerHTML = itens.map((i, n) => i === '-' ? '<hr>' : `<button data-n="${n}" class="${i.perigo ? 'perigo' : ''}">${i.ic || ''}<span>${esc(i.t)}${i.s ? `<small>${esc(i.s)}</small>` : ''}</span></button>`).join('');
  document.body.appendChild(m);
  const z = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--z')) || 1; // a interface pode estar ampliada/reduzida
  const r = ancora.getBoundingClientRect();
  const w = m.offsetWidth, h = m.offsetHeight, IW = innerWidth / z, IH = innerHeight / z;
  const rl = r.left / z, rr = r.right / z, rt = r.top / z, rb = r.bottom / z;
  let x = alinharDireita ? rr - w : rl, y = rb + 6;
  if (y + h > IH - 8) y = Math.max(8, rt - h - 6);
  m.style.left = Math.max(8, Math.min(x, IW - w - 8)) + 'px'; m.style.top = y + 'px';
  m.onclick = ev => { const b = ev.target.closest('[data-n]'); if (!b) return; m.remove(); itens[+b.dataset.n].f(); };
  setTimeout(() => document.addEventListener('mousedown', function f(e) { if (!m.contains(e.target)) { m.remove(); document.removeEventListener('mousedown', f); } }), 0);
  return m;
}
function menuConversa(el, id) {
  const c = S.convs.find(x => x.id === id);
  menu(el, [
    { ic: IC.fixar, t: c.fixada ? 'Desafixar' : 'Fixar no topo', f: async () => { const full = await api('GET', '/api/conversas/' + id); full.fixada = !c.fixada; await api('PUT', '/api/conversas/' + id, full); c.fixada = full.fixada; if (S.conv && S.conv.id === id) S.conv.fixada = full.fixada; desenharLista(); } },
    { ic: IC.nome, t: 'Renomear', f: async () => { const n = await perguntar('Novo nome da conversa:', c.titulo); if (n && n.trim()) { const full = await api('GET', '/api/conversas/' + id); full.titulo = n.trim().slice(0, 80); await api('PUT', '/api/conversas/' + id, full); c.titulo = full.titulo; if (S.conv && S.conv.id === id) S.conv.titulo = full.titulo; desenharLista(); } } },
    '-', { ic: IC.lixo, t: 'Apagar', perigo: true, f: async () => { if (!await confirmar('Apagar esta conversa?')) return; await api('DELETE', '/api/conversas/' + id); S.convs = S.convs.filter(x => x.id !== id); if (S.conv && S.conv.id === id) novaConversa(); desenharLista(); } },
  ]);
}
$('#buscaBt').onclick = async () => { const q = await perguntar('Pesquisar conversas pelo título:', ''); if (q !== null) desenharLista(q); };
$('#fecharLado').onclick = () => { $('#app').classList.add('fechado'); $('#abrirLado').style.display = ''; };
$('#abrirLado').onclick = () => { $('#app').classList.remove('fechado'); $('#abrirLado').style.display = 'none'; };
$('#novoBt').onclick = () => novaConversa();
$('#configBt').onclick = () => configuracoes();
$('#imagensBt').onclick = () => telaCriacoes();
$('#perfilBt').onclick = ev => menu(ev.currentTarget, [
  { ic: IC.usuario, t: 'Trocar de usuário', f: () => { sessionStorage.removeItem('hl.u'); S.usuario = null; telaUsuarios(); } },
  { ic: IC.cfg, t: 'Configurações', f: () => configuracoes() },
]);
$('#buscaBt').innerHTML = IC.search; $('#fecharLado').innerHTML = IC.lado; $('#abrirLado').innerHTML = IC.lado;
$('#novoBt').innerHTML = IC.novo + ' Novo chat'; $('#imagensBt').innerHTML = IC.cri + ' Criações'; $('#configBt').innerHTML = IC.cfg + ' Configurações';
$('#maisBt').innerHTML = IC.plus; $('#micBt').innerHTML = IC.mic; $('#envBt').innerHTML = IC.onda; $('#pensarBt').innerHTML = IC.pensar + '<span>Pensar</span>';

/* ---------------------------------------------------------------- conversa */
function novaConversa() {
  S.conv = { id: 'c' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6), titulo: 'Nova conversa', mensagens: [], novo: true };
  S.anexos = []; sincGerando(); desenharAnexos(); desenharChat(); reanimar($('#chat'), 'troca'); desenharLista(); $('#texto').focus();
}
async function abrirConversa(id) {
  const g = S.gens.get(id);
  if (g) S.conv = g.conv; else { try { S.conv = await api('GET', '/api/conversas/' + id); } catch (e) { aviso(e.message); return; } }
  S.conv.mensagens = S.conv.mensagens || [];
  sincGerando();
  desenharChat(); reanimar($('#chat'), 'troca'); desenharLista(); rolar(true);
}
async function salvarConversa(c = S.conv) {
  if (!c || !c.mensagens.length) return;
  const corpo = { id: c.id, titulo: c.titulo, fixada: !!c.fixada, modelo: c.modelo || '', mensagens: c.mensagens };
  try { await api('PUT', '/api/conversas/' + c.id, corpo); } catch (e) { aviso('Não consegui salvar: ' + e.message); return; }
  const r = S.convs.find(x => x.id === c.id);
  if (r) { r.titulo = c.titulo; r.atualizado = Date.now(); r.mensagens = c.mensagens.length; } else S.convs.unshift({ id: c.id, titulo: c.titulo, atualizado: Date.now(), fixada: !!c.fixada, mensagens: c.mensagens.length });
  desenharLista();
}
function saudacao() {
  const h = new Date().getHours(); const p = h < 5 ? 'Boa madrugada' : h < 12 ? 'Bom dia' : h < 18 ? 'Boa tarde' : 'Boa noite';
  return `${p}, ${S.usuario.nome}. Pronto quando o senhor quiser.`;
}
function desenharChat() {
  const ch = $('#chat');
  if (!S.conv.mensagens.length) {
    ch.innerHTML = `<div id="boasvindas"><h1>${esc(saudacao())}</h1><div class="sugestoes">
      <button data-s="img">${IC.img} Crie uma imagem</button><button data-s="txt">${IC.nome} Escreva ou edite um texto</button><button data-s="arq">${IC.arq} Crie um arquivo (código, planilha, documento)</button></div></div>`;
    return;
  }
  ch.innerHTML = '<div class="col" id="msgs"></div>';
  S.conv.mensagens.forEach((m, i) => $('#msgs').appendChild(noMsg(m, i)));
  const g = S.gens.get(S.conv.id);
  if (g) { const u = $('#msgs').lastElementChild; if (u) { g.no = u; u.innerHTML = corpoIA(g.a, true); g.a._n = contarPalavras(g.a.content); g.a._d = 1e9; } }
  S.conv.mensagens.forEach(m => { if (m.tarefa && m.tarefa.estado === 'andando') acompanhar(S.conv, m); });
}
$('#chat').addEventListener('click', ev => {
  const s = ev.target.closest('[data-s]'); if (!s) return;
  if (s.dataset.s === 'img') return telaImagens();
  const t = { txt: 'Me ajude a escrever: ', arq: 'Crie um arquivo ' }[s.dataset.s]; $('#texto').value = t; $('#texto').focus(); autoAltura();
});
function noMsg(m, i) {
  const d = document.createElement('div'); d.className = 'msg ' + (m.role === 'user' ? 'eu' : 'ia'); d.dataset.i = i;
  if (m.role === 'user') {
    d.innerHTML = (m.images && m.images.length ? `<div class="imgs">${m.images.map(b => `<img src="data:image/jpeg;base64,${b}">`).join('')}</div>` : '') + `<div class="corpo">${esc(m.content)}</div><div class="acoes"><button data-a="copiar" title="Copiar">${IC.copiar}</button><button data-a="apagar" title="Apagar esta mensagem">${IC.lixo}</button></div>`;
  } else {
    d.innerHTML = corpoIA(m) + `<div class="acoes"><button data-a="copiar" title="Copiar">${IC.copiar}</button><button data-a="falar" title="Ouvir">${IC.falar}</button><button data-a="refazer" title="Gerar de novo">${IC.refazer}</button><button data-a="apagar" title="Apagar esta mensagem">${IC.lixo}</button></div>`;
  }
  return d;
}
function corpoIA(m, vivo = false) {
  let h = '';
  if (m.thinking) h += `<details class="pensou" ${vivo && !m.content ? 'open' : ''}><summary>Pensamento</summary><div>${esc(m.thinking)}</div></details>`;
  if (m.erro) h += `<div class="erro-msg"><span class="bola">!</span><div>${esc(m.erro)}<br><button data-a="saude">Ver o que falta instalar</button></div></div>`;
  else if (!m.content && vivo && !m.etapa) h += '<div class="dots3"><i></i><i></i><i></i></div>';
  else h += `<div class="corpo">${fichas(md(m.content))}</div>`;
  if (m.tarefa) h += cartaoTarefa(m);
  if (m.interrompida && !vivo) h += '<div class="intr">resposta interrompida pela sua nova mensagem</div>';
  if (vivo && m.etapa) h += `<div class="etapa"><i></i>${esc(m.etapa)}</div>`;
  if (m.fontes && m.fontes.length) h += `<div class="fontes">${m.fontes.map(f => `<a href="${esc(f.url)}" target="_blank" rel="noopener noreferrer" title="${esc(f.url)}">[${f.n}] ${esc(f.titulo)}</a>`).join('')}</div>`;
  return h;
}
$('#chat').addEventListener('click', async ev => {
  const b = ev.target.closest('[data-a]'); if (!b) return;
  const msg = b.closest('.msg'); const i = msg ? +msg.dataset.i : -1; const m = i >= 0 ? S.conv.mensagens[i] : null;
  switch (b.dataset.a) {
    case 'copiar': b.classList.remove('feito'); void b.offsetWidth; b.classList.add('feito'); navigator.clipboard.writeText(m.content || '').then(() => aviso('Copiado.', 1500)).catch(() => aviso('Não consegui copiar.')); break;
    case 'copiar-cod': { const t = b.closest('.cod').querySelector('code').textContent; navigator.clipboard.writeText(t).then(() => aviso('Código copiado.', 1500)); break; }
    case 'baixar-cod': {
      const t = b.closest('.cod').querySelector('code').textContent; let n = b.dataset.n || 'arquivo.txt';
      try { const r = await api('POST', '/api/salvar-como', { nome: n, texto: t }); if (r.caminho) aviso('Salvo em ' + r.caminho, 6000); } catch (e) { aviso(e.message); } break;
    }
    case 'apagar': if (S.gerando) return; S.conv.mensagens.splice(i, 1); if (!S.conv.mensagens.length && !S.conv.novo) { await api('DELETE', '/api/conversas/' + S.conv.id); S.convs = S.convs.filter(c => c.id !== S.conv.id); } else await salvarConversa(); desenharChat(); break;
    case 'falar': falar(m.content); break;
    case 'refazer': if (S.gerando) return; S.conv.mensagens.splice(i); responder(); break;
    case 'saude': painelSaude(); break;
    case 'tentar': if (S.gens.has(S.conv.id)) return; S.conv.mensagens.splice(i); responder(); break;
    case 'salvar-tarefa': salvarComo(b.dataset.arq); break;
    case 'ver-criacoes': telaCriacoes(); break;
    case 'tok': fichaClique(b); break;
  }
});
function reanimar(el, cls) { if (!el) return; el.classList.remove(cls); void el.offsetWidth; el.classList.add(cls); }
let COLADO = true;
function rolar(forte) {
  const c = $('#chat'); if (!c) return; if (forte) COLADO = true;
  if (!COLADO) return;
  c.scrollTop = c.scrollHeight; requestAnimationFrame(() => { if (COLADO) c.scrollTop = c.scrollHeight; });
}
document.addEventListener('DOMContentLoaded', () => { const c = $('#chat'); if (c) c.addEventListener('scroll', () => { COLADO = c.scrollHeight - c.scrollTop - c.clientHeight < 90; }, { passive: true }); });

/* ---------------------------------------------------------------- texto que aparece palavra por palavra, suave */
function contarPalavras(t) { return (String(t || '').match(/\S+/g) || []).length; }
function cortarPalavras(t, n) {
  const re = /\S+/g; let m, k = 0, fim = 0;
  while ((m = re.exec(t)) && k < n) { k++; fim = m.index + m[0].length; }
  return k >= n && m ? t.slice(0, fim) : t;
}
function envolverNovas(corpo, desde) {
  // embrulha em <span class="pal"> só as palavras que ainda não estavam na tela (elas surgem de transparente até visível)
  let idx = 0; const w = document.createTreeWalker(corpo, NodeFilter.SHOW_TEXT); const nos = []; while (w.nextNode()) nos.push(w.currentNode);
  for (const n of nos) {
    const partes = n.nodeValue.split(/(\s+)/); let mudou = false; const frag = document.createDocumentFragment();
    for (const p of partes) {
      if (!p) continue;
      if (/^\s+$/.test(p)) { frag.appendChild(document.createTextNode(p)); continue; }
      if (idx >= desde) { const sp = document.createElement('span'); sp.className = 'pal'; sp.textContent = p; frag.appendChild(sp); mudou = true; } else frag.appendChild(document.createTextNode(p));
      idx++;
    }
    if (mudou) n.parentNode.replaceChild(frag, n);
  }
  return idx;
}
function contarDom(el) { const c = el.querySelector('.corpo'); return c ? contarPalavras(c.textContent) : 0; }

/* ---------------------------------------------------------------- tarefas pedidas no chat (imagem, vídeo, arquivo, cortes, download) */
function cartaoTarefa(m) {
  const t = m.tarefa; const u = a => '/api/saida/' + encodeURIComponent(a);
  if (t.estado === 'ok' && t.arquivo) {
    const ext = t.arquivo.split('.').pop().toLowerCase();
    const prev = ['png', 'jpg', 'jpeg', 'webp', 'gif'].includes(ext) ? `<img src="${u(t.arquivo)}" class="tprev">` : ['mp4', 'webm'].includes(ext) ? `<video src="${u(t.arquivo)}" class="tprev" controls preload="metadata"></video>` : `<div class="tarq">${IC.arq}<b>${esc(t.arquivo.replace(/^[a-z]+-?[a-z0-9]*-/, '').slice(0, 40))}</b></div>`;
    return `<div class="tcard ok">${prev}<div class="tbt"><a class="bs" href="${u(t.arquivo)}?baixar=1" download>${IC.baixar} Baixar</a><button class="bs" data-a="salvar-tarefa" data-arq="${esc(t.arquivo)}">Salvar em uma pasta…</button><button class="bs" data-a="ver-criacoes">Ver em Criações</button></div></div>`;
  }
  if (t.estado === 'erro') return `<div class="tcard erro"><div><b>Não consegui concluir</b><small>${esc(t.msg || 'Algo deu errado.')} Posso tentar novamente, senhor.</small></div><div class="tbt"><button class="bp" data-a="tentar">Tentar novamente</button></div></div>`;
  return `<div class="tcard andando"><span class="giro"></span><div class="tx"><b>${esc(t.rotulo || 'Produzindo…')}</b><small>${esc(t.msg || 'Na fila…')}</small><div class="barra"><i style="width:${Math.min(99, t.pct || 3)}%"></i></div></div></div>`;
}
function acompanhar(conv, a) {
  const chave = conv.id + ':' + a.tarefa.job; if (S.obs.has(chave)) return; S.obs.add(chave);
  const redesenhar = () => { if (S.conv !== conv) return; const i = conv.mensagens.indexOf(a); const el = $('#msgs') && $(`#msgs .msg[data-i="${i}"]`); if (el) { const n = noMsg(a, i); el.replaceWith(n); } };
  const volta = async () => {
    let it = null; try { const d = await api('GET', '/api/downloads'); it = d.itens.find(x => x.id === a.tarefa.job); } catch (_) { setTimeout(volta, 2000); return; }
    if (!it) { a.tarefa.estado = 'erro'; a.tarefa.msg = 'A produção foi interrompida (o programa foi reiniciado).'; }
    else if (it.estado === 'ok') { a.tarefa.estado = 'ok'; a.tarefa.arquivo = it.arquivo; }
    else if (it.estado === 'erro') { a.tarefa.estado = 'erro'; a.tarefa.msg = it.msg; }
    else { a.tarefa.pct = it.pct; a.tarefa.msg = it.msg; redesenharBarra(conv, a); setTimeout(volta, 900); return; }
    S.obs.delete(chave); redesenhar(); salvarConversa(conv);
    if (a.tarefa.estado === 'ok') aviso('Pronto, senhor: já está no chat e em Criações.', 4000); else aviso('Uma criação falhou. Posso tentar de novo.', 5000);
  };
  setTimeout(volta, 600);
}
function redesenharBarra(conv, a) {
  if (S.conv !== conv) return; const i = conv.mensagens.indexOf(a); const el = $(`#msgs .msg[data-i="${i}"] .tcard .barra i`);
  if (el) { el.style.width = Math.min(99, a.tarefa.pct || 3) + '%'; const s = el.closest('.tx').querySelector('small'); if (s) s.textContent = a.tarefa.msg || ''; } else { const e2 = $(`#msgs .msg[data-i="${i}"]`); if (e2) e2.replaceWith(noMsg(a, i)); }
}
function sincGerando() { S.gerando = !!(S.conv && S.gens.has(S.conv.id)); atualizarBotao(); }

/* ---------------------------------------------------------------- enviar / receber */
function contexto() {
  let tot = 0; const out = [];
  for (let i = S.conv.mensagens.length - 1; i >= 0; i--) {
    const m = S.conv.mensagens[i]; if (m.erro) continue;
    const c = (m.content || '') + (m.interrompida ? '\n\n[o usuário interrompeu esta resposta com uma nova mensagem]' : '');
    tot += c.length; if (tot > 24000 && out.length) break;
    out.unshift({ role: m.role, content: c, images: m.images });
  }
  return out;
}
async function enviar() {
  const t = $('#texto').value.trim();
  const g = S.gens.get(S.conv.id);
  if (g && !t && !S.anexos.length) return pararGeracao();
  if (!t && !S.anexos.length) return;
  let conteudo = t; const imagens = [];
  for (const a of S.anexos) {
    if (a.tipo === 'imagem') imagens.push(a.base64);
    else conteudo += `\n\n[Arquivo anexado: ${a.nome}]\n\`\`\`\n${a.texto}\n\`\`\``;
  }
  const m = { role: 'user', content: conteudo.trim() || '(imagem)', t: Date.now() };
  if (imagens.length) m.images = imagens;
  const nomeAnexo = S.anexos[0] && S.anexos[0].nome;
  $('#texto').value = ''; autoAltura(); S.anexos = []; desenharAnexos();
  // duas mensagens seguidas: a resposta em andamento é interrompida (o que já saiu fica) e a IA responde considerando as duas
  if (g) { g.a.interrompida = true; g.abort.abort(); await g.fim; }
  const conv = S.conv; conv.mensagens.push(m);
  if (conv.mensagens.length === 1) conv.titulo = (t || nomeAnexo || 'Conversa').replace(/\s+/g, ' ').slice(0, 48);
  desenharChat(); { const u = $('#msgs') && $('#msgs').lastElementChild; if (u) u.classList.add('novo'); } rolar(true);
  await salvarConversa(conv);
  responder(conv);
}
async function responder(conv = S.conv) {
  const a = { role: 'assistant', content: '', thinking: '', t: Date.now(), modelo: conv.modelo || S.cfg.modelo || S.modeloPadrao };
  conv.mensagens.push(a);
  const g = { conv, a, abort: new AbortController(), no: null, raf: 0 }; g.fim = new Promise(r => g.resolver = r);
  S.gens.set(conv.id, g); desenharLista(); const aqui = () => S.conv === conv;
  if (aqui()) { sincGerando(); if (!$('#msgs')) { $('#chat').innerHTML = '<div class="col" id="msgs"></div>'; conv.mensagens.slice(0, -1).forEach((m, i) => $('#msgs').appendChild(noMsg(m, i))); }
    const no = document.createElement('div'); no.className = 'msg ia novo'; no.dataset.i = conv.mensagens.length - 1; no.innerHTML = corpoIA(a, true); $('#msgs').appendChild(no); g.no = no; rolar(true); }
  const viva = () => g.no && document.body.contains(g.no) && aqui();
  a._n = 0; a._d = 0;
  const desenhar = () => {
    if (!viva()) return;
    const vista = { ...a, content: cortarPalavras(a.content, a._n) };
    g.no.innerHTML = corpoIA(vista, true);
    const c = g.no.querySelector('.corpo'); if (c) a._d = envolverNovas(c, a._d);
    rolar(false);
  };
  const passo = () => {
    g.raf = 0; const tot = contarPalavras(a.content);
    if (!viva()) { a._n = tot; return; }
    if (a._n < tot) a._n += Math.max(1, Math.ceil((tot - a._n) / 9));
    desenhar(); if (a._n < tot) g.raf = requestAnimationFrame(passo);
  };
  const pintar = () => { if (!g.raf) g.raf = requestAnimationFrame(passo); };
  try {
    const corpo = { messages: contexto().slice(0, -1) };
    if (conv.modelo || S.cfg.modelo) corpo.model = conv.modelo || S.cfg.modelo;
    if (S.cfg.pensar) corpo.think = true;
    if (S.web) corpo.web = true;
    if (S.fluxo && S.fluxo.length >= 2) corpo.fluxo = S.fluxo;
    const r = await fetch('/api/chat', { method: 'POST', headers: H(), body: JSON.stringify(corpo), signal: g.abort.signal });
    if (!r.ok) { let j = null; try { j = await r.json(); } catch (_) {} throw new Error((j && j.erro) || 'Erro ' + r.status); }
    const rd = r.body.getReader(), dec = new TextDecoder(); let buf = '';
    for (;;) {
      const { done, value } = await rd.read(); if (done) break;
      buf += dec.decode(value, { stream: true }); let k;
      while ((k = buf.indexOf('\n')) >= 0) {
        const l = buf.slice(0, k).trim(); buf = buf.slice(k + 1); if (!l) continue;
        let j; try { j = JSON.parse(l); } catch (_) { continue; }
        if (j.error) throw new Error(j.error);
        if ('etapa' in j) { a.etapa = j.etapa; if (viva()) g.no.innerHTML = corpoIA({ ...a, content: cortarPalavras(a.content, a._n) }, true); }
        if (j.fontes) a.fontes = j.fontes;
        if (j.tarefa) { a.tarefa = { job: j.tarefa, tipo: j.tipo, estado: 'andando', rotulo: j.rotulo || '' }; }
        if (j.message) { if (j.message.content) a.content += j.message.content; if (j.message.thinking) a.thinking += j.message.thinking; pintar(); }
      }
    }
    if (!a.content.trim() && !a.thinking && !a.tarefa) a.content = '(resposta vazia)';
  } catch (e) {
    if (e.name === 'AbortError') { if (!a.content && !a.thinking && !a.tarefa) { const k = conv.mensagens.indexOf(a); if (k >= 0) conv.mensagens.splice(k, 1); } else a.interrompida = !!a.interrompida; }
    else { a.erro = e.message || String(e); atualizarSaude(); }
  }
  // deixa o texto terminar de aparecer (rápido) antes de fechar
  for (let i = 0; i < 100 && viva() && a._n < contarPalavras(a.content); i++) await new Promise(r => setTimeout(r, 30));
  S.gens.delete(conv.id); desenharLista(); if (aqui()) sincGerando();
  g.resolver();
  if (!conv.mensagens.includes(a)) { if (aqui()) desenharChat(); return; }
  delete conv.novo; await salvarConversa(conv);
  if (a.tarefa) acompanhar(conv, a);
  if (aqui()) { const idx = conv.mensagens.indexOf(a); const n2 = noMsg(a, idx); if (g.no && document.body.contains(g.no)) g.no.replaceWith(n2); else desenharChat(); rolar(false); }
  if (aqui() && !a.erro && (S.vozModo || S.cfg.voz_responder) && a.content) falar(a.content);
}
function pararGeracao() { const g = S.conv && S.gens.get(S.conv.id); if (g) g.abort.abort(); fetch('/api/voz/calar', { method: 'POST', headers: H(), body: '{}' }).catch(() => {}); }
function atualizarBotao() {
  const b = $('#envBt'); b.classList.toggle('parar', S.gerando);
  b.dataset.m = S.gerando ? 'parar' : ($('#texto').value.trim() || S.anexos.length ? 'seta' : 'onda');
  b.innerHTML = S.gerando ? IC.parar : ($('#texto').value.trim() || S.anexos.length ? IC.seta : IC.onda);
  b.title = S.gerando ? 'Parar' : ($('#texto').value.trim() || S.anexos.length ? 'Enviar' : 'Modo voz: falar e ouvir a resposta');
}
function atualizarPensar() { $('#pensarBt').classList.toggle('on', !!S.cfg.pensar); }
function autoAltura() { const t = $('#texto'); t.style.height = 'auto'; t.style.height = Math.min(t.scrollHeight + 2, 200) + 'px'; t.style.overflowY = t.scrollHeight > 200 ? 'auto' : 'hidden'; atualizarBotao(); }
$('#texto').addEventListener('input', autoAltura);
$('#texto').addEventListener('keydown', ev => { if (ev.key === 'Enter' && !ev.shiftKey && !ev.isComposing) { ev.preventDefault(); enviar(); } if (ev.key === 'Escape' && S.gerando) pararGeracao(); });
$('#envBt').onclick = () => {
  if (S.gerando) return pararGeracao();
  if ($('#texto').value.trim() || S.anexos.length) return enviar();
  S.vozModo = true; aviso('Modo voz: fale, toque no microfone para terminar e eu respondo em voz alta.', 5000); alternarGravacao(true);
};
$('#pensarBt').onclick = () => { salvarCfg({ pensar: !S.cfg.pensar }); aviso(S.cfg.pensar ? 'Pensar ligado: a IA raciocina antes de responder.' : 'Pensar desligado.', 2500); };

/* ---------------------------------------------------------------- voz (escuta contínua: só fala humana vira texto) */
let vozTimer = null, vozEnvio = null;
function vozNivel(n, fala) { const m = $('#micBt'); m.style.setProperty('--n', n.toFixed(2)); m.classList.toggle('fala', !!fala); }
async function vozSonda() {
  if (!S.gravando) return;
  let e; try { e = await api('GET', '/api/voz/estado'); } catch (_) { return; }
  vozNivel(e.nivel || 0, e.fala);
  if (e.erro) { aviso(e.erro, 6000); }
  if (e.textos && e.textos.length) {
    $('#texto').value = ($('#texto').value + ' ' + e.textos.join(' ')).trim(); autoAltura();
    clearTimeout(vozEnvio);
  }
  if (S.vozModo && $('#texto').value.trim() && !e.fala && !e.pendentes && !e.ia_falando && !S.gerando) {
    clearTimeout(vozEnvio); vozEnvio = setTimeout(() => { if (S.gravando && S.vozModo && $('#texto').value.trim() && !S.gerando) enviar(); }, 700);
  }
  const st = e.pendentes ? 'Transcrevendo…' : e.fala ? 'Ouvindo sua fala…' : e.ia_falando ? 'Respondendo…' : 'Aguardando você falar…';
  const l = $('#vozSt'); if (l) l.textContent = st;
}
async function alternarGravacao(deModoVoz) {
  if (S.gravando) {
    S.gravando = false; clearInterval(vozTimer); clearTimeout(vozEnvio); $('#micBt').classList.remove('grav', 'fala'); vozNivel(0, false);
    const l = $('#vozSt'); if (l) l.remove();
    aviso('Finalizando a transcrição…', 60000);
    try {
      const r = await api('POST', '/api/voz/parar'); $('#aviso').classList.remove('on');
      if (r.texto) { $('#texto').value = ($('#texto').value + ' ' + r.texto).trim(); autoAltura(); if (S.vozModo) enviar(); else $('#texto').focus(); }
    } catch (e) { S.vozModo = false; aviso(e.message, 7000); atualizarSaude(); }
    S.vozModo = false;
    return;
  }
  try {
    await api('POST', '/api/voz/gravar'); S.gravando = true; $('#micBt').classList.add('grav');
    const l = document.createElement('div'); l.id = 'vozSt'; l.className = 'vozst'; l.textContent = 'Aguardando você falar…'; $('#compor').prepend(l);
    clearInterval(vozTimer); vozTimer = setInterval(vozSonda, 250);
    aviso(S.vozModo ? 'Modo voz: fale quando quiser; eu envio e respondo em voz alta. Toque no microfone para parar.' : 'Ouvindo: só falas humanas são transcritas, logo após uma pausa. Toque no microfone para parar.', 5000);
  } catch (e) { S.vozModo = false; aviso(e.message, 7000); }
}
$('#micBt').onclick = () => { if (!S.gravando) S.vozModo = false; alternarGravacao(false); };
async function falar(t) {
  const txt = textoLimpo(t); if (!txt) return;
  try { await api('POST', '/api/voz/falar', { texto: txt }); } catch (e) { aviso(e.message, 6000); }
}

/* ---------------------------------------------------------------- anexos / arrastar */
function desenharAnexos() {
  $('#anexos').innerHTML = S.anexos.map((a, i) => `<span class="chip">${a.tipo === 'imagem' ? `<img src="data:image/jpeg;base64,${a.base64}">` : IC.arq}<span>${esc(a.nome)}</span><button data-x="${i}">${IC.x}</button></span>`).join('');
  atualizarBotao();
}
$('#anexos').onclick = ev => { const b = ev.target.closest('[data-x]'); if (b) { S.anexos.splice(+b.dataset.x, 1); desenharAnexos(); } };
$('#maisBt').onclick = ev => menu(ev.currentTarget, [
  { ic: IC.clip, t: 'Enviar arquivos', s: 'Imagens e documentos de texto', f: () => $('#arq').click() },
  { ic: IC.img, t: 'Criar imagem', s: 'Abre o gerador de imagens', f: () => telaImagens() },
  { ic: IC.cfg, t: 'Aplicar atualização .karthus', s: 'Escolher o arquivo', f: () => $('#arqUpd').click() },
]);
$('#arq').onchange = e => { adicionarArquivos([...e.target.files]); e.target.value = ''; };
$('#arqUpd').onchange = e => { if (e.target.files[0]) atualizacao(e.target.files[0]); e.target.value = ''; };
const EXT_TEXTO = /\.(txt|md|json|csv|tsv|py|js|ts|jsx|tsx|rs|go|c|h|cpp|hpp|cs|java|kt|lua|rb|php|sh|bat|cmd|ps1|html|htm|css|scss|xml|yml|yaml|toml|ini|cfg|conf|log|sql|r|swift|dart|vue|svelte|gradle|properties|env|tf|asm)$/i;
async function reduzirImagem(f) {
  const url = URL.createObjectURL(f);
  try {
    const im = await new Promise((ok, no) => { const i = new Image(); i.onload = () => ok(i); i.onerror = no; i.src = url; });
    const k = Math.min(1, 1280 / Math.max(im.width, im.height)); const c = document.createElement('canvas');
    c.width = Math.round(im.width * k); c.height = Math.round(im.height * k); c.getContext('2d').drawImage(im, 0, 0, c.width, c.height);
    return c.toDataURL('image/jpeg', 0.85).split(',')[1];
  } finally { URL.revokeObjectURL(url); }
}
async function adicionarArquivos(fs) {
  for (const f of fs) {
    if (/\.(karthus|hirelod)$/i.test(f.name)) { await atualizacao(f); continue; }
    try {
      if (/^image\//.test(f.type)) { S.anexos.push({ tipo: 'imagem', nome: f.name, base64: await reduzirImagem(f) }); }
      else if (EXT_TEXTO.test(f.name) || /^text\//.test(f.type)) {
        if (f.size > 400 * 1024) { aviso(f.name + ' é grande demais (limite 400 KB de texto).'); continue; }
        S.anexos.push({ tipo: 'texto', nome: f.name, texto: await f.text() });
      } else aviso(`${f.name}: tipo não suportado no chat. Textos, código e imagens funcionam.`, 6000);
    } catch (e) { aviso('Não consegui ler ' + f.name); }
  }
  desenharAnexos();
}
let dragN = 0;
addEventListener('dragenter', e => { e.preventDefault(); dragN++; $('#solte').classList.add('on'); });
addEventListener('dragleave', e => { e.preventDefault(); if (--dragN <= 0) { dragN = 0; $('#solte').classList.remove('on'); } });
addEventListener('dragover', e => e.preventDefault());
addEventListener('drop', e => { e.preventDefault(); dragN = 0; $('#solte').classList.remove('on'); if (e.dataTransfer && e.dataTransfer.files.length) { if (!S.usuario) return; const fs = [...e.dataTransfer.files]; const mot = fs.filter(x => tipoMotor(x.name)); if (mot.length) importarMotores(mot); adicionarArquivos(fs.filter(x => !mot.includes(x))); } });
addEventListener('paste', e => { const fs = [...(e.clipboardData ? e.clipboardData.files : [])].filter(f => /^image\//.test(f.type)); if (fs.length) { e.preventDefault(); adicionarArquivos(fs); } });

/* ---------------------------------------------------------------- modelos */
async function carregarModelos() {
  try { S.modelos = await api('GET', '/api/modelos'); } catch (_) { S.modelos = []; }
  try { S.global = await api('GET', '/api/global'); S.modeloPadrao = S.global.modelo_padrao; } catch (_) {}
  $('#modeloNome').textContent = S.cfg.modelo || S.modeloPadrao || 'Modelo';
}
$('#modeloBt').onclick = ev => {
  const itens0 = S.modelos.length ? S.modelos.map(m => ({ t: m.nome, s: (S.cfg.modelo || S.modeloPadrao) === m.nome ? 'em uso' : '', f: () => { salvarCfg({ modelo: m.nome }); $('#modeloNome').textContent = m.nome; } })) : [{ t: 'Nenhum modelo encontrado', s: 'Veja o ícone vermelho', f: () => painelSaude() }];
  menu(ev.currentTarget, [...itens0, { t: 'Catálogo de IAs…', s: 'baixar outras', f: () => catalogoIA() }]);
};

/* ---------------------------------------------------------------- saúde (ícone vermelho) */
async function atualizarSaude() {
  try { S.saude = await api('GET', '/api/saude'); } catch (_) { S.saude = null; }
  const b = $('#saudeBt'); const n = S.saude && S.saude.problemas ? S.saude.problemas.length : 0;
  b.classList.toggle('on', n > 0); b.title = n ? `${n} item(ns) para instalar ou corrigir` : '';
}
$('#saudeBt').onclick = () => painelSaude();
function janela(titulo, html, largura) {
  $$('.fundo').forEach(x => x.remove());
  const f = document.createElement('div'); f.className = 'fundo';
  f.innerHTML = `<div class="janela" ${largura ? `style="width:${largura}"` : ''}><header><span>${esc(titulo)}</span><button class="bt rb" data-fechar>${IC.x}</button></header>${html}</div>`;
  document.body.appendChild(f);
  f.addEventListener('mousedown', e => { if (e.target === f) f.remove(); });
  $('[data-fechar]', f).onclick = () => f.remove();
  return f;
}
async function painelSaude(aba0 = 'itens') {
  await atualizarSaude();
  let aba = aba0;
  const f = janela('Instalação e downloads', '<div class="abas-h"><button data-t="itens">Instalação</button><button data-t="dl">Downloads <span class="selo" id="seloDl"></span></button><button data-t="cr">Criações <span class="selo" id="seloCr"></span></button></div><div class="painel" id="sp"></div>', '680px');
  let cont = true;
  const marcar = () => $$('.abas-h button', f).forEach(b => b.classList.toggle('on', b.dataset.t === aba));
  $$('.abas-h button', f).forEach(b => b.onclick = () => { if (aba === b.dataset.t) return; aba = b.dataset.t; marcar(); $('#sp', f).innerHTML = ''; ult = ''; reanimar($('#sp', f), 'troca'); pintar(true); });
  marcar();
  let ult = '';
  const pintar = async (forca) => {
    if (!document.body.contains(f)) { cont = false; return; }
    if (aba === 'dl' || aba === 'cr') { const d = await dlPuxar(); const ct = c => d.itens.filter(x => (x.cat === 'criacao') === c && (x.estado === 'ativo' || x.estado === 'fila')).length; $('#seloDl', f).textContent = ct(false) || ''; $('#seloCr', f).textContent = ct(true) || ''; const h = dlHtml(d, aba); if (h !== ult || forca) { $('#sp', f).innerHTML = h; ult = h; dlLigar($('#sp', f), () => pintar(true)); } return; }
    ult = '';
    let c; try { c = await api('GET', '/api/carga'); } catch (_) { return; }
    const d = window.DL || { itens: [] }; $('#seloDl', f).textContent = d.itens.filter(x => x.estado === 'ativo' || x.estado === 'fila').length || '';
    const ocupado = new Set(d.itens.filter(x => x.estado === 'ativo' || x.estado === 'fila').map(x => x.item));
    $('#sp', f).innerHTML = `<div style="display:flex;gap:10px;margin-bottom:8px;flex-wrap:wrap"><button class="bp" id="rev">Verificar de novo</button><button class="bs" id="vcat">Catálogo de IAs</button><span style="color:var(--fg2);align-self:center">${c.rodando ? 'Verificando… ' + Math.round(c.pct) + '%' : 'Pasta de instaladores: instaladores/'}</span></div>` +
      (c.itens || []).map(i => { const inst = i.estado === 'instalando' || ocupado.has(i.id); return `<div class="comp"><span class="bola ${inst ? 'instalando' : i.estado}"></span><div class="txt"><b>${esc(i.nome)}</b>${i.critico ? ' <small style="display:inline">(necessário)</small>' : ''}<small>${esc(i.msg || (i.estado === 'espera' ? 'aguardando' : ''))}</small>${i.dica && i.estado !== 'ok' && !inst ? `<small>${esc(i.dica)}</small>` : ''}${inst ? `<div class="barra"><i style="width:${Math.max(3, i.pct || 0)}%"></i></div>` : ''}</div>${i.instalavel && !inst && i.estado !== 'ok' ? `<button class="bp" data-i="${i.id}">Instalar</button>` : ''}${inst ? '<span class="girando"></span>' : ''}</div>`; }).join('');
    $('#rev', f).onclick = async () => { await api('POST', '/api/carga/verificar'); };
    $('#vcat', f).onclick = () => { f.remove(); catalogoIA(); };
    $$('[data-i]', f).forEach(b => b.onclick = async () => { if (b.disabled) return; b.disabled = true; b.textContent = 'Instalando…'; try { await api('POST', '/api/instalar/' + b.dataset.i); await dlPuxar(); aviso('Instalando em segundo plano. Veja na aba Downloads.', 4000); pintar(true); } catch (e) { aviso(e.message, 8000); b.disabled = false; b.textContent = 'Instalar'; } });
    atualizarSaude();
  };
  (async () => { while (cont) { await pintar(); await new Promise(r => setTimeout(r, aba === 'dl' || aba === 'cr' ? 800 : 700)); } })();
}

/* ---------------------------------------------------------------- atualização .karthus */
async function atualizacao(f) {
  let info; try { info = await api('POST', '/api/atualizar/inspecionar', await f.arrayBuffer(), { cru: true }); } catch (e) { aviso('Atualização recusada: ' + e.message, 9000); return; }
  const w = janela('Atualização ' + info.versao, `<div class="painel"><p><b>${esc(info.nome)}</b></p>${(info.notas || []).length ? '<ul>' + info.notas.map(n => `<li>${esc(n)}</li>`).join('') + '</ul>' : ''}<p style="color:var(--fg2)">${info.arquivos.length} arquivo(s). O que for trocado fica guardado em dados/backups.</p><div style="display:flex;gap:10px"><button class="bp" id="aplicar">Aplicar</button><button class="bs" data-fechar2>Cancelar</button></div></div>`, '560px');
  $('[data-fechar2]', w).onclick = () => w.remove();
  $('#aplicar', w).onclick = async () => {
    try { const r = await api('POST', '/api/atualizar/aplicar', { token: info.token }); w.remove(); aviso(r.reiniciar ? 'Atualizado. O Karthus vai reiniciar.' : 'Atualizado. Recarregando…', 5000); setTimeout(() => r.reiniciar ? api('POST', '/api/sair').catch(() => {}) : location.reload(), 1200); }
    catch (e) { aviso(e.message, 9000); }
  };
}

/* ---------------------------------------------------------------- imagens */
/* ---------------------------------------------------------------- motores (Whisper, imagem, vídeo) */
function tipoMotor(nome) {
  const n = nome.toLowerCase();
  if (/^ggml-.*\.bin$/.test(n)) return 'whisper';
  if (n === 'sd.exe' || /\.(safetensors|gguf|ckpt)$/.test(n)) return 'imagem';
  if (n === 'ffmpeg.exe') return 'video';
  return null;
}
function enviarMotor(tipo, file, prog) {
  return new Promise((ok, err) => {
    const x = new XMLHttpRequest();
    x.open('POST', `/api/motores/adicionar?tipo=${encodeURIComponent(tipo)}&nome=${encodeURIComponent(file.name)}`);
    x.setRequestHeader('X-Hirelod', '1');
    x.upload.onprogress = e => { if (e.lengthComputable && prog) prog(e.loaded, e.total); };
    x.onload = () => { let j = {}; try { j = JSON.parse(x.responseText); } catch (_) {} x.status < 300 ? ok(j) : err(new Error(j.erro || ('Erro ' + x.status))); };
    x.onerror = () => err(new Error('O envio falhou.'));
    x.send(file);
  });
}
async function importarMotores(fs, tipoPadrao) {
  let n = 0;
  for (const f of fs) {
    const tipo = tipoMotor(f.name) || (/\.dll$/i.test(f.name) ? tipoPadrao : null);
    if (!tipo) { aviso(`${f.name}: não reconheci como motor. Use ggml-*.bin, sd.exe, um modelo .safetensors/.gguf ou ffmpeg.exe.`, 8000); continue; }
    try {
      aviso(`Adicionando ${f.name}…`, 60000);
      await enviarMotor(tipo, f, (a, b) => aviso(`Adicionando ${f.name}: ${Math.round(a * 100 / b)}%`, 60000));
      n++;
    } catch (e) { aviso(`${f.name}: ${e.message}`, 9000); }
  }
  if (n) { aviso(n + ' arquivo(s) adicionado(s). Conferindo…', 3500); atualizarSaude(); }
  return n;
}

/* ---------------------------------------------------------------- criar imagem / vídeo */
function telaImagens(aba = 'imagem') {
  const lista = JSON.parse(localStorage.getItem('hl.img.' + S.usuario.id) || '[]');
  const vids = JSON.parse(localStorage.getItem('hl.vid.' + S.usuario.id) || '[]');
  const f = janela('Criar', `<div class="corpo2" style="flex-direction:column"><div class="abas-h">${[['imagem', 'Imagem'], ['video', 'Vídeo'], ['motores', 'Motores']].map(([k, t]) => `<button data-k="${k}">${t}</button>`).join('')}</div><div class="painel" id="cp2"></div></div>`, '900px');
  const P = $('#cp2', f);
  const salvar = async nome => { try { const r = await api('POST', '/api/salvar-como', { nome, saida: nome }); if (r.caminho) aviso('Salvo em ' + r.caminho, 6000); } catch (er) { aviso(er.message); } };
  async function rodar(rota, corpo, btn, barra, st, ao) {
    try {
      const { job } = await api('POST', rota, corpo);
      btn.disabled = true; barra.style.display = '';
      for (;;) {
        await new Promise(r => setTimeout(r, 700));
        if (!document.body.contains(f)) return;
        const j = await api('GET', '/api/imagem/' + job); $('i', barra).style.width = j.pct + '%'; st.textContent = j.msg;
        if (j.estado === 'pronto') { ao(j); break; }
        if (j.estado === 'erro') { aviso(j.msg, 9000); atualizarSaude(); break; }
      }
    } catch (e) { aviso(e.message, 9000); atualizarSaude(); }
    if (document.body.contains(f)) { btn.disabled = false; barra.style.display = 'none'; }
  }
  const abas = {
    imagem() {
      P.innerHTML = `<textarea class="cx" id="ip" placeholder="Descreva a imagem (em inglês costuma dar melhor resultado)"></textarea>
        <div style="display:flex;gap:10px;margin:10px 0;flex-wrap:wrap;align-items:center"><select id="is"><option value="512x512">512 × 512</option><option value="768x512">768 × 512</option><option value="512x768">512 × 768</option><option value="768x768">768 × 768</option></select>
        <label>Passos <input type="text" id="ipas" value="20" size="3"></label><button class="bp" id="ig">Gerar</button><span id="ist" style="color:var(--fg2)"></span></div>
        <div class="barra" id="ib" style="display:none"><i></i></div><div class="imggen" id="ig2"></div>`;
      const gal = () => { $('#ig2', P).innerHTML = lista.map(a => `<figure><img src="/api/saida/${esc(a.arq)}"><figcaption><span title="${esc(a.p)}">${esc(a.p.slice(0, 32))}</span><button data-s="${esc(a.arq)}" class="bs">Salvar como…</button></figcaption></figure>`).join(''); };
      gal();
      $('#ig2', P).onclick = e => { const b = e.target.closest('[data-s]'); if (b) salvar(b.dataset.s); };
      $('#ig', P).onclick = () => {
        const p = $('#ip', P).value.trim(); if (!p) return; const [w, h] = $('#is', P).value.split('x').map(Number);
        rodar('/api/imagem', { prompt: p, largura: w, altura: h, passos: +$('#ipas', P).value || 20 }, $('#ig', P), $('#ib', P), $('#ist', P), j => { lista.unshift({ arq: j.arquivo, p }); localStorage.setItem('hl.img.' + S.usuario.id, JSON.stringify(lista.slice(0, 60))); gal(); });
      };
    },
    video() {
      P.innerHTML = `<textarea class="cx" id="vp" placeholder="Descreva o vídeo (cada cena vira uma imagem com movimento suave)"></textarea>
        <div style="display:flex;gap:10px;margin:10px 0;flex-wrap:wrap;align-items:center"><label>Cenas <select id="vc"><option>3</option><option selected>4</option><option>6</option><option>8</option></select></label>
        <label>Segundos por cena <select id="vs"><option>2</option><option selected>3</option><option>4</option><option>6</option></select></label>
        <select id="vt"><option value="512x512">512 × 512</option><option value="768x512">768 × 512 (paisagem)</option><option value="512x768">512 × 768 (retrato)</option></select>
        <button class="bp" id="vg">Gerar vídeo</button><span id="vst" style="color:var(--fg2)"></span></div>
        <div class="barra" id="vb" style="display:none"><i></i></div><div class="imggen" id="vg2"></div>`;
      const gal = () => { $('#vg2', P).innerHTML = vids.map(a => `<figure><video src="/api/saida/${esc(a.arq)}" controls preload="metadata" style="width:100%;display:block"></video><figcaption><span title="${esc(a.p)}">${esc(a.p.slice(0, 32))}</span><button data-s="${esc(a.arq)}" class="bs">Salvar como…</button></figcaption></figure>`).join(''); };
      gal();
      $('#vg2', P).onclick = e => { const b = e.target.closest('[data-s]'); if (b) salvar(b.dataset.s); };
      $('#vg', P).onclick = () => {
        const p = $('#vp', P).value.trim(); if (!p) return; const [w, h] = $('#vt', P).value.split('x').map(Number);
        rodar('/api/video', { prompt: p, cenas: +$('#vc', P).value, segundos: +$('#vs', P).value, largura: w, altura: h }, $('#vg', P), $('#vb', P), $('#vst', P), j => { vids.unshift({ arq: j.arquivo, p }); localStorage.setItem('hl.vid.' + S.usuario.id, JSON.stringify(vids.slice(0, 30))); gal(); });
      };
    },
    async motores() {
      const ITENS = [
        ['voz_ouvir', 'whisper', 'Voz: ouvir (Whisper)', 'Escolha o arquivo ggml-base.bin (ou ggml-small.bin).', '.bin'],
        ['imagem', 'imagem', 'Gerador de imagens', 'Escolha o sd.exe (com as DLLs ao lado, se houver) e um modelo .safetensors ou .gguf.', '.exe,.dll,.safetensors,.gguf,.ckpt'],
        ['video', 'video', 'Gerador de vídeos', 'Escolha o ffmpeg.exe. O vídeo também usa o gerador de imagens.', '.exe,.dll'],
      ];
      const pintar = async () => {
        let c = { itens: [] }; try { c = await api('GET', '/api/carga'); } catch (_) {}
        P.innerHTML = `<p style="color:var(--fg2);margin-top:0">Adicione aqui os arquivos dos motores. Eles vão direto para a pasta certa; nada é baixado da internet. Dá para arrastar os arquivos para dentro desta janela.</p>` +
          ITENS.map(([id, tipo, nome, dica, ext]) => { const i = (c.itens || []).find(x => x.id === id) || {}; return `<div class="comp mot" data-t="${tipo}"><span class="bola ${i.estado || ''}"></span><div class="txt"><b>${nome}</b><small>${esc(i.msg || '')}</small><small>${dica}</small></div><button class="bs" data-add="${tipo}" data-ext="${ext}">${IC.plus} Adicionar arquivo…</button></div>`; }).join('') +
          `<input type="file" id="arqM" multiple hidden>`;
        $$('[data-add]', P).forEach(b => b.onclick = () => { const inp = $('#arqM', P); inp.accept = b.dataset.ext; inp.onchange = async () => { await importarMotores([...inp.files], b.dataset.add); pintar(); }; inp.click(); });
      };
      f.addEventListener('dragover', e => e.preventDefault());
      f.addEventListener('drop', async e => { e.preventDefault(); e.stopPropagation(); await importarMotores([...e.dataTransfer.files], null); if ($('.mot', P)) pintar(); });
      await pintar();
    },
  };
  const ir = k => { $$('.abas-h button', f).forEach(b => b.classList.toggle('on', b.dataset.k === k)); reanimar(P, 'troca'); abas[k](); };
  $$('.abas-h button', f).forEach(b => b.onclick = () => ir(b.dataset.k));
  ir(aba);
}

/* ---------------------------------------------------------------- configurações */
async function configuracoes(aba = 'geral') {
  const f = janela('Configurações', `<div class="corpo2"><div class="abas">${[['geral', 'Geral'], ['ia', 'IA e modelos'], ['voz', 'Voz'], ['usuario', 'Usuário'], ['inst', 'Instalação'], ['sobre', 'Sobre']].map(([k, t]) => `<button data-k="${k}" class="${k === aba ? 'on' : ''}">${t}</button>`).join('')}</div><div class="painel" id="cp"></div></div>`);
  const seg = (nome, ops, atual) => `<span class="sel" data-sel="${nome}">${ops.map(([v, t]) => `<button data-v="${v}" class="${v === atual ? 'on' : ''}">${t}</button>`).join('')}</span>`;
  const mostrar = async k => {
    $$('.abas button', f).forEach(b => b.classList.toggle('on', b.dataset.k === k)); const p = $('#cp', f); reanimar(p, 'troca');
    if (k === 'geral') p.innerHTML = `<div class="linha2"><div>Tema<small>Claro, escuro ou igual ao Windows</small></div>${seg('tema', [['claro', 'Claro'], ['escuro', 'Escuro'], ['auto', 'Automático']], S.cfg.tema)}</div>
      <div class="linha2"><div>Tamanho da interface<small>Automático acompanha o tamanho da tela</small></div>${seg('escala', [['auto', 'Automático'], ['pequena', 'Pequena'], ['normal', 'Normal'], ['grande', 'Grande'], ['enorme', 'Enorme']], S.cfg.escala || 'auto')}</div>
      <div class="linha2"><div>Cor de destaque</div>${seg('acento', [['branco', 'Branco'], ['vermelho', 'Vermelho'], ['azul', 'Azul']], S.cfg.acento || 'branco')}</div>
      <div class="linha2"><div>Instruções para a IA<small>Como o senhor quer que ela responda (opcional)</small></div></div><textarea class="cx" id="persona" placeholder="Ex.: Seja breve. Explique como se eu fosse iniciante.">${esc(S.cfg.persona || '')}</textarea><div style="margin-top:8px"><button class="bp" id="salvaPersona">Salvar instruções</button></div>`;
    if (k === 'ia') {
      await carregarModelos();
      p.innerHTML = `<div class="linha2"><div>Modelo padrão deste computador<small>Usado por todos os usuários que não escolherem outro</small></div><select id="mp">${S.modelos.map(m => `<option ${m.nome === S.modeloPadrao ? 'selected' : ''}>${esc(m.nome)}</option>`).join('')}</select></div>
      <div class="linha2"><div>Meu modelo<small>Só para ${esc(S.usuario.nome)}</small></div><select id="mu"><option value="">(usar o padrão)</option>${S.modelos.map(m => `<option ${m.nome === S.cfg.modelo ? 'selected' : ''}>${esc(m.nome)}</option>`).join('')}</select></div>
      <div class="linha2"><div>Pensar antes de responder<small>Mais lento, mas costuma errar menos</small></div>${seg('pensar', [['true', 'Ligado'], ['false', 'Desligado']], String(!!S.cfg.pensar))}</div>`;
      $('#mp', f).onchange = async e => { await api('PUT', '/api/global', { modelo_padrao: e.target.value }); S.modeloPadrao = e.target.value; carregarModelos(); };
      $('#mu', f).onchange = e => { salvarCfg({ modelo: e.target.value }); $('#modeloNome').textContent = e.target.value || S.modeloPadrao; };
    }
    if (k === 'voz') {
      p.innerHTML = `<div class="linha2"><div>Ler as respostas em voz alta<small>Usa a voz do Windows</small></div>${seg('voz_responder', [['true', 'Ligado'], ['false', 'Desligado']], String(!!S.cfg.voz_responder))}</div>
      <div class="linha2"><div>Voz<small id="vinfo">Procurando vozes…</small></div><select id="vz"></select></div><div style="margin-top:10px"><button class="bs" id="vt">Testar a voz</button></div>`;
      let vs = []; try { vs = await api('GET', '/api/voz/vozes'); } catch (_) {}
      $('#vz', f).innerHTML = '<option value="">(automática, português)</option>' + vs.map(v => `<option value="${esc(v.nome)}" ${v.nome === S.cfg.voz_nome ? 'selected' : ''}>${esc(v.rotulo || v.nome)} — ${esc(v.idioma)}</option>`).join('');
      try { const cat = await api('GET', '/api/voz/catalogo'); p.insertAdjacentHTML('beforeend', `<div class="sec" style="margin:14px 0 4px;padding:0">Mais vozes em português (offline, Piper)</div>` + cat.map(c => `<div class="comp"><div class="txt"><b>${esc(c.nome)}</b></div>${c.instalada ? '<span class="selo2">instalada</span>' : c.baixando ? '<span class="girando"></span>' : `<button class="bs" data-bv="${esc(c.id)}">Baixar</button>`}</div>`).join('') + '<p style="color:var(--fg2)">A escuta (voz para texto) usa somente o Whisper. Estas vozes só servem para a IA falar com o senhor.</p>');
        $$('[data-bv]', p).forEach(b => b.onclick = async () => { b.disabled = true; try { await api('POST', '/api/voz/baixar', { id: b.dataset.bv }); await dlPuxar(); aviso('Baixando a voz em segundo plano. Veja o ícone de downloads.', 5000); } catch (e) { aviso(e.message, 8000); b.disabled = false; } }); } catch (_) {}
      $('#vinfo', f).textContent = vs.length ? vs.length + ' voz(es) no Windows' : 'Nenhuma voz encontrada';
      $('#vz', f).onchange = e => salvarCfg({ voz_nome: e.target.value });
      $('#vt', f).onclick = () => falar('Sim, senhor. A minha voz está funcionando.');
    }
    if (k === 'usuario') {
      p.innerHTML = `<div style="display:flex;gap:14px;align-items:center;margin-bottom:14px">${avatar(S.usuario)}<div><b>${esc(S.usuario.nome)}</b></div></div>
      <div class="linha2"><div>Nome</div><span><input type="text" id="un" value="${esc(S.usuario.nome)}" maxlength="40"> <button class="bp" id="usn">Salvar</button></span></div>
      <div class="linha2"><div>Cor do perfil</div><span class="cores">${CORES.map(c => `<button data-c="${c}" style="background:${c}" class="${c === S.usuario.cor ? 'on' : ''}"></button>`).join('')}</span></div>
      <div class="linha2"><div>Trocar de usuário ou adicionar outro</div><button class="bs" id="utr">Abrir lista de usuários</button></div>`;
      $('#usn', f).onclick = async () => { try { S.usuario = await api('PUT', '/api/usuarios/' + S.usuario.id, { nome: $('#un', f).value }); desenharPerfil(); aviso('Nome salvo.', 2000); } catch (e) { aviso(e.message); } };
      $$('[data-c]', f).forEach(b => b.onclick = async () => { S.usuario = await api('PUT', '/api/usuarios/' + S.usuario.id, { cor: b.dataset.c }); desenharPerfil(); mostrar('usuario'); });
      $('#utr', f).onclick = () => { f.remove(); sessionStorage.removeItem('hl.u'); S.usuario = null; telaUsuarios(); };
    }
    if (k === 'inst') { p.innerHTML = '<p style="margin-top:0">Confira o que o Karthus precisa e instale o que faltar, sem internet, a partir da pasta <b>instaladores</b>.</p><button class="bp" id="abrirSaude">Abrir verificação</button>'; $('#abrirSaude', f).onclick = () => { f.remove(); painelSaude(); }; }
    if (k === 'sobre') { const e = await api('GET', '/api/estado'); p.innerHTML = `<div class="linha2"><div>Karthus</div><b>${esc(e.versao)}</b></div><div class="linha2"><div>Motor de IA</div><span>${esc(e.ollama)}</span></div><div class="linha2"><div>Atualizar</div><button class="bs" id="upd">Escolher arquivo .hirelod</button></div><p style="color:var(--fg2)">Dica: também dá para arrastar um arquivo .hirelod para dentro da janela.</p>`; $('#upd', f).onclick = () => $('#arqUpd').click(); }
    $$('[data-sel]', p).forEach(s => s.onclick = e => {
      const b = e.target.closest('[data-v]'); if (!b) return; const n = s.dataset.sel; let v = b.dataset.v; if (v === 'true') v = true; if (v === 'false') v = false;
      $$('button', s).forEach(x => x.classList.toggle('on', x === b)); salvarCfg({ [n]: v });
    });
    const sp = $('#salvaPersona', f); if (sp) sp.onclick = () => { salvarCfg({ persona: $('#persona', f).value }); aviso('Instruções salvas.', 2000); };
  };
  $$('.abas button', f).forEach(b => b.onclick = () => mostrar(b.dataset.k)); mostrar(aba);
}

/* ---------------------------------------------------------------- início */
(async () => {
  autoAltura(); atualizarBotao();
  setInterval(() => { if (S.usuario) atualizarSaude(); }, 30000);
  const id = sessionStorage.getItem('hl.u');
  if (id) { try { const us = await api('GET', '/api/usuarios'); const u = us.find(x => x.id === id); if (u) { S.usuarios = us; return entrar(u); } } catch (_) {} }
  telaUsuarios();
})();

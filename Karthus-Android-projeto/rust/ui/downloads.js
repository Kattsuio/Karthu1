'use strict';
/* Karthus 5.0 - downloads em segundo plano, catálogo de IAs, pesquisa na web e modo fluxo. */
window.DL = { itens: [], ativos: false };
const fmtB = b => b >= 1e9 ? (b / 1e9).toFixed(2) + ' GB' : b >= 1e6 ? (b / 1e6).toFixed(1) + ' MB' : Math.max(0, Math.round(b / 1e3)) + ' KB';
async function dlPuxar() {
  try { window.DL = await api('GET', '/api/downloads'); } catch (_) {}
  dlIndicador(); return window.DL;
}
function dlIndicador() {
  const b = $('#dlBt'); if (!b) return;
  const at = DL.itens.filter(x => x.estado === 'ativo' || x.estado === 'fila');
  const rec = DL.itens.some(x => x.estado === 'erro' && Date.now() / 1000 - x.fim < 120);
  b.classList.toggle('on', at.length > 0 || rec); b.classList.toggle('err', !at.length && rec);
  if (at.length) {
    const t = at.reduce((s, x) => s + (x.total || 0), 0), f = at.reduce((s, x) => s + (x.feito || 0), 0);
    const pct = t ? Math.round(f * 100 / t) : 0;
    b.innerHTML = `<span class="anel" style="--p:${pct}"></span><b>${t ? pct + '%' : '…'}</b>`;
    b.title = at.map(x => `${x.titulo}: ${x.total ? Math.round(x.feito * 100 / x.total) + '%' : '…'}`).join('\n');
  } else { b.innerHTML = '<span class="anel" style="--p:0"></span><b>!</b>'; b.title = 'Um download falhou: clique para ver'; }
}
function dlHtml(d, aba) {
  const todos = d.itens; d = { itens: todos.filter(x => aba === 'cr' ? x.cat === 'criacao' : x.cat !== 'criacao') };
  if (!d.itens.length && aba === 'cr') return '<p style="color:var(--fg2)">Nada sendo criado agora. Peça uma imagem, vídeo, arquivo ou cortes no chat (ou em Criações > Criar) e a produção aparece aqui, em segundo plano.</p>';
  if (!d.itens.length) return '<p style="color:var(--fg2)">Nenhum download ou instalação por enquanto. Tudo que for baixado aparece aqui e continua em segundo plano — o senhor pode conversar com a IA enquanto isso.</p>';
  const ord = [...d.itens].sort((a, b) => (b.estado === 'ativo') - (a.estado === 'ativo') || (b.inicio || 0) - (a.inicio || 0));
  return `<div style="display:flex;justify-content:flex-end;margin-bottom:6px"><button class="bs" data-limpar>Limpar concluídos</button></div>` + ord.map(x => {
    const pct = x.estado === 'ok' ? 100 : x.cat === 'criacao' ? Math.min(99, x.pct || 0) : x.total ? Math.min(99, x.feito * 100 / x.total) : 0;
    const ativo = x.estado === 'ativo', fila = x.estado === 'fila';
    const ind = ativo && !x.total && x.cat !== 'criacao';
    const eta = ativo && x.eta_s ? ` · entrega estimada em ~${Math.max(1, Math.round(x.eta_s / 60))} min` : '';
    const ver = aba === 'cr' && x.estado === 'ok' && x.arquivo ? `<button class="bs" data-ver="${esc(x.arquivo)}">Ver</button>` : '';
    return `<div class="dl ${x.estado}"><div class="dl-topo"><b>${esc(x.titulo)}</b><span class="dl-pct">${x.estado === 'ok' ? 'Concluído' : x.estado === 'erro' ? 'Falhou' : fila ? 'Na fila' : ((x.total || x.cat === 'criacao') ? Math.round(pct) + '%' : '…')}</span></div>
      <div class="barra${ind ? ' ind' : ''}"><i style="width:${ind ? 40 : pct}%"></i></div>
      <small>${esc(x.msg || '')}${eta}${ativo && x.total ? ` · ${fmtB(x.feito)} de ${fmtB(x.total)}` : ''}${ativo && x.vel > 0 ? ` · ${fmtB(x.vel)}/s` : ''}</small>${ver}</div>`;
  }).join('');
}
function dlLigar(el, rep) {
  $$('[data-ver]', el).forEach(b => b.onclick = () => verCriacao(b.dataset.ver));
  const l = $('[data-limpar]', el); if (l) l.onclick = async () => { await api('POST', '/api/downloads/limpar'); await dlPuxar(); rep(); };
}
function dlPainel() { painelSaude('dl'); }
/* sonda leve: só pergunta enquanto há algo baixando */
setInterval(async () => { if (!S.usuario) return; await dlPuxar(); }, 1500);
document.addEventListener('DOMContentLoaded', () => { $('#dlBt').onclick = () => dlPainel(); setTimeout(dlPuxar, 800); });

/* ---- botões web / fluxo ---- */
S.web = localStorage.getItem('hl.web') === '1';
try { S.fluxo = JSON.parse(localStorage.getItem('hl.fluxo') || '[]'); } catch (_) { S.fluxo = []; }
const ICG = P('<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c3 3 3 15 0 18M12 3c-3 3-3 15 0 18"/>');
const ICF = P('<circle cx="5" cy="12" r="2.2"/><circle cx="19" cy="6" r="2.2"/><circle cx="19" cy="18" r="2.2"/><path d="M7 11l10-4M7 13l10 4"/>');
function pintarToggles() {
  const w = $('#webBt'), f = $('#fluxoBt'); if (!w) return;
  w.innerHTML = ICG + '<span>Web</span>'; w.classList.toggle('on', S.web);
  f.innerHTML = ICF + '<span>Fluxo' + (S.fluxo.length >= 2 ? ' (' + S.fluxo.length + ')' : '') + '</span>'; f.classList.toggle('on', S.fluxo.length >= 2);
}
document.addEventListener('DOMContentLoaded', () => {
  pintarToggles();
  $('#webBt').onclick = () => { S.web = !S.web; localStorage.setItem('hl.web', S.web ? '1' : '0'); pintarToggles(); aviso(S.web ? 'Pesquisa na web LIGADA: a IA vai consultar sites públicos antes de responder.' : 'Pesquisa na web desligada: tudo fica só no seu computador.', 4500); };
  $('#fluxoBt').onclick = () => escolherFluxo();
});
function escolherFluxo() {
  const ms = S.modelos.map(m => m.nome);
  const f = janela('Modo fluxo', `<div class="painel"><p>Vários modelos trabalham juntos: cada um escreve um rascunho, depois cada um revisa usando os rascunhos dos outros, e o modelo principal junta tudo na resposta final. <b>Demora bem mais</b> (cada modelo é carregado um de cada vez). Escolha 2 ou mais:</p>
  ${ms.length < 2 ? '<p style="color:var(--fg2)">Só há um modelo instalado. Baixe outro no catálogo de IAs.</p>' : ms.map(n => `<label class="comp" style="cursor:pointer"><input type="checkbox" data-m="${esc(n)}" ${S.fluxo.includes(n) ? 'checked' : ''}><div class="txt"><b>${esc(n)}</b></div></label>`).join('')}
  <div style="display:flex;gap:10px;margin-top:10px"><button class="bp" id="fok">Salvar</button><button class="bs" id="fcat">Catálogo de IAs</button><button class="bs" id="foff">Desligar</button></div></div>`, '520px');
  $('#fok', f).onclick = () => { S.fluxo = $$('[data-m]', f).filter(x => x.checked).map(x => x.dataset.m); if (S.fluxo.length === 1) { S.fluxo = []; aviso('Escolha pelo menos 2 modelos.'); } localStorage.setItem('hl.fluxo', JSON.stringify(S.fluxo)); pintarToggles(); f.remove(); };
  $('#foff', f).onclick = () => { S.fluxo = []; localStorage.setItem('hl.fluxo', '[]'); pintarToggles(); f.remove(); };
  $('#fcat', f).onclick = () => { f.remove(); catalogoIA(); };
}

/* ---- botões que a IA oferece no texto: [[baixar:ID]] [[usar:NOME]] [[aplicar-imagem]] ---- */
function fichas(h) {
  return h.replace(/\[\[(baixar|usar):([\w.\-: ]{1,80})\]\]/g, (_, k, v) => `<button class="ficha" data-a="tok" data-k="${k}" data-v="${esc(v)}">${k === 'baixar' ? 'Baixar ' : 'Usar '}${esc(v)}</button>`)
          .replace(/\[\[aplicar-imagem\]\]/g, '<button class="ficha" data-a="tok" data-k="aplicar">Aplicar decisão de imagem/vídeo</button>');
}
async function fichaClique(b) {
  const k = b.dataset.k, v = b.dataset.v; b.disabled = true;
  try {
    if (k === 'baixar') { await api('POST', '/api/catalogo/baixar', { id: v }); await dlPuxar(); aviso('Baixando em segundo plano. Veja o ícone de downloads.', 4500); }
    else if (k === 'usar') { await api('POST', '/api/catalogo/usar', { nome: v }); await salvarCfg({ modelo: v }); await carregarModelos(); aviso('Agora a IA ativa é ' + v, 3500); }
    else if (k === 'aplicar') { const r = await api('POST', '/api/recursos/aplicar-imagem'); aviso(r.ok ? 'Aplicado: ' + (r.motivos || []).join(' ') : (r.erro || 'Não foi possível aplicar.'), 12000); }
  } catch (e) { aviso(e.message, 8000); b.disabled = false; }
}

/* ---- catálogo de IAs ---- */
async function catalogoIA() {
  const f = janela('Catálogo de IAs', '<div class="painel" id="cat"><p style="color:var(--fg2)">Carregando…</p></div>', '760px');
  let cont = true;
  const pintar = async () => {
    if (!document.body.contains(f)) { cont = false; return; }
    let c; try { c = await api('GET', '/api/catalogo'); } catch (e) { $('#cat', f).innerHTML = `<p>${esc(e.message)}</p>`; return; }
    const quando = c.atualizado ? new Date(c.atualizado * 1000).toLocaleString('pt-BR') : 'nunca';
    const grupos = [['Instaladas', c.itens.filter(i => i.instalado)], ['Disponíveis para baixar', c.itens.filter(i => !i.instalado)]];
    const base = c.itens.filter(i => i.instalado && /gemma|llama/i.test(i.familia)).length;
    $('#cat', f).innerHTML = `<div style="display:flex;gap:10px;flex-wrap:wrap;align-items:center;margin-bottom:8px"><button class="bp" id="atu">Atualizar lista (pesquisar online)</button><button class="bs" id="add">Adicionar arquivo .gguf</button><span style="color:var(--fg2)">Memória deste PC: ${c.ram_gb || '?'} GB · lista atualizada: ${esc(quando)}</span></div>
    <p style="color:var(--fg2);margin:4px 0 10px">O gemma4 é opcional. As IAs abaixo rodam 100% no seu PC depois de baixadas; a internet só é usada para baixar o arquivo e, se o senhor ligar o botão Web, para pesquisar.</p>` +
    grupos.map(([t, l]) => l.length ? `<div class="sec" style="margin:10px 0 4px">${t}</div>` + l.map(i => `<div class="comp"><div class="txt"><b>${esc(i.nome)}</b> <small style="display:inline">${esc(i.familia)}${i.raciocina ? ' · raciocina' : ''}${i.uso ? ' · ' + esc(i.uso) : ''}${i.tamanho ? ' · ' + fmtB(i.tamanho) : ''}</small><small>${esc(i.descricao || '')}${!i.cabe && !i.instalado ? ' — provavelmente pesada demais para este PC.' : ''}</small></div>
      ${i.instalado ? (i.ativo ? '<span class="selo2">em uso</span>' : `<button class="bp" data-usar="${esc(i.nome_local)}">Usar</button>`) + (i.fonte !== 'local' && i.nome_local ? `<button class="bs" data-rm="${esc(i.nome_local)}">Remover</button>` : '') : i.baixando ? '<span class="girando"></span>' : `<button class="bp" data-bx="${esc(i.id)}">Baixar</button>`}</div>`).join('') : '').join('');
    $('#atu', f).onclick = async () => { try { await api('POST', '/api/catalogo/atualizar'); await dlPuxar(); aviso('Pesquisando IAs novas em segundo plano…', 3500); setTimeout(pintar, 2500); } catch (e) { aviso(e.message); } };
    $('#add', f).onclick = () => { const inp = document.createElement('input'); inp.type = 'file'; inp.accept = '.gguf'; inp.onchange = async () => { const fl = inp.files[0]; if (!fl) return; try { aviso('Adicionando ' + fl.name + '…', 60000); await enviarMotor('modelo', fl, (a, b) => aviso(`Adicionando ${fl.name}: ${Math.round(a * 100 / b)}%`, 60000)); aviso('Adicionada.', 3000); await carregarModelos(); pintar(); } catch (e) { aviso(e.message, 8000); } }; inp.click(); };
    $$('[data-bx]', f).forEach(b => b.onclick = async () => { b.disabled = true; try { await api('POST', '/api/catalogo/baixar', { id: b.dataset.bx }); await dlPuxar(); aviso('Baixando em segundo plano. Veja o ícone de downloads.', 4500); pintar(); } catch (e) { aviso(e.message, 8000); b.disabled = false; } });
    $$('[data-usar]', f).forEach(b => b.onclick = async () => { await api('POST', '/api/catalogo/usar', { nome: b.dataset.usar }); await salvarCfg({ modelo: b.dataset.usar }); await carregarModelos(); pintar(); });
    $$('[data-rm]', f).forEach(b => b.onclick = async () => { if (!await confirmar('Remover ' + b.dataset.rm + ' do computador?')) return; try { await api('DELETE', '/api/catalogo/' + encodeURIComponent(b.dataset.rm)); await carregarModelos(); pintar(); } catch (e) { aviso(e.message, 8000); } });
  };
  await pintar();
  let n = 0; (async () => { while (cont && n++ < 2000) { await new Promise(r => setTimeout(r, 2500)); if (DL.ativos) await pintar(); } })();
}

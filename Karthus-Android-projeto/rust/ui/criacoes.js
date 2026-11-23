'use strict';
/* Karthus 6.0 - Criações: galeria, tarefas em segundo plano, Cortes, barra de título própria e escala. */
const ICO = {
  corte: P('<circle cx="6" cy="6" r="2.6"/><circle cx="6" cy="18" r="2.6"/><path d="M8 8l12 10M8 16L20 6"/>'),
  arq: P('<path d="M6 3h8l5 5v13H6z"/><path d="M14 3v5h5"/>'),
  baixar: P('<path d="M12 3v12m0 0l-4-4m4 4l4-4M5 20h14"/>'),
  pasta: P('<path d="M3 6h6l2 2h10v11H3z"/>'),
  lixo: P('<path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13"/>'),
  filme: P('<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M10 9l5 3-5 3z"/>'),
};
const TIPOS = { imagem: 'Imagem', video: 'Vídeo', corte: 'Corte', arquivo: 'Arquivo', download: 'Download' };
const salvarComo = async nome => { try { const r = await api('POST', '/api/salvar-dialogo', { arq: nome }); if (r.caminho) aviso('Salvo em ' + r.caminho, 6000); } catch (er) { aviso(er.message); } };
const salvarVarios = async arqs => { if (!arqs.length) return aviso('Nada para salvar.'); try { const r = await api('POST', '/api/salvar-pasta', { arqs }); if (r.pasta) aviso(r.copiados + ' arquivo(s) salvos em ' + r.pasta, 6000); } catch (er) { aviso(er.message); } };
const urlSaida = a => '/api/saida/' + encodeURIComponent(a);

/* ---- tarefas em andamento (mini lista usada nas abas) ---- */
function tarefasHtml() {
  const t = (window.DL ? DL.itens : []).filter(x => x.cat === 'criacao' && (x.estado === 'ativo' || x.estado === 'fila' || (x.estado !== 'ok' ? (Date.now() / 1000 - x.fim < 90) : (Date.now() / 1000 - x.fim < 6))));
  if (!t.length) return '';
  return '<div class="tarefas">' + t.map(x => {
    const pct = x.estado === 'ok' ? 100 : Math.min(99, x.pct || 0);
    const eta = x.estado === 'ativo' && x.eta_s ? ` · entrega estimada ~${Math.max(1, Math.round(x.eta_s / 60))} min` : '';
    return `<div class="tarefa ${x.estado}"><span class="giro"></span><div class="tx"><b>${esc(x.titulo)}</b><small>${esc(x.estado === 'fila' ? 'Na fila…' : x.msg || '')}${eta}</small><div class="barra"><i style="width:${pct}%"></i></div></div><span class="pc">${x.estado === 'erro' ? 'Falhou' : x.estado === 'ok' ? 'Pronto' : Math.round(pct) + '%'}</span></div>`;
  }).join('') + '</div>';
}
function ligarTarefas(el) {
  let ult = '', fim = 0;
  const l = () => { if (!document.body.contains(el)) return; const h = tarefasHtml(); if (h !== ult) { const q = $('.tar-slot', el); if (q) q.innerHTML = h; ult = h; } setTimeout(l, 700); };
  l();
}

/* ---- visualizar uma criação ---- */
function verCriacao(arq) {
  const ext = arq.split('.').pop().toLowerCase();
  const corpo = ['png', 'jpg', 'jpeg', 'webp', 'gif'].includes(ext) ? `<img src="${urlSaida(arq)}" style="max-width:100%;border-radius:12px">`
    : ['mp4', 'webm', 'mkv'].includes(ext) ? `<video src="${urlSaida(arq)}" controls autoplay style="max-width:100%;border-radius:12px;background:#000"></video>`
    : `<p>Arquivo <b>${esc(arq)}</b></p><a class="bs" href="${urlSaida(arq)}?baixar=1" download>Baixar</a>`;
  const f = janela('Criação', `<div class="painel" style="text-align:center">${corpo}<div style="margin-top:12px;display:flex;gap:8px;justify-content:center"><button class="bp" data-sv>Salvar como…</button><button class="bs" data-ab>Abrir pasta</button></div></div>`, '760px');
  $('[data-sv]', f).onclick = () => salvarComo(arq);
  $('[data-ab]', f).onclick = () => api('POST', '/api/abrir-pasta', { arq });
}

/* ---- galeria de Criações (barra lateral) ---- */
async function telaCriacoes() {
  const f = janela('Criações', '<div class="painel" id="gal"><div class="tar-slot"></div><div class="filtros"></div><div class="imggen" id="gg"></div></div>', '980px');
  let lista = [], filtro = 'todos', sig = '';
  const pintar = () => {
    const fl = [['todos', 'Tudo'], ['imagem', 'Imagens'], ['video', 'Vídeos'], ['corte', 'Cortes'], ['arquivo', 'Arquivos'], ['download', 'Downloads']];
    $('.filtros', f).innerHTML = fl.map(([k, t]) => `<button class="pill ${k === filtro ? 'on' : ''}" data-f="${k}">${t}</button>`).join('') + `<span class="esp"></span><button class="bs" data-novo>${ICO.corte} Criar…</button><button class="bs" data-tudo>${ICO.baixar} Salvar tudo em uma pasta…</button><button class="bs" data-pasta>${ICO.pasta} Abrir pasta</button>`;
    const v = lista.filter(c => filtro === 'todos' || c.tipo === filtro);
    $('#gg', f).innerHTML = v.length ? v.map((c, i) => {
      const u = urlSaida(c.arq);
      const m = c.tipo === 'imagem' ? `<img loading="lazy" src="${u}">` : (c.tipo === 'video' || c.tipo === 'corte') ? `<video src="${u}#t=0.5" preload="metadata" muted></video><span class="play">${ICO.filme}</span>` : `<div class="arqi">${ICO.arq}<b>${esc(c.arq.split('.').pop().toUpperCase())}</b></div>`;
      return `<figure style="--i:${i % 12}" data-ver="${esc(c.arq)}"><div class="mid">${m}</div><figcaption><span title="${esc(c.trecho || c.prompt || c.titulo)}"><i class="tag">${TIPOS[c.tipo] || c.tipo}</i> ${esc((c.titulo || '').slice(0, 30))}</span><span class="ac"><button data-sv="${esc(c.arq)}" title="Salvar como…">${ICO.baixar}</button><button data-rm="${esc(c.arq)}" title="Apagar">${ICO.lixo}</button></span></figcaption></figure>`;
    }).join('') : `<p style="color:var(--fg2);grid-column:1/-1">Nada por aqui ainda. Peça no chat, por exemplo: <i>“crie uma imagem de um gato astronauta”</i>, <i>“gere um vídeo de ondas no mar”</i> ou <i>“crie um arquivo csv com…”</i> — tudo é produzido em segundo plano e aparece aqui. Para cortes de vídeo, use o botão Criar…</p>`;
  };
  const carregar = async () => {
    if (!document.body.contains(f)) return;
    try { const l = await api('GET', '/api/criacoes'); const s = l.map(x => x.arq).join('|'); if (s !== sig) { sig = s; lista = l; pintar(); } } catch (_) {}
    setTimeout(carregar, 1500);
  };
  f.addEventListener('click', async e => {
    const b = e.target.closest('button'); const fg = e.target.closest('figure');
    if (b && b.dataset.f) { filtro = b.dataset.f; pintar(); return; }
    if (b && b.hasAttribute('data-novo')) { f.remove(); telaImagens('cortes'); return; }
    if (b && b.hasAttribute('data-tudo')) { salvarVarios(lista.filter(c => filtro === 'todos' || c.tipo === filtro).map(c => c.arq)); return; }
    if (b && b.hasAttribute('data-pasta')) { api('POST', '/api/abrir-pasta', {}); return; }
    if (b && b.dataset.sv) { salvarComo(b.dataset.sv); return; }
    if (b && b.dataset.rm) { if (!await confirmar('Apagar esta criação?')) return; await api('DELETE', '/api/criacoes/' + encodeURIComponent(b.dataset.rm)); sig = ''; carregar(); return; }
    if (fg && fg.dataset.ver) verCriacao(fg.dataset.ver);
  });
  ligarTarefas(f); pintar(); carregar();
}

/* ---- Criar: Imagem, Vídeo, Cortes, Arquivo, Motores ---- */
function telaImagens(aba = 'imagem') {
  const f = janela('Criar', `<div class="corpo2" style="flex-direction:column"><div class="abas-h">${[['imagem', 'Imagem'], ['video', 'Vídeo'], ['cortes', 'Cortes'], ['arquivo', 'Arquivo'], ['motores', 'Motores']].map(([k, t]) => `<button data-k="${k}">${t}</button>`).join('')}</div><div class="painel" id="cp2"></div></div>`, '900px');
  const P2 = $('#cp2', f); const P_ = P2;
  const enviado = (txt) => { aviso(txt + ' Acompanhe aqui embaixo, em Criações ou no ícone de downloads.', 5000); dlPuxar(); };
  const slot = '<div class="tar-slot"></div><button class="bs" id="vg_" style="margin-top:10px">Ver em Criações</button>';
  const rod = async (rota, corpo, btn, msg) => { btn.disabled = true; try { await api('POST', rota, corpo); enviado(msg); } catch (e) { aviso(e.message, 9000); atualizarSaude(); } setTimeout(() => btn.disabled = false, 800); };
  const abas = {
    imagem() {
      P2.innerHTML = `<textarea class="cx" id="ip" placeholder="Descreva a imagem (em inglês costuma dar melhor resultado)"></textarea>
        <div class="fila-op"><select id="is"><option value="512x512">512 × 512</option><option value="768x512">768 × 512</option><option value="512x768">512 × 768</option><option value="768x768">768 × 768</option></select>
        <label>Passos <input type="text" id="ipas" value="20" size="3"></label><button class="bp" id="ig">Gerar em segundo plano</button></div>${slot}`;
      $('#ig', P2).onclick = () => { const p = $('#ip', P2).value.trim(); if (!p) return; const [w, h] = $('#is', P2).value.split('x').map(Number); rod('/api/criar/imagem', { prompt: p, largura: w, altura: h, passos: +$('#ipas', P2).value || 20 }, $('#ig', P2), 'Imagem na fila.'); };
    },
    video() {
      P2.innerHTML = `<textarea class="cx" id="vp" placeholder="Descreva o vídeo (cada cena vira uma imagem com movimento suave)"></textarea>
        <div class="fila-op"><label>Cenas <select id="vc"><option>3</option><option selected>4</option><option>6</option><option>8</option></select></label>
        <label>Segundos por cena <select id="vs"><option>2</option><option selected>3</option><option>4</option><option>6</option></select></label>
        <select id="vt"><option value="512x512">512 × 512</option><option value="768x512">768 × 512 (paisagem)</option><option value="512x768">512 × 768 (retrato)</option></select>
        <button class="bp" id="vg">Gerar em segundo plano</button></div>${slot}`;
      $('#vg', P2).onclick = () => { const p = $('#vp', P2).value.trim(); if (!p) return; const [w, h] = $('#vt', P2).value.split('x').map(Number); rod('/api/criar/video', { prompt: p, cenas: +$('#vc', P2).value, segundos: +$('#vs', P2).value, largura: w, altura: h }, $('#vg', P2), 'Vídeo na fila.'); };
    },
    cortes() {
      P2.innerHTML = `<p style="margin-top:0;color:var(--fg2)">Escolha um vídeo do seu computador. O Karthus transcreve a fala, escolhe os melhores trechos e corta sozinho. <b>100% local e gratuito</b>: sem chave de API, sem tokens, sem enviar nada para a internet.</p>
        <div class="fila-op"><input type="text" id="cv" placeholder="Caminho do vídeo (ex.: C:\\Videos\\aula.mp4)" style="flex:1;min-width:240px"><button class="bs" id="cesc">${ICO.pasta} Escolher vídeo…</button></div>
        <div class="grade-op"><label>Quantos cortes<select id="cq"><option>1</option><option>2</option><option selected>3</option><option>5</option><option>8</option></select></label>
        <label>Duração de cada<select id="cd"><option value="15-30">15 a 30 s</option><option value="30-60" selected>30 a 60 s</option><option value="60-90">60 a 90 s</option></select></label>
        <label class="ck"><input type="checkbox" id="cvert"> Vertical 9:16 (Reels/TikTok/Shorts)</label>
        <label class="ck"><input type="checkbox" id="cleg" checked> Legendas no vídeo</label>
        <label class="ck"><input type="checkbox" id="cia" checked> Usar a IA local para escolher os trechos</label></div>
        <div class="fila-op"><button class="bp" id="cgo">Gerar cortes</button><span id="cest" style="color:var(--fg2)"></span></div>${slot}`;
      $('#cesc', P2).onclick = async () => { try { const r = await api('POST', '/api/escolher-arquivo'); if (r.caminho) $('#cv', P2).value = r.caminho; } catch (e) { aviso(e.message + ' (cole o caminho no campo)', 6000); } };
      $('#cgo', P2).onclick = () => { const v = $('#cv', P2).value.trim(); if (!v) { aviso('Escolha um vídeo primeiro.'); return; } const [mn, mx] = $('#cd', P2).value.split('-').map(Number);
        rod('/api/criar/cortes', { video: v, quantidade: +$('#cq', P2).value, min: mn, max: mx, vertical: $('#cvert', P2).checked, legendas: $('#cleg', P2).checked, usar_ia: $('#cia', P2).checked }, $('#cgo', P2), 'Cortes na fila.'); };
    },
    arquivo() {
      P2.innerHTML = `<p style="margin-top:0;color:var(--fg2)">Peça qualquer arquivo de texto ou código (csv, json, html, py, md, sql…). A IA escreve e salva em Criações. Também dá para baixar um arquivo de um endereço https público.</p>
        <textarea class="cx" id="ap" placeholder="Ex.: planilha csv com 10 clientes fictícios (nome, cidade, e-mail)"></textarea>
        <div class="fila-op"><button class="bp" id="ago">Criar em segundo plano</button></div>
        <div class="fila-op"><input type="text" id="au" placeholder="https://… (arquivo para baixar)" style="flex:1;min-width:240px"><button class="bs" id="adl">${ICO.baixar} Baixar</button></div>${slot}`;
      $('#ago', P2).onclick = () => { const p = $('#ap', P2).value.trim(); if (!p) return; rod('/api/criar/arquivo', { pedido: p }, $('#ago', P2), 'Arquivo na fila.'); };
      $('#adl', P2).onclick = () => { const u = $('#au', P2).value.trim(); if (!u) return; rod('/api/criar/download', { url: u }, $('#adl', P2), 'Download na fila.'); };
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
  const ir = k => { $$('.abas-h button', f).forEach(b => b.classList.toggle('on', b.dataset.k === k)); reanimar(P2, 'troca'); abas[k](); const sl = $('.tar-slot', f); if (sl) ligarTarefas(f); const v = $('#vg_', f); if (v) v.onclick = () => { f.remove(); telaCriacoes(); }; };
  $$('.abas-h button', f).forEach(b => b.onclick = () => ir(b.dataset.k));
  ir(aba);
}

/* ---- escala automática: a interface cresce com o tamanho da tela ---- */
function aplicarEscala() {
  const modo = (typeof S !== 'undefined' && S.cfg && S.cfg.escala) || 'auto';
  let z = 1;
  if (modo === 'auto') z = window.innerWidth < 700 ? 1 : Math.max(0.75, Math.min(1.6, Math.min(window.innerWidth / 1280, window.innerHeight / 760)));
  else z = { pequena: 0.8, normal: 1, grande: 1.25, enorme: 1.5 }[modo] || 1;
  z = Math.round(z * 20) / 20;
  document.documentElement.style.setProperty('--z', z);
  document.documentElement.style.zoom = z;
}
window.addEventListener('resize', aplicarEscala); document.addEventListener('DOMContentLoaded', aplicarEscala); aplicarEscala();

/* janela pequena: a barra lateral fecha sozinha para a conversa ter espaço */
(function () { let ult = null; const f = () => { const w = window.innerWidth / (parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--z')) || 1); const peq = w < 900; if (peq !== ult) { ult = peq; const a = document.getElementById('app'); if (a) a.classList.toggle('fechado', peq); const ab = document.getElementById('abrirLado'); if (ab) ab.style.display = peq ? '' : 'none'; } }; window.addEventListener('resize', f); window.addEventListener('load', () => setTimeout(f, 50)); })();

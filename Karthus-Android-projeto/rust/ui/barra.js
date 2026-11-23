'use strict';
/* Karthus - barra de título própria (arrastar, minimizar, maximizar, fechar). Usada na tela de carregamento e no app. */
(function(){var st=document.createElement('style');st.textContent="/* barra de t\u00edtulo pr\u00f3pria */\n#tb{display:none;height:48px;align-items:stretch;background:var(--bg2,var(--bg));user-select:none;position:fixed;top:0;left:0;right:0;z-index:100;animation:fade .3s both}\n.moldura-propria #tb{display:flex}\n.tb-marca{display:flex;align-items:center;gap:8px;padding:0 20px;font-weight:600;font-size:14px;letter-spacing:.2px}\n.tb-arrasta{flex:1}\n.tb-bt{display:flex}.tb-bt button{width:58px;display:flex;align-items:center;justify-content:center;color:var(--fg);transition:background .12s}.tb-bt button:hover{background:var(--hov)}.tb-bt button.x:hover{background:#e81123;color:#fff}\n.tb-bt svg{width:14px;height:14px;fill:none;stroke:currentColor;stroke-width:1;opacity:.85}\n.rz{position:fixed;z-index:101}.rz-n,.rz-s{left:6px;right:6px;height:5px;cursor:ns-resize}.rz-n{top:0}.rz-s{bottom:0}\n.rz-e,.rz-w{top:6px;bottom:6px;width:5px;cursor:ew-resize}.rz-e{right:0}.rz-w{left:0}\n.rz-ne,.rz-nw,.rz-se,.rz-sw{width:9px;height:9px}.rz-ne{top:0;right:0;cursor:nesw-resize}.rz-nw{top:0;left:0;cursor:nwse-resize}.rz-se{bottom:0;right:0;cursor:nwse-resize}.rz-sw{bottom:0;left:0;cursor:nesw-resize}\n.moldura-propria body{background:var(--bg)}@keyframes fade{from{opacity:0}to{opacity:1}}@keyframes fadeb{from{opacity:0}to{opacity:1}}";document.head.appendChild(st);})();
/* ---- barra de título própria (janela sem moldura) ---- */
(function () {
  const ipc = window.ipc; if (!ipc) return;
  document.documentElement.classList.add('moldura-propria');
  const tb = document.createElement('div'); tb.id = 'tb';
  tb.innerHTML = `<div class="tb-marca"></div><div class="tb-arrasta" id="tbA"></div><div class="tb-bt"><button data-a="min" title="Minimizar"><svg viewBox="0 0 12 12"><path d="M2 6h8"/></svg></button><button data-a="max" title="Maximizar ou restaurar"><svg viewBox="0 0 12 12"><rect x="2.2" y="2.2" width="7.6" height="7.6"/></svg></button><button data-a="close" class="x" title="Fechar"><svg viewBox="0 0 12 12"><path d="M2.5 2.5l7 7M9.5 2.5l-7 7"/></svg></button></div>`;
  document.body.prepend(tb);
  const A = tb.querySelector('#tbA');
  A.addEventListener('mousedown', e => { if (e.button === 0 && e.detail === 1) ipc.postMessage('drag'); });
  A.addEventListener('dblclick', () => ipc.postMessage('max'));
  tb.querySelectorAll('button').forEach(b => b.onclick = () => ipc.postMessage(b.dataset.a));
  ['n', 's', 'e', 'w', 'ne', 'nw', 'se', 'sw'].forEach(d => { const r = document.createElement('div'); r.className = 'rz rz-' + d; r.onmousedown = e => { if (e.button === 0) ipc.postMessage('resize:' + d); }; document.body.appendChild(r); });
})();


/* ---- confirmação e pergunta dentro do app (no lugar das caixas do navegador), com animação ---- */
(function () {
  const st = document.createElement('style');
  st.textContent = `.dlg-fundo{position:fixed;inset:0;z-index:300;background:rgba(0,0,0,.45);display:flex;align-items:center;justify-content:center;animation:dlgf .18s both}
.dlg{background:var(--bg,#fff);color:var(--fg,#111);border:1px solid var(--line,#ddd);border-radius:16px;padding:22px 22px 18px;width:min(380px,88vw);box-shadow:0 18px 50px rgba(0,0,0,.35);animation:dlgc .22s cubic-bezier(.2,.9,.3,1.2) both}
.dlg p{margin:0 0 16px;font-size:15px;line-height:1.45}
.dlg input{width:100%;box-sizing:border-box;margin:0 0 16px;padding:9px 12px;border-radius:10px;border:1px solid var(--line,#ddd);background:var(--bg2,transparent);color:inherit;font:inherit;outline:none}
.dlg input:focus{border-color:var(--acc,#888)}
.dlg .bts{display:flex;justify-content:flex-end;gap:8px}
.dlg button{font:inherit;font-weight:600;border:0;border-radius:10px;padding:8px 18px;cursor:pointer;background:var(--bg3,#eee);color:var(--fg,#111);transition:transform .1s,filter .12s}
.dlg button:hover{filter:brightness(1.12)}.dlg button:active{transform:scale(.96)}
.dlg button.ok{background:var(--acc,#222);color:var(--accfg,#fff)}.dlg button.ok.perigo{background:#d4202f;color:#fff}
.dlg-fundo.sai{animation:dlgs .14s both}.dlg-fundo.sai .dlg{animation:dlgx .14s both}
@keyframes dlgf{from{opacity:0}to{opacity:1}}@keyframes dlgs{to{opacity:0}}
@keyframes dlgc{from{opacity:0;transform:translateY(10px) scale(.94)}to{opacity:1;transform:none}}@keyframes dlgx{to{opacity:0;transform:scale(.96)}}
@media(prefers-reduced-motion:reduce){.dlg,.dlg-fundo{animation:none!important}}`;
  document.head.appendChild(st);
  function abrir(msg, o) {
    return new Promise(res => {
      const f = document.createElement('div'); f.className = 'dlg-fundo';
      const d = document.createElement('div'); d.className = 'dlg'; d.setAttribute('role', 'dialog');
      const p = document.createElement('p'); p.textContent = msg; d.appendChild(p);
      let inp = null;
      if (o.campo) { inp = document.createElement('input'); inp.value = o.valor || ''; d.appendChild(inp); }
      const b = document.createElement('div'); b.className = 'bts';
      const c = document.createElement('button'); c.textContent = 'Cancelar';
      const k = document.createElement('button'); k.textContent = 'OK'; k.className = 'ok' + (o.perigo ? ' perigo' : '');
      b.append(c, k); d.appendChild(b); f.appendChild(d); document.body.appendChild(f);
      let fim = false;
      const sair = v => { if (fim) return; fim = true; document.removeEventListener('keydown', tecla, true); f.classList.add('sai'); setTimeout(() => f.remove(), 150); res(v); };
      const tecla = e => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); sair(o.campo ? null : false); } else if (e.key === 'Enter') { e.preventDefault(); e.stopPropagation(); sair(o.campo ? inp.value : true); } };
      document.addEventListener('keydown', tecla, true);
      c.onclick = () => sair(o.campo ? null : false); k.onclick = () => sair(o.campo ? inp.value : true);
      f.onmousedown = e => { if (e.target === f) sair(o.campo ? null : false); };
      setTimeout(() => (inp || k).focus(), 30); if (inp) setTimeout(() => inp.select(), 40);
    });
  }
  window.confirmar = (msg, perigo) => abrir(msg, { perigo: perigo !== false });
  window.perguntar = (msg, valor) => abrir(msg, { campo: true, valor });
})();

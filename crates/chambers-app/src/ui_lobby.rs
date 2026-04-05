//! Lobby UI — the main window HTML.
//! Uses HTTP fetch for data, IPC only for "open chamber window" signal.

pub fn lobby_html() -> String {
    r##"<!DOCTYPE html>
<html><head><meta charset="utf-8">
<style>
*{margin:0;padding:0;box-sizing:border-box}
:root{--bg:#111118;--bg2:#1a1a2e;--fg:#e8e8e8;--fg2:#888;--green:#16c784;--red:#ea3943;
--yellow:#f0b90b;--mono:'SF Mono','Cascadia Code','Fira Code',monospace}
html,body{height:100%;background:var(--bg);color:var(--fg);font:13px/1.6 var(--mono)}
button{font:inherit;cursor:pointer;background:#0a4a2a;color:var(--fg);border:1px solid #1a6a3a;
padding:8px 20px;border-radius:4px}
button:hover{background:#1a6a3a}
button.secondary{background:#16213e;border-color:#334}
button.secondary:hover{background:#1a5a90}
input,select{font:inherit;background:var(--bg);color:var(--fg);border:1px solid #334;padding:6px 10px;border-radius:4px;width:100%}
select{width:auto;min-width:200px}
label{display:block;color:var(--fg2);font-size:10px;text-transform:uppercase;letter-spacing:.8px;margin-bottom:3px}
table{width:100%;border-collapse:collapse;font-size:12px}
th,td{text-align:left;padding:6px 10px;border-bottom:1px solid #1a1a2e}
th{color:var(--fg2);font-size:10px;text-transform:uppercase}
.container{display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:100vh;padding:40px}
h1{font-size:24px;letter-spacing:5px;margin-bottom:4px}
.subtitle{color:var(--fg2);font-size:12px;margin-bottom:40px}
.box{background:var(--bg2);border:1px solid #2a2a4a;border-radius:8px;padding:28px;width:100%;max-width:480px}
.box h2{font-size:11px;text-transform:uppercase;letter-spacing:1px;color:var(--fg2);margin-bottom:14px}
.field{margin-bottom:12px}
.btn-row{display:flex;gap:10px;margin-top:14px}
.result{margin-top:10px;padding:8px;border-radius:4px;font-size:12px}
.result.err{background:rgba(234,57,67,.1);border:1px solid rgba(234,57,67,.25);color:var(--red)}
.result.ok{background:rgba(22,199,132,.1);border:1px solid rgba(22,199,132,.25);color:var(--green)}
.result.warn{background:rgba(240,185,11,.08);border:1px solid rgba(240,185,11,.2);color:var(--yellow)}
#archive{margin-top:36px;width:100%;max-width:600px}
#archive h2{font-size:11px;text-transform:uppercase;letter-spacing:1px;color:var(--fg2);margin-bottom:10px}
</style></head><body>
<div class="container">
  <h1>CHAMBERS</h1>
  <div class="subtitle">Thinking rooms that self-destruct</div>
  <div class="box">
    <h2>Open a Chamber</h2>
    <div class="field"><label>Chamber Design</label>
      <select id="f-grammar"><option value="decision_chamber_v1">Decision Chamber</option></select></div>
    <div class="field"><label>Question</label>
      <input id="f-objective" type="text" placeholder="What are you deciding?"></div>
    <div class="btn-row">
      <button onclick="openChamber()">Open Chamber</button>
      <button class="secondary" onclick="loadDemo()">Load Demo</button>
    </div>
    <div id="result"></div>
  </div>
  <div id="archive">
    <h2>Archive — Survivors from past chambers</h2>
    <div id="archive-list"><span style="color:var(--fg2)">No survivors yet.</span></div>
  </div>
</div>
<script>
'use strict';
const API = '';  // same origin — relative URLs work
const el = () => document.getElementById('result');

async function api(method, path, body) {
  const opts = { method, headers: { 'Content-Type': 'application/json' } };
  if (body) opts.body = JSON.stringify(body);
  const res = await fetch(API + path, opts);
  if (!res.ok) { const t = await res.text(); throw new Error(t || 'HTTP ' + res.status); }
  return res.json();
}

function esc(s) { if (s==null) return ''; const d=document.createElement('div'); d.textContent=String(s); return d.innerHTML; }

function entryContent(p) {
  if (!p || typeof p !== 'object') return String(p||'');
  return p.statement||p.description||p.decision||p.summary||'';
}

async function openChamber() {
  const objective = document.getElementById('f-objective').value.trim();
  if (!objective) { el().innerHTML = '<div class="result err">Enter a question</div>'; return; }
  el().innerHTML = '<div class="result warn">Creating chamber...</div>';
  try {
    const data = await api('POST', '/api/worlds', { grammar_id: document.getElementById('f-grammar').value, objective });
    // Tell Rust to spawn the native window
    await fetch('/app/open-chamber/' + data.world_id, { method: 'POST' });
    el().innerHTML = '<div class="result ok">Chamber opened: ' + data.world_id.substring(0,13) + '</div>';
  } catch(e) {
    el().innerHTML = '<div class="result err">' + esc(e.message) + '</div>';
  }
}

async function loadDemo() {
  el().innerHTML = '<div class="result warn">Creating chamber...</div>';
  try {
    // Create world via HTTP
    const data = await api('POST', '/api/worlds', {
      grammar_id: 'decision_chamber_v1',
      objective: 'Should we build our own authentication system or buy an identity provider?'
    });
    const wid = data.world_id;
    el().innerHTML = '<div class="result warn">Adding entries...</div>';

    // Add all demo entries via HTTP
    const ids = {};
    async function add(key, type, payload, lc) {
      const r = await api('POST', '/api/worlds/' + wid + '/submit',
        { CreateObject: { object_type: type, payload, lifecycle_class: lc || 'Temporary', preservable: false }});
      ids[key] = r.ObjectCreated;
    }
    async function link(src, tgt, type) {
      await api('POST', '/api/worlds/' + wid + '/submit',
        { LinkObjects: { source_id: ids[src], target_id: ids[tgt], link_type: type }});
    }

    await add('p1','premise',{statement:'Our current auth is a custom PHP implementation from 2016 with known security issues.',source:'security audit Q1'});
    await add('p2','premise',{statement:'Engineering team has 2 developers with identity/auth experience.',source:'team lead'});
    await add('p3','premise',{statement:'Three customer contracts require SOC2 compliance by end of year.',source:'sales team'});
    await add('c1','constraint',{description:'Must achieve SOC2 Type II compliance within 8 months.',severity:'hard'});
    await add('c2','constraint',{description:'Migration must not cause more than 4 hours of downtime.',severity:'hard'});
    await add('c3','constraint',{description:'Annual cost must stay under $50,000.',severity:'soft'});
    await add('a1','alternative',{description:'Auth0 — managed identity platform',pros:'Fast to implement, SOC2 certified, handles MFA/SSO out of box',cons:'Vendor lock-in, per-user pricing scales badly'},'Intermediate');
    await add('a2','alternative',{description:'Build custom with Passport.js + PostgreSQL',pros:'Full control, no vendor dependency, one-time cost',cons:'6+ months to build, security risk during development'},'Intermediate');
    await add('a3','alternative',{description:'Keycloak self-hosted',pros:'Open source, full control, no per-user cost',cons:'Complex to operate, requires dedicated DevOps'},'Intermediate');
    await add('r1','risk',{description:'Custom build takes 6+ months, missing SOC2 deadline — loss of $400K ARR.',likelihood:'high',impact:'critical'});
    await add('r2','risk',{description:'Auth0 per-user pricing exceeds budget at 50K+ users (month 18).',likelihood:'medium',impact:'medium'});
    await add('r3','risk',{description:'Keycloak operational complexity causes auth outages in first 6 months.',likelihood:'medium',impact:'high'});
    await add('u1','upside',{description:'Auth0 gets us SOC2-ready in under 2 months, unblocking all 3 contracts immediately.',magnitude:'high'});
    await add('u2','upside',{description:'Keycloak gives full data sovereignty for EU expansion planned Q3.',magnitude:'medium'});

    el().innerHTML = '<div class="result warn">Connecting entries...</div>';
    await link('r1','a2','risks');
    await link('r2','a1','risks');
    await link('r3','a3','risks');
    await link('u1','a1','benefits');
    await link('u2','a3','benefits');

    // Tell Rust to spawn the chamber window
    await fetch('/app/open-chamber/' + wid, { method: 'POST' });
    el().innerHTML = '<div class="result ok">Demo loaded — 14 entries, 5 connections</div>';
  } catch(e) {
    el().innerHTML = '<div class="result err">Demo failed: ' + esc(e.message) + '</div>';
  }
}

async function loadArchive() {
  try {
    const artifacts = await api('GET', '/api/vault');
    const list = artifacts.artifacts || artifacts;
    const ael = document.getElementById('archive-list');
    if (!list || list.length === 0) { ael.innerHTML = '<span style="color:var(--fg2)">No survivors yet.</span>'; return; }
    let h = '<table><tr><th>Kind</th><th>Content</th><th>Preserved At</th></tr>';
    for (const a of list) {
      h += '<tr><td>' + esc(a.artifact_class) + '</td><td style="max-width:300px">' +
        esc(entryContent(a.payload||{})) + '</td><td style="color:var(--fg2)">' + esc(a.sealed_at) + '</td></tr>';
    }
    h += '</table>';
    ael.innerHTML = h;
  } catch(e) { /* silent on startup */ }
}

loadArchive();
setInterval(loadArchive, 3000);

// Esc from lobby = quit the app. This is the only exit.
document.addEventListener('keydown', function(e) {
  if (e.key === 'Escape') {
    fetch('/app/quit', { method: 'POST' });
  }
  // Block Cmd+Q and other system shortcuts in lobby too
  if (e.metaKey || e.ctrlKey) {
    e.preventDefault();
  }
});
// Block right-click in lobby
document.addEventListener('contextmenu', function(e) { e.preventDefault(); });
</script></body></html>"##.to_string()
}

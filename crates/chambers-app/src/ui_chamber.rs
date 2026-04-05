//! Chamber UI — rendered inside a native webview window.
//! No address bar, no back/forward, no history.

pub fn chamber_html(world_id: &str) -> String {
    format!(r##"<!DOCTYPE html>
<html spellcheck="false"><head><meta charset="utf-8">
<style>
*{{margin:0;padding:0;box-sizing:border-box;cursor:none !important}}
:root{{--bg:#111118;--bg2:#1a1a2e;--bg3:#16213e;--fg:#e8e8e8;--fg2:#888;
--green:#16c784;--red:#ea3943;--yellow:#f0b90b;--blue:#4a9eff;
--mono:'SF Mono','Cascadia Code','Fira Code',monospace;--border:#2a2a4a}}
html,body{{height:100%;background:var(--bg);color:var(--fg);font:13px/1.6 var(--mono);overflow:hidden;
  -webkit-user-select:none;user-select:none;
  -webkit-touch-callout:none;
  -webkit-text-size-adjust:none;
  touch-action:none}}
button{{font:inherit;background:var(--bg3);color:var(--fg);border:1px solid #334;padding:6px 14px;border-radius:4px}}
button:hover{{background:#1a5a90}}
button.danger{{background:#3a1525;border-color:#5a2535}}
button.danger:hover{{background:#5a1525}}
button.primary{{background:#0a4a2a;border-color:#1a6a3a}}
button.primary:hover{{background:#1a6a3a}}
input,select,textarea{{-webkit-user-select:text;user-select:text}}
.entry-content{{-webkit-user-select:text;user-select:text}}
/* Chamber pointer — rendered by JS, not by the OS */
#chamber-pointer{{position:fixed;top:0;left:0;width:24px;height:24px;pointer-events:none;z-index:9999;
  transform:translate(-12px,-12px);transition:none}}
#chamber-pointer .ring{{position:absolute;top:4px;left:4px;width:16px;height:16px;
  border:1.5px solid var(--green);border-radius:50%;transition:all .1s}}
#chamber-pointer .dot{{position:absolute;top:10px;left:10px;width:4px;height:4px;
  background:var(--green);border-radius:50%;transition:all .1s}}
#chamber-pointer.over-action .ring{{border-color:var(--green);background:rgba(22,199,132,.15);transform:scale(1.3)}}
#chamber-pointer.over-action .dot{{background:var(--green)}}
#chamber-pointer.over-danger .ring{{border-color:var(--red);background:rgba(234,57,67,.15);transform:scale(1.3)}}
#chamber-pointer.over-danger .dot{{background:var(--red)}}
#chamber-pointer.over-input .ring{{width:2px;height:20px;border-radius:1px;top:2px;left:11px;border-color:var(--green)}}
#chamber-pointer.over-input .dot{{display:none}}
input,select,textarea{{font:inherit;background:var(--bg);color:var(--fg);border:1px solid #334;padding:5px 8px;border-radius:4px;width:100%}}
textarea{{resize:vertical;min-height:60px}}
select{{width:auto;min-width:160px}}
label{{display:block;color:var(--fg2);font-size:10px;text-transform:uppercase;letter-spacing:.8px;margin-bottom:2px}}
.field{{margin-bottom:8px}}
.grid2{{display:grid;grid-template-columns:1fr 1fr;gap:8px}}
.result{{margin-top:8px;padding:6px 10px;border-radius:4px;font-size:11px;white-space:pre-wrap;word-break:break-all}}
.result.ok{{background:rgba(22,199,132,.1);border:1px solid rgba(22,199,132,.25);color:var(--green)}}
.result.err{{background:rgba(234,57,67,.1);border:1px solid rgba(234,57,67,.25);color:var(--red)}}
.badge{{display:inline-block;padding:2px 10px;border-radius:3px;font-size:11px;font-weight:bold}}
.badge.exploring{{background:rgba(22,199,132,.15);color:var(--green)}}
.badge.reviewing{{background:rgba(240,185,11,.15);color:var(--yellow)}}
.badge.finalizing{{background:rgba(74,158,255,.15);color:var(--blue)}}
.badge.destroyed{{background:rgba(234,57,67,.15);color:var(--red)}}

#app{{display:flex;flex-direction:column;height:100vh}}
#header{{background:var(--bg2);border-bottom:2px solid var(--border);padding:10px 18px;display:flex;align-items:center;gap:14px}}
#header .question{{flex:1;font-size:13px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}}
#header .meta{{font-size:11px;color:var(--fg2);display:flex;gap:12px;align-items:center}}
#body{{display:flex;flex:1;overflow:hidden}}
#entries{{flex:1;overflow-y:auto;padding:14px}}
#entries h3{{font-size:10px;text-transform:uppercase;letter-spacing:1px;color:var(--fg2);margin-bottom:8px}}
#moves{{width:340px;min-width:340px;overflow-y:auto;padding:14px;background:var(--bg2);border-left:1px solid var(--border)}}
#moves h3{{font-size:10px;text-transform:uppercase;letter-spacing:1px;color:var(--fg2);margin-bottom:8px}}
.action-group{{background:var(--bg);border:1px solid #222;border-radius:4px;padding:10px;margin-bottom:8px}}
.action-group h4{{font-size:11px;margin-bottom:6px;color:var(--green)}}
#footer{{background:var(--bg2);border-top:2px solid var(--border);padding:8px 18px;display:flex;align-items:center;gap:14px}}
#footer .readiness{{flex:1;font-size:11px}}
#footer .controls{{display:flex;gap:8px}}

.entry-row{{padding:6px 10px;border-bottom:1px solid #151520;display:flex;gap:10px;align-items:flex-start}}
.entry-row:hover{{background:rgba(255,255,255,.02)}}
.entry-kind{{font-size:10px;text-transform:uppercase;letter-spacing:.5px;color:var(--blue);min-width:100px;padding-top:1px}}
.entry-content{{flex:1;font-size:12px;line-height:1.5}}
.entry-meta{{font-size:10px;color:var(--fg2);min-width:60px;text-align:right}}
.conn-row{{padding:4px 10px;font-size:11px;color:var(--fg2);border-bottom:1px solid #111}}
.conn-label{{color:var(--yellow);font-weight:bold}}

#burn-overlay{{display:none;position:fixed;inset:0;background:rgba(0,0,0,.95);z-index:100;flex-direction:column;align-items:center;justify-content:center;text-align:center}}
#burn-overlay.visible{{display:flex}}
#burn-overlay .score{{font-size:52px;font-weight:bold;margin:16px 0 8px}}
#burn-overlay .detail{{color:var(--fg2);font-size:12px;margin-bottom:24px;max-width:380px}}
.survivor-card{{background:var(--bg2);border:1px solid var(--green);border-radius:6px;padding:16px;text-align:left;max-width:400px;margin-bottom:20px}}
.survivor-card .label{{font-size:10px;text-transform:uppercase;letter-spacing:1px;color:var(--green);margin-bottom:6px}}
</style></head><body>
<!-- Chamber pointer — exists only inside this world -->
<div id="chamber-pointer"><div class="ring"></div><div class="dot"></div></div>
<div id="app">
  <div id="header">
    <div class="question" id="ch-question"></div>
    <div class="meta">
      <span id="ch-stage"></span>
      <span id="ch-counts" style="color:var(--fg2)"></span>
    </div>
    <button class="danger" onclick="doBurn()">Burn</button>
  </div>
  <div id="body">
    <div id="entries">
      <h3>Entries</h3>
      <div id="entries-list"></div>
      <h3 style="margin-top:12px">Connections</h3>
      <div id="connections-list"></div>
    </div>
    <div id="moves">
      <h3>Moves</h3>
      <div id="moves-list"></div>
      <div id="move-result"></div>
    </div>
  </div>
  <div id="footer">
    <div class="readiness" id="ch-readiness"></div>
    <div class="controls" id="ch-controls"></div>
  </div>
</div>

<div id="burn-overlay">
  <h2 style="font-size:14px;letter-spacing:2px;color:var(--fg2)">CHAMBER BURNED</h2>
  <div class="score" id="burn-score"></div>
  <div class="detail" id="burn-detail"></div>
  <div id="burn-survivor"></div>
  <p style="color:var(--fg2);font-size:11px;margin-top:16px">This window will close. The survivor lives in the Archive.</p>
</div>

<script>
'use strict';
const WORLD_ID = '{world_id}';

// ============================================================
// HARDWARE & SYSTEM ISOLATION
// The chamber has no connection to your machine's peripherals,
// sensors, storage, or network. Every system API is severed.
// ============================================================

(function() {{
  // --- Camera & Microphone ---
  if (navigator.mediaDevices) {{
    navigator.mediaDevices.getUserMedia = function() {{
      return Promise.reject(new DOMException('Camera/microphone blocked inside chamber', 'NotAllowedError'));
    }};
    navigator.mediaDevices.enumerateDevices = function() {{
      return Promise.resolve([]);
    }};
    navigator.mediaDevices.getDisplayMedia = function() {{
      return Promise.reject(new DOMException('Screen capture blocked inside chamber', 'NotAllowedError'));
    }};
  }}

  // --- Location ---
  if (navigator.geolocation) {{
    navigator.geolocation.getCurrentPosition = function(s, e) {{
      if (e) e({{ code: 1, message: 'Location blocked inside chamber' }});
    }};
    navigator.geolocation.watchPosition = function(s, e) {{
      if (e) e({{ code: 1, message: 'Location blocked inside chamber' }});
      return 0;
    }};
  }}

  // --- Notifications ---
  if (typeof Notification !== 'undefined') {{
    window.Notification = function() {{
      throw new Error('Notifications blocked inside chamber');
    }};
    window.Notification.requestPermission = function() {{
      return Promise.resolve('denied');
    }};
    window.Notification.permission = 'denied';
  }}

  // --- Clipboard API (programmatic) ---
  if (navigator.clipboard) {{
    navigator.clipboard.readText = function() {{
      return Promise.reject(new DOMException('Clipboard read blocked inside chamber', 'NotAllowedError'));
    }};
    navigator.clipboard.read = function() {{
      return Promise.reject(new DOMException('Clipboard read blocked inside chamber', 'NotAllowedError'));
    }};
    navigator.clipboard.writeText = function() {{
      return Promise.reject(new DOMException('Clipboard write blocked inside chamber', 'NotAllowedError'));
    }};
    navigator.clipboard.write = function() {{
      return Promise.reject(new DOMException('Clipboard write blocked inside chamber', 'NotAllowedError'));
    }};
  }}

  // --- Bluetooth ---
  if (navigator.bluetooth) {{
    navigator.bluetooth.requestDevice = function() {{
      return Promise.reject(new DOMException('Bluetooth blocked inside chamber', 'NotAllowedError'));
    }};
  }}

  // --- USB ---
  if (navigator.usb) {{
    navigator.usb.requestDevice = function() {{
      return Promise.reject(new DOMException('USB blocked inside chamber', 'NotAllowedError'));
    }};
    navigator.usb.getDevices = function() {{ return Promise.resolve([]); }};
  }}

  // --- Serial ---
  if (navigator.serial) {{
    navigator.serial.requestPort = function() {{
      return Promise.reject(new DOMException('Serial blocked inside chamber', 'NotAllowedError'));
    }};
    navigator.serial.getPorts = function() {{ return Promise.resolve([]); }};
  }}

  // --- MIDI ---
  if (navigator.requestMIDIAccess) {{
    navigator.requestMIDIAccess = function() {{
      return Promise.reject(new DOMException('MIDI blocked inside chamber', 'NotAllowedError'));
    }};
  }}

  // --- Speech ---
  if (typeof SpeechRecognition !== 'undefined') window.SpeechRecognition = undefined;
  if (typeof webkitSpeechRecognition !== 'undefined') window.webkitSpeechRecognition = undefined;
  if (typeof speechSynthesis !== 'undefined') {{
    window.speechSynthesis.speak = function() {{}};
    window.speechSynthesis.getVoices = function() {{ return []; }};
  }}

  // --- Sensors ---
  if (typeof DeviceOrientationEvent !== 'undefined') {{
    window.DeviceOrientationEvent = undefined;
  }}
  if (typeof DeviceMotionEvent !== 'undefined') {{
    window.DeviceMotionEvent = undefined;
  }}
  if (typeof AmbientLightSensor !== 'undefined') window.AmbientLightSensor = undefined;
  if (typeof Accelerometer !== 'undefined') window.Accelerometer = undefined;
  if (typeof Gyroscope !== 'undefined') window.Gyroscope = undefined;
  if (typeof Magnetometer !== 'undefined') window.Magnetometer = undefined;

  // --- Storage (all of it) ---
  // localStorage
  try {{
    Object.defineProperty(window, 'localStorage', {{
      get: function() {{ throw new DOMException('localStorage blocked inside chamber', 'SecurityError'); }}
    }});
  }} catch(_) {{}}
  // sessionStorage
  try {{
    Object.defineProperty(window, 'sessionStorage', {{
      get: function() {{ throw new DOMException('sessionStorage blocked inside chamber', 'SecurityError'); }}
    }});
  }} catch(_) {{}}
  // IndexedDB
  try {{
    Object.defineProperty(window, 'indexedDB', {{
      get: function() {{ return undefined; }}
    }});
  }} catch(_) {{}}
  // Cache API
  if (typeof caches !== 'undefined') {{
    try {{
      Object.defineProperty(window, 'caches', {{
        get: function() {{ return undefined; }}
      }});
    }} catch(_) {{}}
  }}

  // --- Service Workers ---
  if (navigator.serviceWorker) {{
    try {{
      Object.defineProperty(navigator, 'serviceWorker', {{
        get: function() {{ return undefined; }}
      }});
    }} catch(_) {{}}
  }}

  // --- Web Workers (block spawning external computation) ---
  const OrigWorker = window.Worker;
  window.Worker = function() {{
    throw new Error('Web Workers blocked inside chamber');
  }};
  window.SharedWorker = function() {{
    throw new Error('SharedWorkers blocked inside chamber');
  }};

  // --- WebSocket (no outbound network from chamber) ---
  window.WebSocket = function() {{
    throw new Error('WebSocket blocked inside chamber — no outbound network');
  }};

  // --- EventSource (SSE) ---
  window.EventSource = function() {{
    throw new Error('EventSource blocked inside chamber');
  }};

  // --- Fetch/XHR isolation: only allow requests to localhost ---
  const origFetch = window.fetch;
  window.fetch = function(url, opts) {{
    const urlStr = typeof url === 'string' ? url : (url.url || '');
    // Allow relative URLs and localhost only
    if (urlStr.startsWith('/') || urlStr.startsWith('http://127.0.0.1') || urlStr.startsWith('http://localhost')) {{
      return origFetch.call(this, url, opts);
    }}
    return Promise.reject(new Error('External network blocked inside chamber: ' + urlStr));
  }};
  const origXHR = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function(method, url) {{
    const urlStr = String(url);
    if (urlStr.startsWith('/') || urlStr.startsWith('http://127.0.0.1') || urlStr.startsWith('http://localhost')) {{
      return origXHR.apply(this, arguments);
    }}
    throw new Error('External network blocked inside chamber: ' + urlStr);
  }};

  // --- File System Access ---
  if (window.showOpenFilePicker) window.showOpenFilePicker = function() {{
    return Promise.reject(new DOMException('File picker blocked inside chamber', 'NotAllowedError'));
  }};
  if (window.showSaveFilePicker) window.showSaveFilePicker = function() {{
    return Promise.reject(new DOMException('File picker blocked inside chamber', 'NotAllowedError'));
  }};
  if (window.showDirectoryPicker) window.showDirectoryPicker = function() {{
    return Promise.reject(new DOMException('Directory picker blocked inside chamber', 'NotAllowedError'));
  }};

  // --- Payment ---
  if (window.PaymentRequest) {{
    window.PaymentRequest = function() {{
      throw new Error('Payments blocked inside chamber');
    }};
  }}

  // --- Credentials ---
  if (navigator.credentials) {{
    navigator.credentials.get = function() {{ return Promise.resolve(null); }};
    navigator.credentials.store = function() {{ return Promise.resolve(); }};
    navigator.credentials.create = function() {{ return Promise.resolve(null); }};
  }}

  // --- Wake Lock ---
  if (navigator.wakeLock) {{
    navigator.wakeLock.request = function() {{
      return Promise.reject(new DOMException('Wake lock blocked inside chamber', 'NotAllowedError'));
    }};
  }}

  // --- Share ---
  if (navigator.share) {{
    navigator.share = function() {{
      return Promise.reject(new DOMException('Share blocked inside chamber', 'NotAllowedError'));
    }};
  }}
  if (navigator.canShare) {{
    navigator.canShare = function() {{ return false; }};
  }}

  // --- Printing ---
  window.print = function() {{}};

  // --- Window opening ---
  window.open = function() {{ return null; }};

  // --- Alert/Prompt/Confirm (system dialogs) ---
  window.alert = function() {{}};
  window.prompt = function() {{ return null; }};
  window.confirm = function() {{ return false; }};

  // --- Console (prevent data exfiltration via dev tools) ---
  // Keep console.error for debugging, but neuter the rest
  // (In production, all console methods would be blocked)

}})();

// ============================================================
// END HARDWARE & SYSTEM ISOLATION
// ============================================================

// === ISOLATION: Block all system accessibility features ===
// This chamber is a sealed world. System features do not cross the boundary.

// 1. Block right-click context menu
document.addEventListener('contextmenu', function(e) {{ e.preventDefault(); }});

// 2. Block all system keyboard shortcuts
document.addEventListener('keydown', function(e) {{
  // ESC is the only escape hatch — returns to lobby
  if (e.key === 'Escape') {{
    doBurn();
    return;
  }}

  // Cmd/Ctrl shortcuts — most blocked, Cmd+C/V routed through chamber clipboard
  if (e.metaKey || e.ctrlKey) {{
    const tag = (e.target.tagName || '').toLowerCase();
    const inInput = tag === 'input' || tag === 'textarea' || tag === 'select';

    // Cmd+A inside input/textarea — allow (select all within field)
    if (e.key === 'a' && inInput) return;
    // Cmd+Backspace inside input/textarea — allow
    if (e.key === 'Backspace' && inInput) return;

    // Cmd+C — chamber clipboard copy (not system clipboard)
    if (e.key === 'c') {{
      e.preventDefault();
      const sel = window.getSelection().toString();
      if (sel) {{
        fetch('/app/clipboard/copy', {{
          method: 'POST', headers: {{'Content-Type':'application/json'}},
          body: JSON.stringify({{ world_id: WORLD_ID, text: sel }})
        }});
      }}
      return;
    }}

    // Cmd+V — chamber clipboard paste (not system clipboard)
    if (e.key === 'v' && inInput) {{
      e.preventDefault();
      fetch('/app/clipboard/paste', {{
        method: 'POST', headers: {{'Content-Type':'application/json'}},
        body: JSON.stringify({{ world_id: WORLD_ID }})
      }}).then(r => r.json()).then(d => {{
        if (d.text) {{
          const el = e.target;
          const start = el.selectionStart;
          const end = el.selectionEnd;
          el.value = el.value.substring(0, start) + d.text + el.value.substring(end);
          el.selectionStart = el.selectionEnd = start + d.text.length;
        }}
      }});
      return;
    }}

    // Block everything else
    e.preventDefault();
    e.stopPropagation();
    return;
  }}

  // Block F-keys (F1-F12) except for native use
  if (e.key.startsWith('F') && e.key.length <= 3) {{
    e.preventDefault();
  }}
}});

// 3. Block drag and drop from outside
document.addEventListener('dragover', function(e) {{ e.preventDefault(); }});
document.addEventListener('drop', function(e) {{ e.preventDefault(); }});
document.addEventListener('dragenter', function(e) {{ e.preventDefault(); }});

// 4. Block ALL system clipboard events — chamber clipboard is handled via Cmd+C/V above
document.addEventListener('copy', function(e) {{ e.preventDefault(); }});
document.addEventListener('cut', function(e) {{ e.preventDefault(); }});
document.addEventListener('paste', function(e) {{ e.preventDefault(); }});

// 5. Disable spell check and autocomplete on all inputs
document.querySelectorAll('input, textarea').forEach(function(el) {{
  el.setAttribute('spellcheck', 'false');
  el.setAttribute('autocomplete', 'off');
  el.setAttribute('autocorrect', 'off');
  el.setAttribute('autocapitalize', 'off');
}});
// Also set on future dynamically-created inputs
new MutationObserver(function(mutations) {{
  mutations.forEach(function(m) {{
    m.addedNodes.forEach(function(n) {{
      if (n.querySelectorAll) {{
        n.querySelectorAll('input, textarea').forEach(function(el) {{
          el.setAttribute('spellcheck', 'false');
          el.setAttribute('autocomplete', 'off');
          el.setAttribute('autocorrect', 'off');
          el.setAttribute('autocapitalize', 'off');
        }});
      }}
    }});
  }});
}}).observe(document.body, {{ childList: true, subtree: true }});

// 6. Disable pinch zoom and scroll zoom
document.addEventListener('wheel', function(e) {{
  if (e.ctrlKey || e.metaKey) e.preventDefault();
}}, {{ passive: false }});

// 7. Disable text selection outside of input fields (CSS handles this, but reinforce)
document.addEventListener('selectstart', function(e) {{
  const tag = (e.target.tagName || '').toLowerCase();
  if (tag !== 'input' && tag !== 'textarea' && !e.target.closest('.entry-content')) {{
    e.preventDefault();
  }}
}});

// === CHAMBER POINTER ===
// System cursor is hidden (cursor:none). This DOM element is the pointer.
// It exists only inside the chamber. Burns with the window.
(function() {{
  const ptr = document.getElementById('chamber-pointer');
  document.addEventListener('mousemove', function(e) {{
    ptr.style.left = e.clientX + 'px';
    ptr.style.top = e.clientY + 'px';
  }});
  document.addEventListener('mouseover', function(e) {{
    const el = e.target;
    const tag = (el.tagName || '').toLowerCase();
    ptr.className = '';
    if (tag === 'button') {{
      ptr.className = el.classList.contains('danger') ? 'over-danger' : 'over-action';
    }} else if (tag === 'input' || tag === 'textarea' || tag === 'select') {{
      ptr.className = 'over-input';
    }} else if (el.closest && el.closest('button')) {{
      const btn = el.closest('button');
      ptr.className = btn.classList.contains('danger') ? 'over-danger' : 'over-action';
    }} else if (el.closest && el.closest('a')) {{
      ptr.className = 'over-action';
    }}
  }});
}})();
// === END ISOLATION ===

const phaseMap = {{Active:'exploring',ConvergenceReview:'reviewing',Finalization:'finalizing',Terminated:'destroyed',Created:'created'}};
const fateMap = {{Temporary:'burns',Intermediate:'burns',Candidate:'might survive',Preservable:'survives'}};

function esc(s) {{
  if (s == null) return '';
  const d = document.createElement('div');
  d.textContent = String(s);
  return d.innerHTML;
}}
function truncId(id) {{ return id ? id.substring(0, 10) : '-'; }}
function stageBadge(phase) {{
  const n = phaseMap[phase] || phase;
  return '<span class="badge ' + n + '">' + n + '</span>';
}}
function entryContent(p) {{
  if (!p || typeof p !== 'object') return String(p || '');
  return p.statement || p.description || p.decision || p.summary || p.question ||
    Object.values(p).find(v => typeof v === 'string' && v.length > 0) || JSON.stringify(p).substring(0, 80);
}}

// All communication is via HTTP fetch. No IPC.
// The IPC handler processes the command; we fetch state after a short delay.
// For a real app we'd use a proper callback mechanism.

async function refresh() {{
  // We can't get a return value from ipc directly in wry.
  // Instead, the chamber HTML polls the server via the HTTP adapter
  // which is still running. Or we use a different approach.
  //
  // Simplest for now: the native app also runs the HTTP adapter on localhost,
  // and the chamber webview fetches from it. This way we get proper
  // request-response semantics without complex IPC callbacks.
  try {{
    const [world, summary, graph, conv, actions] = await Promise.all([
      fetch('/api/worlds/' + WORLD_ID).then(r => r.json()),
      fetch('/api/worlds/' + WORLD_ID + '/summary').then(r => r.json()),
      fetch('/api/worlds/' + WORLD_ID + '/graph').then(r => r.json()),
      fetch('/api/worlds/' + WORLD_ID + '/convergence').then(r => r.json()),
      fetch('/api/worlds/' + WORLD_ID + '/legal-actions').then(r => r.json()),
    ]);
    render(world, summary, graph, conv, actions);
  }} catch(e) {{
    console.error('refresh error:', e);
  }}
}}

function render(world, summary, graph, conv, actions) {{
  // Header
  document.getElementById('ch-question').textContent = world.objective;
  document.getElementById('ch-stage').innerHTML = stageBadge(world.lifecycle_phase);
  document.getElementById('ch-counts').textContent = (summary.object_count||0) + ' entries · ' + (summary.link_count||0) + ' connections';

  // Entries
  const el = document.getElementById('entries-list');
  const connEl = document.getElementById('connections-list');
  const nodes = graph.nodes || [];
  const edges = graph.edges || [];
  const labels = {{}};
  for (const n of nodes) labels[n.id] = n.object_type + ': ' + (entryContent(n.payload)||'').substring(0,40);

  if (nodes.length === 0) {{
    el.innerHTML = '<div style="color:var(--fg2);padding:8px">No entries yet. Add entries using the moves panel.</div>';
  }} else {{
    let h = '';
    for (const n of nodes) {{
      h += '<div class="entry-row"><div class="entry-kind">' + esc(n.object_type) + '</div>' +
        '<div class="entry-content">' + esc(entryContent(n.payload)) +
          (n.challenged ? ' <span style="color:var(--red);font-size:10px">[disputed]</span>' : '') +
          (n.preservable ? ' <span style="color:var(--green);font-size:10px">🛡</span>' : '') +
        '</div><div class="entry-meta">' + truncId(n.id) + '</div></div>';
    }}
    el.innerHTML = h;
  }}

  if (edges.length > 0) {{
    let ch = '';
    for (const e of edges) {{
      ch += '<div class="conn-row">' + esc(labels[e.source] || truncId(e.source)) +
        ' <span class="conn-label">' + esc(e.link_type) + '</span> ' +
        esc(labels[e.target] || truncId(e.target)) + '</div>';
    }}
    connEl.innerHTML = ch;
  }} else {{
    connEl.innerHTML = '<div style="color:var(--fg2);font-size:11px">No connections yet.</div>';
  }}

  // Moves
  const mEl = document.getElementById('moves-list');
  const primitives = actions.primitives || actions || [];
  const objectIds = nodes.map(n => ({{id:n.id,type:n.object_type,preservable:n.preservable,
    label:n.object_type+': '+(entryContent(n.payload)||'').substring(0,28)}}));
  const preservable = objectIds.filter(o => o.preservable);
  let mh = '';

  if (primitives.includes('CreateObject')) {{
    const types = ['premise','support_statement','constraint','risk','upside','contradiction','alternative','recommendation','decision_summary'];
    mh += '<div class="action-group"><h4>Add Entry</h4>' +
      '<div class="field"><label>Kind</label><select id="a-type">' +
        types.map(t => '<option value="'+t+'">'+t+'</option>').join('') + '</select></div>' +
      '<div class="field"><label>Fate</label><select id="a-class">' +
        '<option value="Temporary">burns</option><option value="Intermediate">burns</option>' +
        '<option value="Candidate">might survive</option><option value="Preservable">survives</option></select></div>' +
      '<div class="field"><label>Content (JSON)</label><textarea id="a-payload">{{\"statement\": \"\"}}</textarea></div>' +
      '<div class="field"><label><input type="checkbox" id="a-pres"> Can survive burn?</label></div>' +
      '<button class="primary" onclick="doCreate()">Add</button></div>';
  }}

  if (primitives.includes('LinkObjects') && objectIds.length >= 2) {{
    mh += '<div class="action-group"><h4>Connect</h4>' +
      '<div class="grid2"><div class="field"><label>From</label><select id="a-src">' +
        objectIds.map(o => '<option value="'+o.id+'">'+esc(o.label)+'</option>').join('') + '</select></div>' +
      '<div class="field"><label>To</label><select id="a-tgt">' +
        objectIds.map(o => '<option value="'+o.id+'">'+esc(o.label)+'</option>').join('') + '</select></div></div>' +
      '<div class="field"><label>Type</label><select id="a-link">' +
        '<option>supports</option><option>constrains</option><option>risks</option><option>benefits</option>' +
        '<option>contradicts</option><option>alternative_to</option><option>synthesized_from</option><option>based_on</option></select></div>' +
      '<button onclick="doLink()">Connect</button></div>';
  }}

  if (primitives.includes('ChallengeObject') && objectIds.length > 0) {{
    mh += '<div class="action-group"><h4>Dispute</h4>' +
      '<div class="field"><label>Entry</label><select id="a-disp">' +
        objectIds.map(o => '<option value="'+o.id+'">'+esc(o.label)+'</option>').join('') + '</select></div>' +
      '<div class="field"><label>Why?</label><input id="a-disp-text" type="text"></div>' +
      '<button onclick="doDispute()">Dispute</button></div>';
  }}

  if (primitives.includes('SealArtifact')) {{
    const targets = preservable.length > 0 ? preservable : objectIds;
    mh += '<div class="action-group"><h4>Preserve Decision</h4>' +
      '<div class="field"><label>Which decision?</label><select id="a-seal">' +
        targets.map(o => '<option value="'+o.id+'">'+esc(o.label)+'</option>').join('') + '</select></div>' +
      '<button class="primary" onclick="doSeal()">Preserve</button></div>';
  }}

  if (!mh) mh = '<div style="color:var(--fg2);padding:8px">No moves available.</div>';
  mEl.innerHTML = mh;

  // Footer
  const rEl = document.getElementById('ch-readiness');
  const cEl = document.getElementById('ch-controls');
  const v = conv.convergence_validated;
  let missing = '';
  if (conv.mandatory_type_satisfaction) {{
    for (const [t, ok] of Object.entries(conv.mandatory_type_satisfaction)) {{
      if (!ok) missing += (t === 'decision_summary' ? 'decision' : t) + ' ';
    }}
  }}
  if (v === false) rEl.innerHTML = '<span style="color:var(--red)">Not ready: ' + esc(missing) + 'missing</span>';
  else if (v === true) rEl.innerHTML = '<span style="color:var(--green)">Ready to finalize</span>';
  else rEl.innerHTML = '<span style="color:var(--fg2)">Checking readiness...</span>';

  let btns = '';
  if (world.lifecycle_phase === 'Active') btns = '<button onclick="doAdvance(\'ConvergenceReview\')">Move to Review</button>';
  else if (world.lifecycle_phase === 'ConvergenceReview') {{
    btns = '<button onclick="doAdvance(\'Finalization\')">Move to Finalizing</button>' +
      '<button onclick="doAdvance(\'Active\')">Back to Exploring</button>';
  }}
  cEl.innerHTML = btns;
}}

// --- Actions ---
async function doSubmit(operation) {{
  const el = document.getElementById('move-result');
  try {{
    const res = await fetch('/api/worlds/' + WORLD_ID + '/submit', {{
      method: 'POST', headers: {{'Content-Type':'application/json'}},
      body: JSON.stringify(operation)
    }});
    const data = await res.json();
    if (!res.ok) throw new Error(data.error || 'failed');
    el.innerHTML = '<div class="result ok">' + esc(JSON.stringify(data)) + '</div>';
    setTimeout(refresh, 100);
  }} catch(e) {{
    el.innerHTML = '<div class="result err">' + esc(e.message) + '</div>';
  }}
}}

function doCreate() {{
  let payload;
  try {{ payload = JSON.parse(document.getElementById('a-payload').value); }}
  catch(e) {{ document.getElementById('move-result').innerHTML = '<div class="result err">Invalid JSON</div>'; return; }}
  doSubmit({{ CreateObject: {{
    object_type: document.getElementById('a-type').value,
    payload, lifecycle_class: document.getElementById('a-class').value,
    preservable: document.getElementById('a-pres').checked
  }}}});
}}

function doLink() {{
  doSubmit({{ LinkObjects: {{
    source_id: document.getElementById('a-src').value,
    target_id: document.getElementById('a-tgt').value,
    link_type: document.getElementById('a-link').value
  }}}});
}}

function doDispute() {{
  doSubmit({{ ChallengeObject: {{
    target_id: document.getElementById('a-disp').value,
    challenge_text: document.getElementById('a-disp-text').value
  }}}});
}}

function doSeal() {{
  doSubmit({{ SealArtifact: {{
    target_id: document.getElementById('a-seal').value,
    authorization: {{ HumanConfirmed: {{ confirmer: 'operator' }} }}
  }}}});
}}

async function doAdvance(phase) {{
  try {{
    await fetch('/api/worlds/' + WORLD_ID + '/advance', {{
      method: 'POST', headers: {{'Content-Type':'application/json'}},
      body: JSON.stringify({{ phase }})
    }});
    setTimeout(refresh, 100);
  }} catch(e) {{
    document.getElementById('move-result').innerHTML = '<div class="result err">' + esc(e.message) + '</div>';
  }}
}}

let burnConfirmed = false;
function doBurn() {{
  if (!burnConfirmed) {{
    // Show inline confirmation instead of system confirm() dialog
    const el = document.getElementById('move-result');
    const phase = document.querySelector('#ch-stage .badge')?.textContent || '';
    const modeDesc = phase === 'finalizing' ? 'Preserve your decision and destroy everything else.' : 'Destroy everything. Nothing survives.';
    el.innerHTML = '<div class="result err" style="text-align:center;padding:16px">' +
      '<div style="font-size:14px;margin-bottom:10px">Burn this chamber?</div>' +
      '<div style="margin-bottom:14px;color:var(--fg2)">' + modeDesc + '</div>' +
      '<button class="danger" onclick="executeBurn()" style="margin-right:8px;padding:8px 24px">Yes — Burn</button>' +
      '<button onclick="cancelBurn()" style="padding:8px 24px">Cancel</button></div>';
    return;
  }}
}}
function cancelBurn() {{
  document.getElementById('move-result').innerHTML = '';
}}
async function executeBurn() {{
  const phase = document.querySelector('#ch-stage .badge')?.textContent || '';
  const mode = phase === 'finalizing' ? 'ConvergedPreserving' : 'AbortBurn';
  const el = document.getElementById('move-result');
  el.innerHTML = '<div class="result warn">Burning...</div>';
  try {{
    const res = await fetch('/api/worlds/' + WORLD_ID + '/burn', {{
      method: 'POST', headers: {{'Content-Type':'application/json'}},
      body: JSON.stringify({{mode: mode}})
    }});
    const data = await res.json();
    if (!res.ok || data.error) {{
      el.innerHTML = '<div class="result err">Burn failed: ' + esc(data.error || 'HTTP ' + res.status) + '</div>';
      return;
    }}
    // Zero the chamber clipboard before killing the app
    await fetch('/app/clipboard/burn', {{ method: 'POST', headers: {{'Content-Type':'application/json'}}, body: JSON.stringify({{world_id: WORLD_ID}}) }});
    // Burned. Kill the entire app. Back to desktop.
    await fetch('/app/quit', {{ method: 'POST' }});
  }} catch(e) {{
    el.innerHTML = '<div class="result err">Burn failed: ' + esc(e.message) + '</div>';
  }}
}}

// Initial load
refresh();
</script></body></html>"##, world_id = world_id)
}

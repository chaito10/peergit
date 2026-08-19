pub fn dashboard_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>PeerGit Dashboard</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
:root{--bg:#0d1117;--surface:#161b22;--border:#30363d;--text:#c9d1d9;--dim:#8b949e;--accent:#58a6ff;--green:#3fb950;--red:#f85149;--yellow:#d29922}
body{font-family:system-ui,-apple-system,sans-serif;background:var(--bg);color:var(--text);min-height:100vh}
header{background:var(--surface);border-bottom:1px solid var(--border);padding:1rem 2rem;display:flex;align-items:center;gap:1rem}
header h1{font-size:1.25rem;font-weight:600}
header .version{color:var(--dim);font-size:0.8rem}
nav{background:var(--surface);border-bottom:1px solid var(--border);display:flex;gap:0;padding:0 2rem}
nav button{background:none;border:none;color:var(--dim);padding:0.75rem 1.25rem;cursor:pointer;font-size:0.9rem;border-bottom:2px solid transparent;transition:all 0.15s}
nav button:hover{color:var(--text)}
nav button.active{color:var(--accent);border-bottom-color:var(--accent)}
main{max-width:960px;margin:2rem auto;padding:0 2rem}
section{display:none}
section.active{display:block}
.card{background:var(--surface);border:1px solid var(--border);border-radius:8px;padding:1.25rem;margin-bottom:1rem}
.card h2{font-size:1rem;margin-bottom:0.75rem;color:var(--accent)}
.stat-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem}
.stat{background:var(--bg);border:1px solid var(--border);border-radius:6px;padding:1rem}
.stat .label{color:var(--dim);font-size:0.75rem;text-transform:uppercase;letter-spacing:0.05em}
.stat .value{font-size:1.5rem;font-weight:600;margin-top:0.25rem}
.stat .value.green{color:var(--green)}
table{width:100%;border-collapse:collapse;font-size:0.875rem}
th{text-align:left;color:var(--dim);font-weight:500;padding:0.5rem 0.75rem;border-bottom:1px solid var(--border)}
td{padding:0.5rem 0.75rem;border-bottom:1px solid var(--border)}
.mono{font-family:ui-monospace,SFMono-Regular,monospace;font-size:0.8rem}
.badge{display:inline-block;padding:0.15rem 0.5rem;border-radius:99px;font-size:0.75rem;font-weight:500}
.badge.green{background:#23863620;color:var(--green)}
.badge.yellow{background:#9e6a0320;color:var(--yellow)}
.empty{color:var(--dim);padding:2rem;text-align:center}
button.primary{background:var(--accent);color:#fff;border:none;padding:0.5rem 1rem;border-radius:6px;cursor:pointer;font-size:0.875rem}
button.primary:hover{opacity:0.9}
.form-row{display:flex;gap:0.5rem;margin-top:0.75rem}
.form-row input{flex:1;background:var(--bg);border:1px solid var(--border);border-radius:6px;padding:0.5rem 0.75rem;color:var(--text);font-size:0.875rem}
.form-row input::placeholder{color:var(--dim)}
#toast{position:fixed;bottom:1.5rem;right:1.5rem;background:var(--surface);border:1px solid var(--border);border-radius:8px;padding:0.75rem 1.25rem;font-size:0.875rem;opacity:0;transition:opacity 0.3s;pointer-events:none;z-index:100}
#toast.show{opacity:1}
</style>
</head>
<body>
<header>
  <h1>PeerGit</h1>
  <span class="version">v0.1.0</span>
</header>
<nav>
  <button class="active" onclick="show('dashboard')">Dashboard</button>
  <button onclick="show('peers')">Peers</button>
  <button onclick="show('repos')">Repositories</button>
</nav>
<main>
  <section id="dashboard" class="active">
    <div class="card"><h2>Node Status</h2><div class="stat-grid" id="stats"><div class="empty">Loading...</div></div></div>
  </section>
  <section id="peers">
    <div class="card">
      <h2>Known Peers</h2>
      <table><thead><tr><th>Peer ID</th><th>Alias</th><th>Addresses</th><th>Last Seen</th></tr></thead><tbody id="peers-body"><tr><td colspan="4" class="empty">Loading...</td></tr></tbody></table>
      <div class="form-row">
        <input id="pk-input" placeholder="Public key (multibase)">
        <input id="alias-input" placeholder="Alias (optional)">
        <button class="primary" onclick="addPeer()">Add Peer</button>
      </div>
    </div>
  </section>
  <section id="repos">
    <div class="card">
      <h2>Published Repositories</h2>
      <table><thead><tr><th>RID</th><th>Name</th><th>Visibility</th></tr></thead><tbody id="repos-body"><tr><td colspan="3" class="empty">Loading...</td></tr></tbody></table>
    </div>
  </section>
</main>
<div id="toast"></div>
<script>
function show(id){
  document.querySelectorAll('section').forEach(s=>s.classList.remove('active'));
  document.querySelectorAll('nav button').forEach(b=>b.classList.remove('active'));
  document.getElementById(id).classList.add('active');
  document.querySelector(`nav button[onclick="show('${id}')"]`).classList.add('active');
  if(id==='dashboard')loadStatus();if(id==='peers')loadPeers();if(id==='repos')loadRepos();
}
function toast(m){const t=document.getElementById('toast');t.textContent=m;t.classList.add('show');setTimeout(()=>t.classList.remove('show'),3000)}
async function loadStatus(){
  try{const r=await fetch('/api/status');const d=await r.json();
  document.getElementById('stats').innerHTML=`
    <div class="stat"><div class="label">Alias</div><div class="value">${d.alias}</div></div>
    <div class="stat"><div class="label">Peer ID</div><div class="value mono" style="font-size:0.8rem;word-break:break-all">${d.peer_id}</div></div>
    <div class="stat"><div class="label">Peers</div><div class="value green">${d.peer_count}</div></div>
    <div class="stat"><div class="label">Repositories</div><div class="value green">${d.repo_count}</div></div>
    <div class="stat"><div class="label">Listening</div><div class="value" style="font-size:0.85rem">${d.listen}</div></div>
    <div class="stat"><div class="label">Web UI</div><div class="value" style="font-size:0.85rem">localhost:${d.web_port}</div></div>`;
  }catch(e){document.getElementById('stats').innerHTML='<div class="empty">Failed to load status</div>';}
}
async function loadPeers(){
  try{const r=await fetch('/api/peers');const d=await r.json();
  if(!d.length){document.getElementById('peers-body').innerHTML='<tr><td colspan="4" class="empty">No known peers</td></tr>';return;}
  document.getElementById('peers-body').innerHTML=d.map(p=>`<tr><td class="mono">${p.peer_id.slice(0,20)}...</td><td>${p.alias||'-'}</td><td class="mono" style="font-size:0.8rem">${p.addresses||'-'}</td><td>${p.last_seen.slice(0,10)}</td></tr>`).join('');
  }catch(e){document.getElementById('peers-body').innerHTML='<tr><td colspan="4" class="empty">Failed to load peers</td></tr>';}
}
async function loadRepos(){
  try{const r=await fetch('/api/repos');const d=await r.json();
  if(!d.length){document.getElementById('repos-body').innerHTML='<tr><td colspan="3" class="empty">No published repositories</td></tr>';return;}
  document.getElementById('repos-body').innerHTML=d.map(r=>`<tr><td class="mono">${r.rid.slice(0,20)}...</td><td>${r.name}</td><td><span class="badge green">${r.visibility}</span></td></tr>`).join('');
  }catch(e){document.getElementById('repos-body').innerHTML='<tr><td colspan="3" class="empty">Failed to load repos</td></tr>';}
}
async function addPeer(){
  const pk=document.getElementById('pk-input').value.trim();
  const alias=document.getElementById('alias-input').value.trim();
  if(!pk){toast('Public key required');return;}
  try{await fetch('/api/peers',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({public_key:pk,alias:alias||null})});
  document.getElementById('pk-input').value='';document.getElementById('alias-input').value='';toast('Peer added');loadPeers();
  }catch(e){toast('Failed to add peer');}
}
loadStatus();
</script>
</body>
</html>"#
}

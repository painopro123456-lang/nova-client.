import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Activity, ArrowDownToLine, ArrowUpRight, Bell, Blocks, Box, Check,
  ChevronDown, CircleHelp, Compass, Cpu, Gamepad2, Gauge, HardDrive,
  Layers3, LayoutDashboard, Monitor, Moon, MoreHorizontal, Play, Plus,
  Search, Settings, ShieldCheck, SlidersHorizontal, Sparkles, Users,
  WandSparkles, Wifi, X
} from "lucide-react";

const navGroups = [
  { label: "PLAY", items: [{ id: "home", label: "Overview", icon: LayoutDashboard }, { id: "instances", label: "Instances", icon: Layers3 }] },
  { label: "DISCOVER", items: [{ id: "mods", label: "Mods", icon: Blocks }, { id: "modpacks", label: "Modpacks", icon: Box }] },
  { label: "CLIENT", items: [{ id: "pvp", label: "PvP features", icon: Gamepad2 }, { id: "settings", label: "Settings", icon: Settings }] }
];

const versions = ["1.21.4", "1.21.1", "1.20.4", "1.8.9"];

function Sidebar({ page, setPage }) {
  return <aside className="sidebar">
    <div className="brand"><div className="brand-mark"><Sparkles size={19}/></div><div><b>NOVA</b><span>CLIENT</span></div><span className="build-tag">BETA</span></div>
    <div className="profile-card"><div className="avatar">N</div><div className="profile-copy"><b>Guest profile</b><span>Sign in to continue</span></div><ChevronDown size={15}/></div>
    <nav>{navGroups.map(group => <section className="nav-group" key={group.label}><div className="nav-label">{group.label}</div>{group.items.map(item => <button key={item.id} className={page === item.id ? "nav-item active" : "nav-item"} onClick={() => setPage(item.id)}><item.icon size={17}/><span>{item.label}</span>{item.id === "pvp" && <span className="nav-dot"/>}</button>)}</section>)}</nav>
    <div className="sidebar-bottom"><div className="status-line"><span className="status-pulse"/><span>Launcher UI preview</span></div><button className="help-link" onClick={() => setPage("settings")}><CircleHelp size={16}/> Help & support</button><div className="sidebar-foot">NOVA CLIENT <span>0.1.0</span></div></div>
  </aside>;
}

function Topbar({ title, subtitle }) {
  const win = getCurrentWindow();
  return <header className="topbar" data-tauri-drag-region>
    <div data-tauri-drag-region><div className="eyebrow">NOVA / {title.toUpperCase()}</div><h1>{title}</h1><p>{subtitle}</p></div>
    <div className="top-actions">
      <button className="icon-btn" title="Notifications"><Bell size={17}/><i/></button>
      <div className="top-avatar">G</div>
      <div className="window-controls">
        <button title="Minimize" onClick={() => win.minimize()}><span>−</span></button>
        <button title="Maximize" onClick={() => win.toggleMaximize()}><span>□</span></button>
        <button className="close-window" title="Close" onClick={() => win.close()}><X size={14}/></button>
      </div>
    </div>
  </header>;
}

function Home({ onNavigate, selectedVersion, setSelectedVersion, launching, setLaunching }) {
  return <>
    <div className="hero-panel">
      <div className="hero-glow"/>
      <div className="hero-content"><div className="hero-kicker"><span className="kicker-icon"><Sparkles size={13}/></span> YOUR NEXT SESSION STARTS HERE</div><h2>Make every<br/><em>frame count.</em></h2><p>A cleaner way to launch Minecraft. Your worlds, your settings, your pace.</p><div className="hero-actions"><button className="primary-btn" onClick={() => setLaunching(true)} disabled={launching}><Play size={16} fill="currentColor"/>{launching ? "Launch integration coming soon" : "Play Minecraft"}<ArrowUpRight size={15}/></button><button className="secondary-btn" onClick={() => onNavigate("instances")}>Manage instances</button></div><div className="hero-foot"><span><ShieldCheck size={14}/> Local profile preview</span><span><Gauge size={14}/> Lightweight native shell planned</span></div></div>
      <div className="orbit-art" aria-hidden="true"><div className="orbit-ring ring-one"/><div className="orbit-ring ring-two"/><div className="orbit-core"><Sparkles size={48}/></div><div className="orbit-chip chip-one"><Cpu size={15}/> PERFORMANCE</div><div className="orbit-chip chip-two"><Gamepad2 size={15}/> BUILT FOR PLAY</div></div>
      <div className="hero-index">01 <span>/ 04</span></div>
    </div>
    <div className="section-heading"><div><div className="eyebrow">READY WHEN YOU ARE</div><h2>Quick launch</h2></div><button className="text-button" onClick={() => onNavigate("instances")}>All instances <ArrowUpRight size={14}/></button></div>
    <div className="quick-card"><div className="minecraft-icon"><Box size={22}/></div><div className="quick-info"><b>Vanilla Minecraft</b><span>Clean profile · No extra mods</span></div><div className="version-select"><label>GAME VERSION</label><select value={selectedVersion} onChange={e => setSelectedVersion(e.target.value)}>{versions.map(v => <option key={v}>{v}</option>)}</select></div><button className="small-play" onClick={() => setLaunching(true)} disabled={launching}><Play size={15} fill="currentColor"/> Launch</button></div>
    <div className="feature-grid"><div className="feature-card"><div className="feature-icon purple"><Gauge size={18}/></div><b>Performance first</b><p>A native Rust shell is the foundation. No performance claims until benchmarks exist.</p><span className="feature-status">IN DEVELOPMENT</span></div><div className="feature-card"><div className="feature-icon blue"><Blocks size={18}/></div><b>Your mod library</b><p>Planned browsing and installs through trusted mod platforms.</p><span className="feature-status">PLANNED</span></div><div className="feature-card"><div className="feature-icon green"><ShieldCheck size={18}/></div><b>Profiles that stay yours</b><p>Keep game versions and configurations organized by instance.</p><span className="feature-status">PLANNED</span></div></div>
  </>;
}


function Catalog({ packs }) {
  const [q,setQ]=useState("");
  const [provider,setProvider]=useState("modrinth");
  const [apiKey,setApiKey]=useState("");
  const [hits,setHits]=useState([]);
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState("");
  useEffect(()=>{
    const ctrl=new AbortController();
    const timer=setTimeout(async()=>{
      setBusy(true); setError("");
      try {
        let results=[];
        if(provider==="modrinth") {
          const p=new URLSearchParams({query:q,facets:JSON.stringify([[packs?"project_type:modpack":"project_type:mod"]]),limit:"24"});
          const r=await fetch("https://api.modrinth.com/v2/search?"+p,{signal:ctrl.signal});
          if(!r.ok) throw new Error("Modrinth catalog request failed");
          const data=await r.json();
          results=(data.hits||[]).map(x=>({id:x.project_id,title:x.title,description:x.description,downloads:x.downloads,image:x.icon_url,url:"https://modrinth.com/"+(packs?"modpack/":"mod/")+x.slug}));
        } else {
          if(!apiKey.trim()) { setHits([]); setError("Enter your CurseForge API key to search its official catalog."); setBusy(false); return; }
          const p=new URLSearchParams({gameId:"432",classId:packs?"4471":"6",pageSize:"24",sortField:"2",sortOrder:"desc"});
          if(q.trim()) p.set("searchFilter",q.trim());
          const r=await fetch("https://api.curseforge.com/v1/mods/search?"+p,{headers:{"x-api-key":apiKey.trim(),"Accept":"application/json"},signal:ctrl.signal});
          if(r.status===403) throw new Error("CurseForge rejected this API key. Check that it is valid and approved for API access.");
          if(r.status===401) throw new Error("CurseForge API key is missing or invalid.");
          if(!r.ok) throw new Error("CurseForge catalog request failed ("+r.status+").");
          const data=await r.json();
          results=(data.data||[]).map(x=>({id:String(x.id),title:x.name,description:x.summary||"No description provided.",downloads:x.downloadCount,image:x.logo?.thumbnailUrl,url:x.links?.websiteUrl||"https://www.curseforge.com/minecraft/"+(packs?"modpacks":"mc-mods")+"/"+x.slug}));
        }
        setHits(results);
      } catch(e) { if(e.name!=="AbortError") setError(e.message||"Unable to load catalog. Check your connection and retry."); }
      finally { if(!ctrl.signal.aborted) setBusy(false); }
    },250);
    return ()=>{clearTimeout(timer);ctrl.abort();};
  },[q,packs,provider,apiKey]);
  return <section className="catalog-page">
    <div className="eyebrow">{provider==="modrinth"?"MODRINTH":"CURSEFORGE"} CATALOG · LIVE SEARCH</div>
    <h2>{packs?"Discover modpacks":"Discover mods"}</h2>
    <p>Search either catalog and open the project's page to review files and downloads.</p>
    <div className="catalog-toolbar">
      <div className="catalog-provider" role="group" aria-label="Catalog provider">
        <button className={provider==="modrinth"?"provider-choice selected":"provider-choice"} onClick={()=>setProvider("modrinth")}>Modrinth</button>
        <button className={provider==="curseforge"?"provider-choice selected":"provider-choice"} onClick={()=>setProvider("curseforge")}>CurseForge</button>
      </div>
    </div>
    {provider==="curseforge"&&<div className="curseforge-key">
      <label htmlFor="curseforge-api-key">CURSEFORGE API KEY</label>
      <input id="curseforge-api-key" type="password" autoComplete="off" value={apiKey} onChange={e=>setApiKey(e.target.value)} placeholder="Paste your CurseForge API key"/>
      <span>Your key stays in this app session and is not saved to NOVA's repository.</span>
    </div>}
    <div className="catalog-search"><Search size={17}/><input value={q} onChange={e=>setQ(e.target.value)} placeholder={packs?"Search modpacks":"Search mods"}/></div>
    {busy&&<div className="catalog-note">Loading {provider==="modrinth"?"Modrinth":"CurseForge"} results…</div>}
    {error&&<div className="catalog-note error">{error}</div>}
    {!busy&&!error&&hits.length===0&&<div className="catalog-note">No matching projects found.</div>}
    <div className="catalog-grid">{hits.map(x=><article className="catalog-card" key={x.id}>{x.image?<img src={x.image} alt="" loading="lazy"/>:<div className="catalog-placeholder"><Blocks size={20}/></div>}<h3>{x.title}</h3><p>{x.description}</p><div className="catalog-meta"><span>{Number(x.downloads||0).toLocaleString()} downloads</span><a href={x.url} target="_blank" rel="noreferrer">View project ↗</a></div></article>)}</div>
  </section>;
}
function LocalInstances() {
  const [items,setItems]=useState(()=>{try{return JSON.parse(localStorage.getItem("nova.instances")||"[]")}catch{return[]}});
  const [name,setName]=useState("");
  const [version,setVersion]=useState("1.21.4");
  useEffect(()=>{localStorage.setItem("nova.instances",JSON.stringify(items))},[items]);
  const add=()=>{if(!name.trim())return;setItems(a=>[...a,{id:Date.now(),name:name.trim(),version}]);setName("")};
  return <section className="local-instances"><div className="eyebrow">LOCAL PROFILES</div><h2>Your instances</h2><p>Create local profiles to organize your Minecraft setups. Game installation and launching are still in development.</p><div className="instance-create"><input value={name} onChange={e=>setName(e.target.value)} onKeyDown={e=>e.key==="Enter"&&add()} placeholder="Instance name"/><select value={version} onChange={e=>setVersion(e.target.value)}>{versions.map(v=><option key={v}>{v}</option>)}</select><button className="primary-btn" onClick={add} disabled={!name.trim()}><Plus size={14}/> Create</button></div>{items.length===0?<div className="catalog-note">No profiles yet. Create your first instance above.</div>:items.map(x=><div className="instance-entry" key={x.id}><div className="minecraft-icon"><Box size={20}/></div><div className="quick-info"><b>{x.name}</b><span>Minecraft Java · {x.version}</span></div><button className="delete-entry" onClick={()=>setItems(a=>a.filter(y=>y.id!==x.id))}>Delete</button></div>)}</section>;
}

function EmptyPage({ page, setPage }) {
  const content = {
    instances: { icon: Layers3, title: "Your instances", desc: "Keep separate Minecraft versions, mod loaders, and configurations organized.", action: "Create an instance", note: "Instance installation is not connected yet." },
    mods: { icon: Blocks, title: "Mod library", desc: "Find the performance, utility, and PvP mods you want in one place.", action: "Browse mod sources", note: "Modrinth and CurseForge integration is planned." },
    modpacks: { icon: Box, title: "Modpacks", desc: "Explore curated packs or bring your own pack into NOVA.", action: "Explore modpacks", note: "Pack browsing and installation are not connected yet." },
    pvp: { icon: Gamepad2, title: "PvP toolkit", desc: "A focused set of HUD and quality-of-life tools for competitive play.", action: "Explore modules", note: "No gameplay modules are enabled in this preview." },
    settings: { icon: SlidersHorizontal, title: "Preferences", desc: "Tune the launcher experience and prepare your client setup.", action: "Back to overview", note: "System settings and account connections are coming later." }
  }[page] || { icon: Compass, title: "NOVA", desc: "Your Minecraft, upgraded.", action: "Back to overview", note: "" };
  const Icon = content.icon;
  return <div className="empty-state"><div className="empty-icon"><Icon size={27}/></div><div className="eyebrow">NOVA WORKSPACE</div><h2>{content.title}</h2><p>{content.desc}</p><div className="empty-note"><span className="note-mark"><Activity size={15}/></span><span>{content.note}</span></div><button className="primary-btn" onClick={() => setPage("home")}>{content.action}<ArrowUpRight size={15}/></button></div>;
}

export default function App() {
  const [page, setPage] = useState("home");
  const [selectedVersion, setSelectedVersion] = useState("1.21.4");
  const [launching, setLaunching] = useState(false);
  const titles = { home: ["Overview", "Everything you need, in one place."], instances: ["Instances", "Your game versions, neatly organized."], mods: ["Mods", "Build the Minecraft setup you want."], modpacks: ["Modpacks", "Discover a new way to play."], pvp: ["PvP features", "A focused toolkit for competitive play."], settings: ["Settings", "Make NOVA feel like yours."] };
  const [title, subtitle] = titles[page] || titles.home;
  return <div className="app-shell"><Sidebar page={page} setPage={setPage}/><main className="main-area"><Topbar title={title} subtitle={subtitle}/><div className="page-content">{page === "home" ? <Home onNavigate={setPage} selectedVersion={selectedVersion} setSelectedVersion={setSelectedVersion} launching={launching} setLaunching={setLaunching}/> : page === "mods" ? <Catalog packs={false}/> : page === "modpacks" ? <Catalog packs={true}/> : page === "instances" ? <LocalInstances/> : <EmptyPage page={page} setPage={setPage}/>}</div><footer className="main-footer"><span>© 2026 NOVA CLIENT</span><span>BUILT FOR THE NEXT SESSION <Sparkles size={12}/></span></footer></main></div>;
}

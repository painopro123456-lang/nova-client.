import { MICROSOFT_CLIENT_ID } from "./config";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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

function Home({ onNavigate, selectedVersion, setSelectedVersion, launching, onLaunch, onLogin, signedIn, accountName, loginBusy, loginUrl, message }) {
  return <>
    <div className="hero-panel">
      <div className="hero-glow"/>
      <div className="hero-content"><div className="hero-kicker"><span className="kicker-icon"><Sparkles size={13}/></span> YOUR NEXT SESSION STARTS HERE</div><h2>Make every<br/><em>frame count.</em></h2><p>A cleaner way to launch Minecraft. Your worlds, your settings, your pace.</p><div className="hero-actions"><button className="primary-btn" onClick={onLaunch} disabled={launching || !signedIn}><Play size={16} fill="currentColor"/>{launching ? "Preparing Minecraft…" : "Play Minecraft"}<ArrowUpRight size={15}/></button><button className="secondary-btn" onClick={onLogin} disabled={loginBusy || signedIn}>{signedIn ? `Signed in${accountName ? ` · ${accountName}` : ""}` : loginBusy ? "Waiting for Microsoft…" : "Sign in with Microsoft"}</button><button className="secondary-btn" onClick={() => onNavigate("instances")}>Manage instances</button></div><div className="hero-foot"><span><ShieldCheck size={14}/> Microsoft account stored securely on this PC</span><span><Gauge size={14}/> Vanilla Java Edition</span></div>{loginUrl && !signedIn && <a className="text-button" href={loginUrl} target="_blank" rel="noreferrer">Continue Microsoft sign-in ↗</a>}{message && <p className="launch-message" role="status">{message}</p>}</div>
      <div className="orbit-art" aria-hidden="true"><div className="orbit-ring ring-one"/><div className="orbit-ring ring-two"/><div className="orbit-core"><Sparkles size={48}/></div><div className="orbit-chip chip-one"><Cpu size={15}/> PERFORMANCE</div><div className="orbit-chip chip-two"><Gamepad2 size={15}/> BUILT FOR PLAY</div></div>
      <div className="hero-index">01 <span>/ 04</span></div>
    </div>
    <div className="section-heading"><div><div className="eyebrow">READY WHEN YOU ARE</div><h2>Quick launch</h2></div><button className="text-button" onClick={() => onNavigate("instances")}>All instances <ArrowUpRight size={14}/></button></div>
    <div className="quick-card"><div className="minecraft-icon"><Box size={22}/></div><div className="quick-info"><b>Vanilla Minecraft</b><span>Clean profile · No extra mods</span></div><div className="version-select"><label>GAME VERSION</label><select value={selectedVersion} onChange={e => setSelectedVersion(e.target.value)}>{versions.map(v => <option key={v}>{v}</option>)}</select></div><button className="small-play" onClick={onLaunch} disabled={launching || !signedIn}><Play size={15} fill="currentColor"/> Launch</button></div>
    <div className="feature-grid"><div className="feature-card"><div className="feature-icon purple"><Gauge size={18}/></div><b>Performance first</b><p>A native Rust shell is the foundation. No performance claims until benchmarks exist.</p><span className="feature-status">IN DEVELOPMENT</span></div><div className="feature-card"><div className="feature-icon blue"><Blocks size={18}/></div><b>Your mod library</b><p>Planned browsing and installs through trusted mod platforms.</p><span className="feature-status">PLANNED</span></div><div className="feature-card"><div className="feature-icon green"><ShieldCheck size={18}/></div><b>Profiles that stay yours</b><p>Keep game versions and configurations organized by instance.</p><span className="feature-status">PLANNED</span></div></div>
  </>;
}


function Catalog({ packs }) {
  const [q,setQ]=useState("");
  const [provider,setProvider]=useState("modrinth");
  const [apiKey,setApiKey]=useState("");
  const [keySaved,setKeySaved]=useState(false);
  const [keySaving,setKeySaving]=useState(false);
  const [hits,setHits]=useState([]);
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState("");
  const [page,setPage]=useState(0);

  useEffect(()=>{
    let alive=true;
    invoke("has_curseforge_key").then(value=>{if(alive)setKeySaved(Boolean(value));}).catch(()=>{if(alive)setKeySaved(false);});
    return ()=>{alive=false;};
  },[]);

  useEffect(()=>{
    const ctrl=new AbortController();
    const timer=setTimeout(async()=>{
      if(provider==="curseforge"&&!keySaved){
        setHits([]);
        setBusy(false);
        setError("Add your CurseForge API key below. NOVA will store it in Windows Credential Manager.");
        return;
      }
      setBusy(true); setError("");
      try {
        let results=[];
        if(provider==="modrinth") {
          const p=new URLSearchParams({query:q,facets:JSON.stringify([[packs?"project_type:modpack":"project_type:mod"]]),limit:"24",offset:String(page*24)});
          const r=await fetch("https://api.modrinth.com/v2/search?"+p,{signal:ctrl.signal});
          if(!r.ok) throw new Error("Modrinth catalog request failed");
          const data=await r.json();
          results=(data.hits||[]).map(x=>({id:x.project_id,title:x.title,description:x.description,downloads:x.downloads,image:x.icon_url,url:"https://modrinth.com/"+(packs?"modpack/":"mod/")+x.slug}));
        } else {
          const data=await invoke("curseforge_search",{query:q,packs,index:page*24,gameVersion:null});
          results=(data.data||[]).map(x=>({id:String(x.id),title:x.name,description:x.summary||"No description provided.",downloads:x.downloadCount,image:x.logo?.thumbnailUrl,url:x.links?.websiteUrl||"https://www.curseforge.com/minecraft/"+(packs?"modpacks":"mc-mods")+"/"+x.slug}));
        }
        if(!ctrl.signal.aborted) setHits(results);
      } catch(e) { if(!ctrl.signal.aborted) setError(typeof e==="string"?e:(e.message||"Unable to load catalog. Check your connection and retry.")); }
      finally { if(!ctrl.signal.aborted) setBusy(false); }
    },250);
    return ()=>{clearTimeout(timer);ctrl.abort();};
  },[q,packs,provider,keySaved,page]);

  const saveKey=async()=>{
    if(!apiKey.trim())return;
    setKeySaving(true);setError("");
    try{
      await invoke("save_curseforge_key",{apiKey:apiKey.trim()});
      setApiKey("");setKeySaved(true);setPage(0);
    }catch(e){setError(typeof e==="string"?e:(e.message||"Could not save the key to Windows Credential Manager."));}
    finally{setKeySaving(false);}
  };
  const forgetKey=async()=>{
    try{await invoke("delete_curseforge_key");setKeySaved(false);setHits([]);setError("CurseForge key removed from Windows Credential Manager.");}
    catch(e){setError(typeof e==="string"?e:(e.message||"Could not remove the saved key."));}
  };

  return <section className="catalog-page">
    <div className="catalog-hero">
      <div className="catalog-hero-copy">
        <div className="eyebrow"><span className="live-indicator"/> NOVA DISCOVERY / {packs?"MODPACKS":"MODS"}</div>
        <h2>{packs?"A whole new world.":"Make Minecraft yours."}</h2>
        <p>{packs?"Find complete experiences built by the community.":"Discover the tools, visuals and quality-of-life upgrades you want."}</p>
        <div className="catalog-hero-meta"><span><Sparkles size={13}/> Community-powered</span><span><ShieldCheck size={13}/> Source-linked projects</span></div>
      </div>
      <div className="catalog-hero-art" aria-hidden="true"><div className="catalog-art-orbit orbit-a"/><div className="catalog-art-orbit orbit-b"/><div className="catalog-art-cube"><Blocks size={43}/></div><div className="catalog-art-star"><Sparkles size={18}/></div></div>
    </div>
    <div className="catalog-section-head">
      <div><div className="eyebrow">THE LIBRARY</div><h3>Explore {packs?"modpacks":"mods"}</h3><p>Search across trusted community platforms.</p></div>
      <div className="catalog-provider" role="group" aria-label="Catalog provider">
        <button className={provider==="modrinth"?"provider-choice selected":"provider-choice"} onClick={()=>{setProvider("modrinth");setPage(0);}}>Modrinth</button>
        <button className={provider==="curseforge"?"provider-choice selected":"provider-choice"} onClick={()=>{setProvider("curseforge");setPage(0);}}>CurseForge</button>
      </div>
    </div>
    {provider==="curseforge"&&<div className="curseforge-key">
      <div className="key-heading"><div className="key-lock"><ShieldCheck size={16}/></div><div><b>CurseForge connection</b><span>Connect your approved API key to search the catalog.</span></div></div>
      {keySaved
        ? <div className="key-saved-row"><span><Check size={14}/> Connected · stored in Windows Credential Manager</span><button className="delete-entry" onClick={forgetKey}>Remove key</button></div>
        : <div className="key-input-row"><input id="curseforge-api-key" type="password" autoComplete="off" value={apiKey} onChange={e=>setApiKey(e.target.value)} onKeyDown={e=>e.key==="Enter"&&saveKey()} placeholder="Paste your CurseForge API key"/><button className="primary-btn key-save-button" onClick={saveKey} disabled={!apiKey.trim()||keySaving}>{keySaving?"Saving…":"Connect securely"}</button></div>}
      <span className="key-footnote">Your key stays on this PC and is never placed in NOVA's frontend source.</span>
    </div>}
    <div className="catalog-search-row">
      <div className="catalog-search"><Search size={17}/><input value={q} onChange={e=>{setQ(e.target.value);setPage(0);}} placeholder={packs?"Search modpacks, themes, adventures…":"Search mods, performance, shaders, utilities…"}/>{q&&<button className="clear-search" onClick={()=>setQ("")} aria-label="Clear search"><X size={14}/></button>}</div>
      <div className="catalog-source-pill"><span className="source-dot"/> {provider==="modrinth"?"MODRINTH INDEX":"CURSEFORGE API"}</div>
    </div>
    {busy&&<div className="catalog-state"><div className="loading-orbit"/><b>Finding great projects</b><span>Searching {provider==="modrinth"?"Modrinth":"CurseForge"}…</span></div>}
    {error&&<div className="catalog-note error"><CircleHelp size={16}/><span>{error}</span></div>}
    {!busy&&!error&&hits.length===0&&<div className="catalog-state"><div className="empty-library"><Blocks size={25}/></div><b>No projects found</b><span>Try another search or switch catalog source.</span></div>}
    {!busy&&hits.length>0&&<div className="catalog-results-label"><span>{hits.length} PROJECTS ON THIS PAGE</span><span>PAGE {String(page+1).padStart(2,"0")}</span></div>}
    <div className="catalog-grid">{hits.map((x,i)=><article className="catalog-card" key={x.id}>
      <div className="project-card-top">{x.image?<img src={x.image} alt="" loading="lazy"/>:<div className="catalog-placeholder"><Blocks size={20}/></div>}<span className="project-source">{provider==="modrinth"?"MODRINTH":"CURSEFORGE"}</span><span className="project-rank">{String(page*24+i+1).padStart(2,"0")}</span></div>
      <h3>{x.title}</h3><p>{x.description}</p>
      <div className="catalog-meta"><span><ArrowDownToLine size={12}/>{Number(x.downloads||0).toLocaleString()}</span><a href={x.url} target="_blank" rel="noreferrer">View project <ArrowUpRight size={12}/></a></div>
    </article>)}</div>}
    <div className="catalog-pagination"><button className="secondary-btn" disabled={page===0||busy} onClick={()=>setPage(p=>Math.max(0,p-1))}>← Previous</button><span>{String(page+1).padStart(2,"0")} <i/> {provider==="modrinth"?"DISCOVER":"CURSEFORGE"}</span><button className="secondary-btn" disabled={busy||hits.length===0} onClick={()=>setPage(p=>p+1)}>Next →</button></div>
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
    settings: { icon: SlidersHorizontal, title: "Preferences", desc: "Tune the launcher experience and prepare your client setup.", action: "Back to overview", note: "Microsoft app ID configured (" + MICROSOFT_CLIENT_ID + "). Microsoft sign-in flow is still being implemented." }
  }[page] || { icon: Compass, title: "NOVA", desc: "Your Minecraft, upgraded.", action: "Back to overview", note: "" };
  const Icon = content.icon;
  return <div className="empty-state"><div className="empty-icon"><Icon size={27}/></div><div className="eyebrow">NOVA WORKSPACE</div><h2>{content.title}</h2><p>{content.desc}</p><div className="empty-note"><span className="note-mark"><Activity size={15}/></span><span>{content.note}</span></div><button className="primary-btn" onClick={() => setPage("home")}>{content.action}<ArrowUpRight size={15}/></button></div>;
}

export default function App() {
  const [page, setPage] = useState("home");
  const [selectedVersion, setSelectedVersion] = useState("1.21.4");
  const [launching, setLaunching] = useState(false);
  const [signedIn, setSignedIn] = useState(false);
  const [accountName, setAccountName] = useState("");
  const [loginBusy, setLoginBusy] = useState(false);
  const [loginUrl, setLoginUrl] = useState("");
  const [message, setMessage] = useState("");
  useEffect(() => {
    let alive = true;
    let offLink;
    let offComplete;
    (async () => {
      offLink = await listen("nova-login-link", event => { if (alive) setLoginUrl(event.payload); });
      offComplete = await listen("nova-login-complete", event => { if (alive) { setSignedIn(true); setAccountName(event.payload); setLoginUrl(""); setMessage("Microsoft account connected."); } });
      try {
        const name = await invoke("minecraft_account_name");
        if (alive && name) { setSignedIn(true); setAccountName(name); }
      } catch (error) { if (alive) setMessage(String(error)); }
    })();
    return () => { alive = false; offLink?.(); offComplete?.(); };
  }, []);
  const handleLogin = async () => {
    setLoginBusy(true); setMessage("A Microsoft sign-in link will appear here. Complete sign-in in your browser.");
    try { const name = await invoke("begin_microsoft_login"); setSignedIn(true); setAccountName(name); setLoginUrl(""); setMessage(`Signed in as ${name}.`); }
    catch (error) { setMessage(typeof error === "string" ? error : (error.message || "Microsoft sign-in failed.")); }
    finally { setLoginBusy(false); }
  };
  const handleLaunch = async () => {
    if (!signedIn) { setMessage("Sign in with Microsoft before launching Minecraft."); return; }
    setLaunching(true); setMessage("Preparing Minecraft. The first launch downloads game files and Java runtime.");
    try { await invoke("launch_minecraft", { version: selectedVersion }); setMessage("Minecraft has been started. Keep NOVA open while the game loads."); }
    catch (error) { setMessage(typeof error === "string" ? error : (error.message || "Minecraft could not be started.")); }
    finally { setLaunching(false); }
  };
  const titles = { home: ["Overview", "Everything you need, in one place."], instances: ["Instances", "Your game versions, neatly organized."], mods: ["Mods", "Build the Minecraft setup you want."], modpacks: ["Modpacks", "Discover a new way to play."], pvp: ["PvP features", "A focused toolkit for competitive play."], settings: ["Settings", "Make NOVA feel like yours."] };
  const [title, subtitle] = titles[page] || titles.home;
  return <div className="app-shell"><Sidebar page={page} setPage={setPage}/><main className="main-area"><Topbar title={title} subtitle={subtitle}/><div className="page-content">{page === "home" ? <Home onNavigate={setPage} selectedVersion={selectedVersion} setSelectedVersion={setSelectedVersion} launching={launching} onLaunch={handleLaunch} onLogin={handleLogin} signedIn={signedIn} accountName={accountName} loginBusy={loginBusy} loginUrl={loginUrl} message={message}/> : page === "mods" ? <Catalog packs={false}/> : page === "modpacks" ? <Catalog packs={true}/> : page === "instances" ? <LocalInstances/> : <EmptyPage page={page} setPage={setPage}/>}</div><footer className="main-footer"><span>© 2026 NOVA CLIENT</span><span>BUILT FOR THE NEXT SESSION <Sparkles size={12}/></span></footer></main></div>;
}

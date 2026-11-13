import { MICROSOFT_CLIENT_ID } from "./config";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Activity, ArrowDownToLine, ArrowUpRight, Bell, Blocks, Box,
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


function readStored(key, fallback, validate) {
  try { const value=JSON.parse(localStorage.getItem(key) ?? JSON.stringify(fallback)); return validate(value)?value:fallback; }
  catch { return fallback; }
}
function writeStored(key,value) { try { localStorage.setItem(key,JSON.stringify(value)); } catch { /* Storage may be unavailable or full. */ } }
function Sidebar({ page, setPage, signedIn, accountName, onSignOut }) {
  return <aside className="sidebar">
    <div className="brand"><div className="brand-mark"><Moon size={19}/></div><div><b>NOVA</b><span>CLIENT</span></div><span className="build-tag">BETA</span></div>
    <div className="profile-card"><div className="avatar">{signedIn?(accountName||"N").slice(0,1).toUpperCase():"N"}</div><div className="profile-copy"><b>{signedIn?accountName||"Microsoft account":"Guest profile"}</b><span>{signedIn?"Microsoft account connected":"Sign in to continue"}</span></div>{signedIn&&<button className="text-button" onClick={onSignOut}>Sign out</button>}</div>
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

function Home({ onNavigate, selectedVersion, setSelectedVersion, versions, versionsLoading, loaderType, setLoaderType, memoryMax, launching, onLaunch, onLogin, signedIn, accountName, loginBusy, loginUrl, message }) {
  return <>
    <div className="hero-panel">
      <div className="hero-glow"/>
      <div className="hero-content"><div className="hero-kicker"><span className="kicker-icon"><Sparkles size={13}/></span> YOUR NEXT SESSION STARTS HERE</div><h2>Make every<br/><em>frame count.</em></h2><p>A cleaner way to launch Minecraft. Your worlds, your settings, your pace.</p><div className="hero-actions"><button className="primary-btn" onClick={onLaunch} disabled={launching || !signedIn || versionsLoading || !selectedVersion}><Play size={16} fill="currentColor"/>{launching ? "Preparing Minecraft…" : "Play Minecraft"}<ArrowUpRight size={15}/></button><button className="secondary-btn" onClick={onLogin} disabled={loginBusy || signedIn}>{signedIn ? `Signed in${accountName ? ` · ${accountName}` : ""}` : loginBusy ? "Waiting for Microsoft…" : "Sign in with Microsoft"}</button><button className="secondary-btn" onClick={() => onNavigate("instances")}>Manage instances</button></div><div className="hero-foot"><span><ShieldCheck size={14}/> Microsoft account stored securely on this PC</span><span><Gauge size={14}/> {loaderType === "fabric" ? "Fabric modded" : "Vanilla Java Edition"}</span></div>{loginUrl && !signedIn && <a className="text-button" href={loginUrl} target="_blank" rel="noreferrer">Continue Microsoft sign-in ↗</a>}{message && <p className="launch-message" role="status">{message}</p>}</div>
      <div className="orbit-art" aria-hidden="true"><div className="orbit-ring ring-one"/><div className="orbit-ring ring-two"/><div className="orbit-core"><Sparkles size={48}/></div><div className="orbit-chip chip-one"><Cpu size={15}/> PERFORMANCE</div><div className="orbit-chip chip-two"><Gamepad2 size={15}/> BUILT FOR PLAY</div></div>
      <div className="hero-index">01 <span>/ 04</span></div>
    </div>
    <div className="section-heading"><div><div className="eyebrow">READY WHEN YOU ARE</div><h2>Quick launch</h2></div><button className="text-button" onClick={() => onNavigate("instances")}>All instances <ArrowUpRight size={14}/></button></div>
    <div className="quick-card"><div className="minecraft-icon"><Box size={22}/></div><div className="quick-info"><b>{loaderType === "fabric" ? "Fabric Minecraft" : "Vanilla Minecraft"}</b><span>{selectedVersion || "Select a game version"} · {loaderType === "fabric" ? "Fabric Loader" : "Clean profile"}</span></div><div className="version-select"><label>GAME VERSION</label><select value={selectedVersion} onChange={e => setSelectedVersion(e.target.value)} disabled={versionsLoading || versions.length === 0}><option value="" disabled>{versionsLoading ? "Loading versions…" : "Choose a version"}</option>{versions.map(v => <option key={v.id} value={v.id}>{v.id}{v.type === "snapshot" ? " · Snapshot" : ""}</option>)}</select></div><div className="version-select"><label>MOD LOADER</label><select value={loaderType} onChange={e => setLoaderType(e.target.value)}><option value="vanilla">Vanilla</option><option value="fabric">Fabric</option></select></div><button className="small-play" onClick={onLaunch} disabled={launching || !signedIn || versionsLoading || !selectedVersion}><Play size={15} fill="currentColor"/> Launch</button></div>
    <div className="feature-grid"><div className="feature-card"><div className="feature-icon purple"><Gauge size={18}/></div><b>Performance first</b><p>A native Rust shell is the foundation. No performance claims until benchmarks exist.</p><span className="feature-status">IN DEVELOPMENT</span></div><div className="feature-card"><div className="feature-icon blue"><Blocks size={18}/></div><b>Your mod library</b><p>Planned browsing and installs through trusted mod platforms.</p><span className="feature-status">PLANNED</span></div><div className="feature-card"><div className="feature-icon green"><ShieldCheck size={18}/></div><b>Profiles that stay yours</b><p>Keep game versions and configurations organized by instance.</p><span className="feature-status">PLANNED</span></div></div>
  </>;
}


function Catalog({ packs, memoryMax, onNavigate }) {
  const [q,setQ]=useState("");
  const [provider,setProvider]=useState("modrinth");
  const [hits,setHits]=useState([]);
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState("");
  const [page,setPage]=useState(0);
  const [hasNext,setHasNext]=useState(false);
  const [installed,setInstalled]=useState(()=>readStored("nova.installedPacks",{},v=>v&&typeof v==="object"&&!Array.isArray(v)));
  const [instances,setInstances]=useState(()=>readStored("nova.instances",[],Array.isArray));
  const [targetInstance,setTargetInstance]=useState(()=>{const custom=readStored("nova.instances",[],Array.isArray);const packs=readStored("nova.installedPacks",{},v=>v&&typeof v==="object"&&!Array.isArray(v));return custom[0]?.instanceName||Object.values(packs)[0]?.instanceName||"";});
  const [modInstallBusy,setModInstallBusy]=useState("");
  const [modInstallMessage,setModInstallMessage]=useState("");
  const [installingId,setInstallingId]=useState("");
  const [packActionMessage,setPackActionMessage]=useState("");
  const [versionProject,setVersionProject]=useState(null);
  const [packVersions,setPackVersions]=useState([]);
  const [packVersionsBusy,setPackVersionsBusy]=useState(false);
  const [selectedPackVersion,setSelectedPackVersion]=useState("");
  const [installDialog,setInstallDialog]=useState(()=>readStored("nova.activeInstall",null,v=>v===null||(v&&typeof v==="object")));

  useEffect(()=>{
    const ctrl=new AbortController();
    setBusy(true); setError(""); setHits([]); setHasNext(false);
    const timer=setTimeout(async()=>{
      try {
        let results=[];
        let canContinue=false;
        if(provider==="modrinth") {
          const p=new URLSearchParams({query:q,facets:JSON.stringify([[packs?"project_type:modpack":"project_type:mod"]]),limit:"24",offset:String(page*24)});
          const r=await fetch("https://api.modrinth.com/v2/search?"+p,{signal:ctrl.signal});
          if(!r.ok) throw new Error("Modrinth catalog request failed");
          const data=await r.json();
          results=(data.hits||[]).map(x=>({id:x.project_id,title:x.title,description:x.description,downloads:x.downloads,image:x.icon_url,url:"https://modrinth.com/"+(packs?"modpack/":"mod/")+x.slug}));
          canContinue=(page+1)*24 < Number(data.total_hits||0);
        } else {
          const data=await invoke("curseforge_search",{query:q,packs,index:page*24,gameVersion:null});
          results=(data.data||[]).map(x=>({id:String(x.id),title:x.name,description:x.summary||"No description provided.",downloads:x.downloadCount,image:x.logo?.thumbnailUrl,url:x.links?.websiteUrl||"https://www.curseforge.com/minecraft/"+(packs?"modpacks":"mc-mods")+"/"+x.slug}));
          canContinue=(page+1)*24 < Number(data.pagination?.totalCount||0);
        }
        if(!ctrl.signal.aborted) { setHits(results); setHasNext(canContinue); }
      } catch(e) { if(!ctrl.signal.aborted) setError(typeof e==="string"?e:(e.message||"Unable to load catalog. Check your connection and retry.")); }
      finally { if(!ctrl.signal.aborted) setBusy(false); }
    },250);
    return ()=>{clearTimeout(timer);ctrl.abort();};
  },[q,packs,provider,page]);

  useEffect(()=>{writeStored("nova.installedPacks",installed);},[installed]);
  useEffect(()=>{
    const refresh=()=>{const next=readStored("nova.instances",[],Array.isArray);const packs=readStored("nova.installedPacks",{},v=>v&&typeof v==="object"&&!Array.isArray(v));setInstances(next);setTargetInstance(current=>(next.some(item=>item.instanceName===current)||Object.values(packs).some(item=>item.instanceName===current))?current:(next[0]?.instanceName||Object.values(packs)[0]?.instanceName||""));};
    window.addEventListener("nova-instances-recovered",refresh);
    return()=>window.removeEventListener("nova-instances-recovered",refresh);
  },[]);
  const availableInstances=[...instances,...Object.values(installed)].filter((item,index,list)=>item?.instanceName&&list.findIndex(other=>other?.instanceName===item.instanceName)===index);
  const installMod=async(project)=>{
    if(!targetInstance){setModInstallMessage("Create an instance first, then select it here.");return;}
    setModInstallBusy(String(project.id));setModInstallMessage("Installing "+project.title+"…");
    try{
      const result=await invoke("install_mod_file",{provider,projectId:String(project.id),instanceName:targetInstance});
      setModInstallMessage((result.fileName||project.title)+" installed in "+targetInstance+".");
    }catch(error){setModInstallMessage(typeof error==="string"?error:(error.message||"Could not install this mod."));}
    finally{setModInstallBusy("");}
  };
  useEffect(()=>{
    let alive=true;
    let unlisten;
    listen("nova-pack-progress",event=>{
      const progress=event.payload||{};
      setInstallDialog(current=>current?{
        ...current,
        progress:typeof progress.percent==="number"?progress.percent:null,
        status:progress.phase==="preparing"?"Preparing Minecraft":progress.phase==="archive"?"Downloading modpack archive":"Downloading modpack files",
        detail:progress.detail||current.detail
      }:current);
    }).then(stop=>{if(alive)unlisten=stop;else stop();});
    return()=>{alive=false;unlisten?.();};
  },[]);

  const choosePackVersion=async(project)=>{
    setVersionProject(project);setPackVersions([]);setSelectedPackVersion("");setPackVersionsBusy(true);setPackActionMessage("");
    try{
      let versions=[];
      if(provider==="modrinth"){
        const response=await fetch("https://api.modrinth.com/v2/project/"+encodeURIComponent(project.id)+"/version");
        if(!response.ok)throw new Error("Could not load modpack versions from Modrinth.");
        const data=await response.json();
        versions=(data||[]).map(v=>({id:v.id,label:(v.name||v.version_number||v.id)+" · MC "+(v.game_versions||[]).join(", ")+" · "+(v.loaders||[]).join("/"),published:v.date_published||""}));
      }else{
        const data=await invoke("curseforge_modpack_versions",{modId:String(project.id)});
        versions=(data||[]).map(v=>({id:String(v.id),label:(v.displayName||v.fileName||("File "+v.id))+" · MC "+((v.gameVersions||[]).join(", ")||"version not listed"),published:v.fileDate||""}));
      }
      if(!versions.length)throw new Error("This modpack has no downloadable versions.");
      setPackVersions(versions);setSelectedPackVersion(versions[0].id);
    }catch(e){setPackActionMessage(typeof e==="string"?e:(e.message||"Could not load modpack versions."));setVersionProject(null);}
    finally{setPackVersionsBusy(false);}
  };
  const installPack=async()=>{
    if(!versionProject||!selectedPackVersion)return;
    const project=versionProject;
    const projectId=String(project.id);
    const storageKey=provider+":"+projectId;
    const uniqueSuffix=(typeof crypto!=="undefined"&&crypto.randomUUID)?crypto.randomUUID().slice(0,8):Math.random().toString(36).slice(2,10);
    const instanceName=(project.title.toLowerCase().replace(/[^a-z0-9]+/g,"-").replace(/^-|-$/g,"").slice(0,24)||"modpack")+"-"+Date.now().toString(36)+"-"+uniqueSuffix;
    setInstallingId(projectId);
    setVersionProject(null);
    const initialProgress={name:project.title,instanceName,step:0,progress:null,status:"Starting installation",detail:"Preparing your isolated modpack instance…",done:false,error:""};
    setInstallDialog(initialProgress); writeStored("nova.activeInstall",initialProgress);
    try{
      const downloadProgress={name:project.title,instanceName,step:0,progress:null,status:"Downloading modpack",detail:"Getting the selected release and its manifest…",done:false,error:""};
      setInstallDialog(downloadProgress); writeStored("nova.activeInstall",downloadProgress);
      const metadata=provider==="modrinth"
        ? await invoke("install_modrinth_modpack",{projectId,instanceName,versionId:selectedPackVersion})
        : await invoke("install_curseforge_modpack",{modId:projectId,instanceName,fileId:Number(selectedPackVersion)});
      const preparingProgress={name:metadata.name||project.title,instanceName,step:1,progress:null,status:"Preparing Minecraft",detail:"Installing Minecraft "+metadata.minecraftVersion+" and the Java runtime…",done:false,error:""};
      setInstallDialog(preparingProgress); writeStored("nova.activeInstall",preparingProgress);
      await invoke("prepare_modpack_instance",{instanceName:metadata.instanceName,memoryMax});
      // Only expose a pack as installed after Minecraft and its loader are ready.
      // Remove the previous isolated install only after the replacement is ready.
      const previous=installed[storageKey];
      if(previous?.instanceName&&previous.instanceName!==metadata.instanceName){
        try{await invoke("delete_instance",{instanceName:previous.instanceName});}catch{/* Keep the new working install even if stale cleanup fails. */}
      }
      setInstalled(old=>({...old,[storageKey]:metadata}));
      const completeProgress={name:metadata.name||project.title,instanceName,step:2,progress:100,status:"Installation complete",detail:"Minecraft "+metadata.minecraftVersion+" · "+metadata.loader+(metadata.loaderVersion?" "+metadata.loaderVersion:"")+" is ready to play.",done:true,error:""};
      setInstallDialog(completeProgress); writeStored("nova.activeInstall",completeProgress);
      setPackActionMessage((metadata.name||project.title)+" is fully installed and ready to play.");
    }catch(e){
      const reason=typeof e==="string"?e:(e.message||"Modpack installation failed.");
      try{await invoke("delete_instance",{instanceName});}catch{/* Ignore cleanup errors; show the original installation failure. */}
      setInstallDialog(current=>{const failed={...current,done:true,error:reason,status:"Installation failed",detail:"The incomplete instance was cleaned up where possible. You can safely retry."};writeStored("nova.activeInstall",failed);return failed;});
      setPackActionMessage((project.title)+" installation failed: "+reason);
    }finally{setInstallingId("");}
  };
  const launchPack=async metadata=>{
    setInstallingId(String(metadata.projectId));setPackActionMessage(`Preparing ${metadata.name}…`);
    try{
      await invoke("launch_minecraft",{version:metadata.minecraftVersion,loader:metadata.loader,loaderBuild:metadata.loaderVersion,instanceName:metadata.instanceName,memoryMax});
      setPackActionMessage(`${metadata.name} has been started.`);
    }catch(e){setPackActionMessage(typeof e==="string"?e:(e.message||"Could not launch this modpack."));}
    finally{setInstallingId("");}
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
    <div className="catalog-search-row">
      {!packs&&<div className="catalog-source-pill"><span>INSTALL INTO</span><select value={targetInstance} onChange={e=>setTargetInstance(e.target.value)} disabled={!availableInstances.length}><option value="">{availableInstances.length?"Choose instance":"No instances"}</option>{availableInstances.map(instance=><option key={instance.instanceName} value={instance.instanceName}>{instance.name||instance.instanceName}</option>)}</select>{!availableInstances.length&&<button className="text-button" onClick={()=>onNavigate("instances")}>Create instance</button>}</div>}
      <div className="catalog-search"><Search size={17}/><input value={q} onChange={e=>{setQ(e.target.value);setPage(0);}} placeholder={packs?"Search modpacks, themes, adventures…":"Search mods, performance, shaders, utilities…"}/>{q&&<button className="clear-search" onClick={()=>setQ("")} aria-label="Clear search"><X size={14}/></button>}</div>
      <div className="catalog-source-pill"><span className="source-dot"/> {provider==="modrinth"?"MODRINTH INDEX":"CURSEFORGE API"}</div>
    </div>
    {busy&&<div className="catalog-state"><div className="loading-orbit"/><b>Finding great projects</b><span>Searching {provider==="modrinth"?"Modrinth":"CurseForge"}…</span></div>}
    {error&&<div className="catalog-note error"><CircleHelp size={16}/><span>{error}</span></div>}
    {packActionMessage&&packs&&<div className="catalog-note"><span>{packActionMessage}</span></div>}
    {modInstallMessage&&!packs&&<div className="catalog-note"><span>{modInstallMessage}</span>{!availableInstances.length&&<button className="text-button" onClick={()=>onNavigate("instances")}>Create instance</button>}</div>}
    {!busy&&!error&&hits.length===0&&<div className="catalog-state"><div className="empty-library"><Blocks size={25}/></div><b>No projects found</b><span>Try another search or switch catalog source.</span></div>}
    {!busy&&hits.length>0&&<div className="catalog-results-label"><span>{hits.length} PROJECTS ON THIS PAGE</span><span>PAGE {String(page+1).padStart(2,"0")}</span></div>}
    <div className="catalog-grid">{hits.map((x,i)=><article className="catalog-card" key={x.id}>
      <div className="project-card-top">{x.image?<img src={x.image} alt="" loading="lazy"/>:<div className="catalog-placeholder"><Blocks size={20}/></div>}<span className="project-source">{provider==="modrinth"?"MODRINTH":"CURSEFORGE"}</span><span className="project-rank">{String(page*24+i+1).padStart(2,"0")}</span></div>
      <h3>{x.title}</h3><p>{x.description}</p>
      <div className="catalog-meta"><span><ArrowDownToLine size={12}/>{Number(x.downloads||0).toLocaleString()}</span><a href={x.url} target="_blank" rel="noreferrer">View project <ArrowUpRight size={12}/></a></div>
      {packs?<div className="pack-install-row">{installed[provider+":"+String(x.id)]
        ? <button className="primary-btn" onClick={()=>launchPack(installed[provider+":"+String(x.id)])} disabled={Boolean(installingId)}><Play size={14} fill="currentColor"/> {installingId===String(x.id)?"Launching…":"Install & Play"}</button>
        : <button className="primary-btn" onClick={()=>choosePackVersion(x)} disabled={Boolean(installingId)}><ArrowDownToLine size={14}/> {installingId===String(x.id)?"Installing…" :"Choose version"}</button>}</div>
      : <div className="pack-install-row"><button className="primary-btn" onClick={()=>installMod(x)} disabled={!targetInstance||Boolean(modInstallBusy)}><ArrowDownToLine size={14}/>{modInstallBusy===String(x.id)?"Installing…":"Install mod"}</button></div>}
    </article>)}</div>}
    {installDialog&&<div className="pack-version-backdrop install-progress-backdrop" role="presentation">
      <section className="pack-version-dialog install-progress-dialog" role="dialog" aria-modal="true" aria-labelledby="install-progress-title">
        <div className="eyebrow">NOVA / MODPACK INSTALLER</div>
        <h3 id="install-progress-title">{installDialog.done?(installDialog.error?"Installation stopped":"You're all set"):"Installing your modpack"}</h3>
        <p className="install-progress-name">{installDialog.name}</p>
        <div className="install-progress-track"><div className={"install-progress-fill"+(!installDialog.done?" is-active":"")} style={{width:(typeof installDialog.progress==="number"?installDialog.progress:100)+"%"}}/></div>
        <div className="install-progress-percent">{typeof installDialog.progress==="number"?installDialog.progress+"%":"Working…"}</div>
        <div className="install-progress-status"><span className={installDialog.done?(installDialog.error?"progress-failed":"progress-complete"):"progress-running"}>{installDialog.done?(installDialog.error?"!":"✓"):""}</span><b>{installDialog.status}</b></div>
        <p className="install-progress-detail">{installDialog.detail}</p>
        <div className="install-progress-steps">
          <div className={installDialog.step>=0?"complete":""}><span>1</span><label>Modpack files</label></div>
          <div className={installDialog.step>=1?"complete":""}><span>2</span><label>Minecraft & Java</label></div>
          <div className={installDialog.step>=2?"complete":""}><span>3</span><label>Ready to play</label></div>
        </div>
        {installDialog.error&&<div className="install-progress-error">{installDialog.error}</div>}
        {!installDialog.done&&installDialog.step===0&&<div className="pack-version-actions"><button className="secondary-btn" onClick={async()=>{try{await invoke("cancel_modpack_install",{instanceName:installDialog.instanceName});setInstallDialog(current=>({...current,status:"Cancelling installation",detail:"Stopping the current download safely…"}));}catch(error){setInstallDialog(current=>({...current,detail:typeof error==="string"?error:(error.message||"Could not cancel this installation.")}));}}}>Cancel installation</button></div>}
        {installDialog.done&&<div className="pack-version-actions"><button className="primary-btn" onClick={()=>{setInstallDialog(null);writeStored("nova.activeInstall",null);}}>Close</button></div>}
      </section>
    </div>}
    {versionProject&&<div className="pack-version-backdrop" role="presentation" onClick={()=>!packVersionsBusy&&setVersionProject(null)}>
      <section className="pack-version-dialog" role="dialog" aria-modal="true" aria-labelledby="pack-version-title" onClick={e=>e.stopPropagation()}>
        <div className="eyebrow">MODPACK INSTALLER</div><h3 id="pack-version-title">Choose a version</h3><p>Select the exact modpack release you want NOVA to install. The Minecraft version and mod loader are included in each option.</p>
        {packVersionsBusy?<div className="catalog-state compact"><div className="loading-orbit"/><span>Loading available versions…</span></div>
          : <><label className="pack-version-label" htmlFor="pack-version-select">MODPACK VERSION</label><select id="pack-version-select" className="pack-version-select" value={selectedPackVersion} onChange={e=>setSelectedPackVersion(e.target.value)}>{packVersions.map(v=><option key={v.id} value={v.id}>{v.label}</option>)}</select><div className="pack-version-meta">{packVersions.find(v=>v.id===selectedPackVersion)?.published ? "Published "+new Date(packVersions.find(v=>v.id===selectedPackVersion).published).toLocaleDateString() : "Latest versions are listed first"}</div><div className="pack-version-actions"><button className="secondary-btn" onClick={()=>setVersionProject(null)}>Cancel</button><button className="primary-btn" onClick={installPack} disabled={!selectedPackVersion}><ArrowDownToLine size={14}/> Install selected version</button></div></>}
      </section>
    </div>}
    <div className="catalog-pagination"><button className="secondary-btn" disabled={page===0||busy} onClick={()=>setPage(p=>Math.max(0,p-1))}>← Previous</button><span>{String(page+1).padStart(2,"0")} <i/> {provider==="modrinth"?"DISCOVER":"CURSEFORGE"}</span><button className="secondary-btn" disabled={busy||!hasNext} onClick={()=>setPage(p=>p+1)}>Next →</button></div>
  </section>;
}
function LocalInstances({ versions, memoryMax }) {
  const [items,setItems]=useState(()=>readStored("nova.instances",[],Array.isArray));
  const [packs,setPacks]=useState(()=>readStored("nova.installedPacks",{},v=>v&&typeof v==="object"&&!Array.isArray(v)));
  const [name,setName]=useState("");
  const [version,setVersion]=useState("");
  const [loader,setLoader]=useState("fabric");
  const [action,setAction]=useState("");
  useEffect(()=>{if(!version && versions.length) setVersion(versions.find(v=>v.type==="release")?.id || versions[0].id);},[versions,version]);
  useEffect(()=>{writeStored("nova.instances",items)},[items]);
  useEffect(()=>{const recover=()=>{setItems(readStored("nova.instances",[],Array.isArray));setPacks(readStored("nova.installedPacks",{},v=>v&&typeof v==="object"&&!Array.isArray(v)));};window.addEventListener("nova-instances-recovered",recover);return()=>window.removeEventListener("nova-instances-recovered",recover);},[]);
  const add=async()=>{if(!name.trim()||!version)return;const instanceName=name.trim().toLowerCase().replace(/[^a-z0-9]+/g,"-").replace(/^-|-$/g,"").slice(0,48);setAction("Creating instance "+name.trim()+"…");try{const metadata=await invoke("create_instance",{instanceName,displayName:name.trim(),version,loader});setItems(a=>[...a,{id:Date.now(),...metadata}]);setName("");setAction((metadata.name||name.trim())+" created. Press Install & Play to install Minecraft and its loader.");}catch(e){setAction(typeof e==="string"?e:(e.message||"Could not create this instance."));}};
  const playPack=async pack=>{setAction("Installing Minecraft and loader for "+pack.name+" (if needed), then launching…");try{await invoke("launch_minecraft",{version:pack.minecraftVersion||pack.version,loader:pack.loader||"vanilla",loaderBuild:pack.loaderVersion||"latest",instanceName:pack.instanceName,memoryMax});setAction(pack.name+" started.");}catch(e){setAction(typeof e==="string"?e:(e.message||"Could not launch this instance."));}};
  const removePack=async(id,pack)=>{setAction("Deleting "+pack.name+"…");try{await invoke("delete_instance",{instanceName:pack.instanceName});setItems(old=>old.filter(x=>x.id!==id));setPacks(old=>{const next={...old};delete next[id];writeStored("nova.installedPacks",next);return next;});setAction(pack.name+" deleted.");}catch(e){setAction(typeof e==="string"?e:(e.message||"Could not delete this instance."));}};
  const installedPacks=Object.entries(packs);
  return <section className="local-instances"><div className="eyebrow">LOCAL PROFILES</div><h2>Your instances</h2><p>Each modpack gets its own isolated folder. NOVA installs the pack files, Minecraft, Java and the matching loader before marking the instance ready.</p>
    {installedPacks.length>0&&<><div className="section-heading"><div><div className="eyebrow">INSTALLED MODPACKS</div><h2>Your library</h2></div></div>{installedPacks.map(([id,pack])=><div className="instance-entry" key={id}><div className="minecraft-icon"><Box size={20}/></div><div className="quick-info"><b>{pack.name}</b><span>Minecraft {pack.minecraftVersion} · {pack.loader}{pack.loaderVersion?" "+pack.loaderVersion:""} · {pack.source}</span></div><button className="primary-btn" onClick={()=>playPack(pack)}><Play size={13} fill="currentColor"/> Play</button><button className="delete-entry" onClick={()=>removePack(id,pack)}>Delete</button></div>)}</>}
    <div className="section-heading"><div><div className="eyebrow">CUSTOM PROFILES</div><h2>Organize versions</h2></div></div><div className="instance-create"><input value={name} onChange={e=>setName(e.target.value)} onKeyDown={e=>e.key==="Enter"&&add()} placeholder="Instance name"/><select value={version} onChange={e=>setVersion(e.target.value)} disabled={!version||versions.length===0}>{versions.map(v=><option key={v.id} value={v.id}>{v.id}{v.type==="snapshot"?" · Snapshot":""}</option>)}</select><select value={loader} onChange={e=>setLoader(e.target.value)}><option value="fabric">Fabric</option><option value="forge">Forge</option><option value="neoforge">NeoForge</option><option value="quilt">Quilt</option><option value="vanilla">Vanilla</option></select><button className="primary-btn" onClick={add} disabled={!name.trim()||!version}><Plus size={14}/> Create instance</button></div>{items.length===0?<div className="catalog-note">No custom profiles yet.</div>:items.map(x=><div className="instance-entry" key={x.id}><div className="minecraft-icon"><Box size={20}/></div><div className="quick-info"><b>{x.name}</b><span>Minecraft {x.minecraftVersion||x.version} · {x.loader||"vanilla"} · isolated instance</span></div><button className="primary-btn" onClick={()=>playPack(x)}>Install & Play</button><button className="delete-entry" onClick={()=>removePack(x.id,x)}>Delete</button></div>)}
    {action&&<div className="catalog-note">{action}</div>}
  </section>;
}

function PvpPage({onNavigate}) {
  const modules=[
    ["FPS","Live frame-rate counter."],
    ["Keystrokes","WASD and jump key display."],
    ["Armor Status","Remaining durability for equipped armor."],
    ["Clock","Compact in-game clock."]
  ];
  return <section className="local-instances">
    <div className="eyebrow">NOVA FABRIC CLIENT</div><h2>PvP HUD toolkit</h2>
    <p>These modules are implemented in the NOVA in-game menu. Install the NOVA Fabric 26.2 mod, enter a world, then press Right Shift to open the menu and enable the modules you want.</p>
    <div className="instance-list">{modules.map(([name,description])=><div className="instance-card" key={name}><div><b>{name}</b><span>{description}</span></div><span className="instance-version">READY</span></div>)}</div>
    <button className="primary-btn" onClick={()=>onNavigate("mods")}>Browse more mods</button>
  </section>;
}

function SettingsPage({memoryMax,setMemoryMax}) {
  return <section className="local-instances">
    <div className="eyebrow">NOVA PREFERENCES</div><h2>Performance settings</h2>
    <p>NOVA chooses a conservative first-run limit from your installed RAM. You can adjust it here at any time.</p>
    <div className="instance-create">
      <label htmlFor="nova-memory">GAME MEMORY</label>
      <select id="nova-memory" value={memoryMax||"4G"} onChange={e=>setMemoryMax(e.target.value)}>
        {["1G","2G","3G","4G","6G","8G","12G","16G"].map(v=><option key={v} value={v}>{v.replace("G"," GB")}</option>)}
      </select>
    </div>
    <div className="catalog-note">The selected limit is applied to new launches and modpack preparation. It does not reserve memory while Minecraft is closed.</div>
  </section>;
}

function EmptyPage({ page, setPage }) {
  const content = {
    instances: { icon: Layers3, title: "Your instances", desc: "Keep separate Minecraft versions, mod loaders, and configurations organized.", action: "Create an instance", note: "Instance installation is not connected yet." },
    mods: { icon: Blocks, title: "Mod library", desc: "Find the performance, utility, and PvP mods you want in one place.", action: "Browse mod sources", note: "Modrinth and CurseForge integration is planned." },
    modpacks: { icon: Box, title: "Modpacks", desc: "Explore curated packs or bring your own pack into NOVA.", action: "Explore modpacks", note: "Pack browsing and installation are not connected yet." },
    pvp: { icon: Gamepad2, title: "PvP toolkit", desc: "A focused set of HUD and quality-of-life tools for competitive play.", action: "Explore modules", note: "No gameplay modules are enabled in this preview." },
    settings: { icon: SlidersHorizontal, title: "Preferences", desc: "Tune the launcher experience and prepare your client setup.", action: "Back to overview", note: "Microsoft app ID configured (" + MICROSOFT_CLIENT_ID + "). Microsoft browser sign-in is available from the Overview page." }
  }[page] || { icon: Compass, title: "NOVA", desc: "Your Minecraft, upgraded.", action: "Back to overview", note: "" };
  const Icon = content.icon;
  return <div className="empty-state"><div className="empty-icon"><Icon size={27}/></div><div className="eyebrow">NOVA WORKSPACE</div><h2>{content.title}</h2><p>{content.desc}</p><div className="empty-note"><span className="note-mark"><Activity size={15}/></span><span>{content.note}</span></div><button className="primary-btn" onClick={() => setPage("home")}>{content.action}<ArrowUpRight size={15}/></button></div>;
}

export default function App() {
  const [page, setPage] = useState("home");
  const [selectedVersion, setSelectedVersion] = useState("");
  const [versions, setVersions] = useState([]);
  const [versionsLoading, setVersionsLoading] = useState(true);
  const [loaderType, setLoaderType] = useState("vanilla");
  const [memoryMax,setMemoryMax]=useState(()=>readStored("nova.memoryMax",null,v=>["1G","2G","3G","4G","6G","8G","12G","16G"].includes(v)));
  useEffect(()=>{if(memoryMax)writeStored("nova.memoryMax",memoryMax);},[memoryMax]);
  useEffect(()=>{
    let alive=true;
    const saved=readStored("nova.memoryMax",null,v=>["1G","2G","3G","4G","6G","8G","12G","16G"].includes(v));
    if(!saved)invoke("recommended_memory_max").then(value=>{if(alive&&!readStored("nova.memoryMax",null,v=>["1G","2G","3G","4G","6G","8G","12G","16G"].includes(v)))setMemoryMax(value);}).catch(()=>{if(alive)setMemoryMax("4G");});
    return()=>{alive=false;};
  },[]);
  const [launching, setLaunching] = useState(false);
  const [signedIn, setSignedIn] = useState(false);
  const [accountName, setAccountName] = useState("");
  const [loginBusy, setLoginBusy] = useState(false);
  const [loginUrl, setLoginUrl] = useState("");
  const [message, setMessage] = useState("");
  const [gameExit,setGameExit]=useState("");
  useEffect(() => {
    let alive = true;
    const previousInstall=readStored("nova.activeInstall",null,v=>v&&typeof v==="object");
    if(previousInstall&&!previousInstall.done)writeStored("nova.activeInstall",{...previousInstall,done:true,error:"The launcher closed before this installation finished.",status:"Installation interrupted",detail:"NOVA will remove abandoned temporary files after they become stale. You can safely retry."});
    invoke("list_instances").then(entries=>{if(!alive||!Array.isArray(entries))return;const custom=entries.filter(x=>x&&x.source==="custom").map(x=>({...x,id:x.instanceName||x.name||Date.now()}));const recoveredPacks={};for(const pack of entries.filter(x=>x&&x.source&&x.source!=="custom"&&x.projectId&&x.instanceName))recoveredPacks[String(pack.source)+":"+String(pack.projectId)]=pack;writeStored("nova.instances",custom);writeStored("nova.installedPacks",recoveredPacks);window.dispatchEvent(new Event("nova-instances-recovered"));}).catch(()=>{});
    invoke("minecraft_versions").then(manifest => {
      const entries = Array.isArray(manifest?.versions) ? manifest.versions.filter(v => typeof v.id === "string" && v.id.trim()).map(v => ({id:v.id,type:v.type || "release"})) : [];
      if (!entries.length) throw new Error("Minecraft returned an empty version list.");
      if (alive) { setVersions(entries); setSelectedVersion(manifest.latest?.release && entries.some(v=>v.id===manifest.latest.release) ? manifest.latest.release : entries[0].id); }
    }).catch(error => { if (alive) setMessage(typeof error === "string" ? error : (error.message || "Could not load Minecraft versions.")); })
      .finally(() => { if (alive) setVersionsLoading(false); });
    let offLink;
    let offComplete;
    let offGameExit;
    let offPackProgress;
    (async () => {
      offLink = await listen("nova-login-link", event => { if (alive) { setLoginUrl(event.payload); setMessage("Microsoft sign-in opened in your browser. Complete the sign-in there. If it did not open, use the link below."); } });
      offComplete = await listen("nova-login-complete", event => { if (alive) { setSignedIn(true); setAccountName(event.payload); setLoginUrl(""); setMessage("Microsoft account connected."); } });
      offGameExit=await listen("nova-game-exit",event=>{if(alive){const code=event.payload?.exitCode;setGameExit(code===0?"Minecraft closed normally.":"Minecraft exited with code "+(code??"unknown")+". Check the game log for details.");}});
      offPackProgress=await listen("nova-pack-progress",event=>{if(!alive)return;const progress=event.payload||{};const saved=readStored("nova.activeInstall",null,v=>v&&typeof v==="object");if(saved)writeStored("nova.activeInstall",{...saved,progress:typeof progress.percent==="number"?progress.percent:null,status:progress.phase==="preparing"?"Preparing Minecraft":progress.phase==="archive"?"Downloading modpack archive":"Downloading modpack files",detail:progress.detail||saved.detail});});
      if(!alive){offLink?.();offComplete?.();offGameExit?.();offPackProgress?.();return;}
      try {
        const name = await invoke("minecraft_account_name");
        if (alive && name) { setSignedIn(true); setAccountName(name); }
      } catch (error) { if (alive) setMessage(String(error)); }
    })();
    return () => { alive = false; offLink?.(); offComplete?.(); offGameExit?.(); offPackProgress?.(); };
  }, []);
  const handleSignOut=async()=>{try{await invoke("sign_out_minecraft");setSignedIn(false);setAccountName("");setLoginUrl("");setMessage("Signed out of your Microsoft account.");}catch(error){setMessage(typeof error==="string"?error:(error.message||"Could not sign out."));}};
  const handleLogin = async () => {
    setLoginBusy(true); setLoginUrl(""); setMessage("Requesting a Microsoft sign-in session…");
    try { const name = await invoke("begin_microsoft_login"); setSignedIn(true); setAccountName(name); setLoginUrl(""); setMessage(`Signed in as ${name}.`); }
    catch (error) { setMessage(typeof error === "string" ? error : (error.message || "Microsoft sign-in failed.")); }
    finally { setLoginBusy(false); }
  };
  const handleLaunch = async () => {
    if (!signedIn) { setMessage("Sign in with Microsoft before launching Minecraft."); return; }
    if (typeof selectedVersion !== "string" || !selectedVersion.trim() || selectedVersion === "null" || selectedVersion === "undefined") { setMessage("No Minecraft version is selected. Wait for the version list, then choose a version."); return; }
    if (versionsLoading) { setMessage("Minecraft versions are still loading. Try again in a moment."); return; }
    setLaunching(true); setMessage(`Preparing Minecraft ${selectedVersion} (${loaderType}). The first launch downloads game files and Java runtime.`);
    try { await invoke("launch_minecraft", { version: selectedVersion, loader: loaderType, memoryMax }); setMessage("Minecraft has been started. Keep NOVA open while the game loads."); }
    catch (error) { setMessage(typeof error === "string" ? error : (error.message || "Minecraft could not be started.")); }
    finally { setLaunching(false); }
  };
  const titles = { home: ["Overview", "Everything you need, in one place."], instances: ["Instances", "Your game versions, neatly organized."], mods: ["Mods", "Build the Minecraft setup you want."], modpacks: ["Modpacks", "Discover a new way to play."], pvp: ["PvP features", "A focused toolkit for competitive play."], settings: ["Settings", "Make NOVA feel like yours."] };
  const [title, subtitle] = titles[page] || titles.home;
  return <div className="app-shell"><Sidebar page={page} setPage={setPage} signedIn={signedIn} accountName={accountName} onSignOut={handleSignOut}/><main className="main-area"><Topbar title={title} subtitle={subtitle}/>{gameExit&&<div className="catalog-note" role="status">{gameExit}<button className="text-button" onClick={()=>setGameExit("")}>Dismiss</button></div>}<div className="page-content">{page === "home" ? <Home onNavigate={setPage} selectedVersion={selectedVersion} setSelectedVersion={setSelectedVersion} versions={versions} versionsLoading={versionsLoading} loaderType={loaderType} setLoaderType={setLoaderType} memoryMax={memoryMax} launching={launching} onLaunch={handleLaunch} onLogin={handleLogin} signedIn={signedIn} accountName={accountName} loginBusy={loginBusy} loginUrl={loginUrl} message={message}/> : page === "mods" ? <Catalog packs={false} memoryMax={memoryMax} onNavigate={setPage}/> : page === "modpacks" ? <Catalog packs={true} memoryMax={memoryMax} onNavigate={setPage}/> : page === "instances" ? <LocalInstances versions={versions} memoryMax={memoryMax}/> : page === "settings" ? <SettingsPage memoryMax={memoryMax} setMemoryMax={setMemoryMax}/> : page === "pvp" ? <PvpPage onNavigate={setPage}/> : <EmptyPage page={page} setPage={setPage}/>}</div><footer className="main-footer"><span>© 2026 NOVA CLIENT</span><span>BUILT FOR THE NEXT SESSION <Sparkles size={12}/></span></footer></main></div>;
}

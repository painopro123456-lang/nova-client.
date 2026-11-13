import { MICROSOFT_CLIENT_ID } from "./config";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Activity, ArrowDownToLine, ArrowUpRight, Bell, Blocks, Box, CheckCircle2,
  ChevronDown, CircleHelp, Compass, Cpu, Gamepad2, Gauge, HardDrive,
  Layers3, LayoutDashboard, Monitor, Moon, MoreHorizontal, Play, Plus,
  Search, Settings, ShieldCheck, SlidersHorizontal, Sparkles, Users, Package, ScrollText,
  WandSparkles, Wifi, X
} from "lucide-react";

const navGroups = [
  { label: "PLAY", items: [{ id: "home", label: "Overview", icon: LayoutDashboard }, { id: "instances", label: "Instances", icon: Layers3 }] },
  { label: "DISCOVER", items: [{ id: "mods", label: "Mods", icon: Blocks }, { id: "modpacks", label: "Modpacks", icon: Box }] },
  { label: "CLIENT", items: [{ id: "pvp", label: "PvP features", icon: Gamepad2 }, { id: "settings", label: "Settings", icon: Settings }] }
];


function Sidebar({ page, setPage, signedIn, accountName, onSignOut }) {
  return <aside className="sidebar">
    <div className="brand"><div className="brand-mark"><Moon size={19}/></div><div><b>NOVA</b><span>CLIENT</span></div><span className="build-tag">BETA</span></div>
    <div className="profile-card"><div className="avatar">N</div><div className="profile-copy"><b>{signedIn ? (accountName || "Microsoft account") : "Guest profile"}</b><span>{signedIn ? "Minecraft account connected" : "Sign in to continue"}</span></div><ChevronDown size={15}/></div>
    <nav>{navGroups.map(group => <section className="nav-group" key={group.label}><div className="nav-label">{group.label}</div>{group.items.map(item => <button key={item.id} className={page === item.id ? "nav-item active" : "nav-item"} onClick={() => setPage(item.id)}><item.icon size={17}/><span>{item.label}</span>{item.id === "pvp" && <span className="nav-dot"/>}</button>)}</section>)}</nav>
    <div className="sidebar-bottom"><div className="status-line"><span className="status-pulse"/><span>NOVA launcher</span></div>{signedIn && <button className="help-link" onClick={onSignOut}>Sign out of Microsoft</button>}<button className="help-link" onClick={() => setPage("settings")}><CircleHelp size={16}/> Help & support</button><div className="sidebar-foot">NOVA CLIENT <span>0.1.0</span></div></div>
  </aside>;
}

function Topbar({ title, subtitle, onShowDownloads }) {
  const win = getCurrentWindow();
  return <header className="topbar">
    <div className="topbar-title" data-tauri-drag-region><div className="eyebrow">NOVA / {title.toUpperCase()}</div><h1>{title}</h1><p>{subtitle}</p></div>
    <div className="top-actions">
      <button className="icon-btn" title="Show download progress" onClick={onShowDownloads}><Bell size={17}/><i/></button>
      <div className="top-avatar">G</div>
      <div className="window-controls">
        <button title="Minimize" onClick={() => win.minimize()}><span>−</span></button>
        <button title="Maximize" onClick={() => win.toggleMaximize()}><span>□</span></button>
        <button className="close-window" title="Close" onClick={() => win.close()}><X size={14}/></button>
      </div>
    </div>
  </header>;
}

function Home({ onNavigate, selectedVersion, setSelectedVersion, versions, versionsLoading, loaderType, setLoaderType, launching, onLaunch, onLogin, signedIn, accountName, loginBusy, loginUrl, message }) {
  return <>
    <div className="hero-panel">
      <div className="hero-glow"/>
      <div className="hero-content"><div className="hero-kicker"><span className="kicker-icon"><Sparkles size={13}/></span> YOUR NEXT SESSION STARTS HERE</div><h2>Make every<br/><em>frame count.</em></h2><p>A cleaner way to launch Minecraft. Your worlds, your settings, your pace.</p><div className="hero-actions"><button className="primary-btn" onClick={onLaunch} disabled={launching || !signedIn || versionsLoading || !selectedVersion}><Play size={16} fill="currentColor"/>{launching ? "Preparing Minecraft…" : "Play Minecraft"}<ArrowUpRight size={15}/></button><button className="secondary-btn" onClick={onLogin} disabled={loginBusy || signedIn}>{signedIn ? `Signed in${accountName ? ` · ${accountName}` : ""}` : loginBusy ? "Waiting for Microsoft…" : "Sign in with Microsoft"}</button><button className="secondary-btn" onClick={() => onNavigate("instances")}>Manage instances</button></div><div className="hero-foot"><span><ShieldCheck size={14}/> Microsoft account stored securely on this PC</span><span><Gauge size={14}/> {loaderType === "fabric" ? "Fabric modded" : "Vanilla Java Edition"}</span></div>{loginUrl && !signedIn && <a className="text-button" href={loginUrl} target="_blank" rel="noreferrer">Continue Microsoft sign-in ↗</a>}{message && <p className="launch-message" role="status">{message}</p>}</div>
      <div className="orbit-art" aria-hidden="true"><div className="orbit-ring ring-one"/><div className="orbit-ring ring-two"/><div className="orbit-core"><Sparkles size={48}/></div><div className="orbit-chip chip-one"><Cpu size={15}/> PERFORMANCE</div><div className="orbit-chip chip-two"><Gamepad2 size={15}/> BUILT FOR PLAY</div></div>
      <div className="hero-index">01 <span>/ 04</span></div>
    </div>
    <div className="section-heading"><div><div className="eyebrow">READY WHEN YOU ARE</div><h2>Quick launch</h2></div><button className="text-button" onClick={() => onNavigate("instances")}>All instances <ArrowUpRight size={14}/></button></div>
    <div className="quick-card"><div className="minecraft-icon"><Box size={22}/></div><div className="quick-info"><b>{loaderType === "fabric" ? "Fabric Minecraft" : "Vanilla Minecraft"}</b><span>{selectedVersion || "Select a game version"} · {loaderType === "fabric" ? "Fabric Loader" : "Clean profile"}</span></div><div className="version-select"><label>GAME VERSION</label><select value={selectedVersion} onChange={e => setSelectedVersion(e.target.value)} disabled={versionsLoading || versions.length === 0}><option value="" disabled>{versionsLoading ? "Loading versions…" : "Choose a version"}</option>{versions.map(v => <option key={v.id} value={v.id}>{v.id}{v.type === "snapshot" ? " · Snapshot" : ""}</option>)}</select></div><div className="version-select"><label>MOD LOADER</label><select value={loaderType} onChange={e => setLoaderType(e.target.value)}><option value="vanilla">Vanilla</option><option value="fabric">Fabric</option></select></div><button className="small-play" onClick={onLaunch} disabled={launching || !signedIn || versionsLoading || !selectedVersion}><Play size={15} fill="currentColor"/> Launch</button></div>
    <div className="feature-grid"><div className="feature-card"><div className="feature-icon purple"><Gauge size={18}/></div><b>Performance first</b><p>A native Rust shell is the foundation. No performance claims until benchmarks exist.</p><span className="feature-status">IN DEVELOPMENT</span></div><div className="feature-card"><div className="feature-icon blue"><Blocks size={18}/></div><b>Your mod library</b><p>Browse Modrinth and CurseForge, then install compatible mods into an existing modded instance.</p><span className="feature-status">AVAILABLE</span></div><div className="feature-card"><div className="feature-icon green"><ShieldCheck size={18}/></div><b>Profiles that stay yours</b><p>Create isolated Minecraft instances and keep their modpacks and settings organized.</p><span className="feature-status">AVAILABLE</span></div></div>
  </>;
}


function Catalog({ packs, active, memoryMb, signedIn, installDialog, setInstallDialog }) {
  const [q,setQ]=useState("");
  const [provider,setProvider]=useState("modrinth");
  const [hits,setHits]=useState([]);
  const [hasNext,setHasNext]=useState(false);
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState("");
  const [page,setPage]=useState(0);
  const [installed,setInstalled]=useState(()=>{try{const value=JSON.parse(localStorage.getItem("nova.installedPacks")||"{}");return value&&typeof value==="object"&&!Array.isArray(value)?value:{}}catch{return{}}});
  const [installingId,setInstallingId]=useState("");
  const [packActionMessage,setPackActionMessage]=useState("");
  const [versionProject,setVersionProject]=useState(null);
  const [packVersions,setPackVersions]=useState([]);
  const [packVersionsBusy,setPackVersionsBusy]=useState(false);
  const [selectedPackVersion,setSelectedPackVersion]=useState("");
  const [targetInstances,setTargetInstances]=useState([]);
  const [selectedTargetInstance,setSelectedTargetInstance]=useState(()=>{try{return localStorage.getItem("nova.modsTargetInstance")||""}catch{return ""}});

  useEffect(()=>{
    if(!active) return;
    // Clear stale cards immediately when the search, source, or page changes.
    // Otherwise old results remain clickable during the debounce window while
    // the selected provider has already changed.
    setBusy(true); setError(""); setHits([]); setHasNext(false);
    const ctrl=new AbortController();
    const timer=setTimeout(async()=>{
      try {
        let results=[];
        let moreAvailable=false;
        if(provider==="modrinth") {
          const p=new URLSearchParams({query:q,facets:JSON.stringify([[packs?"project_type:modpack":"project_type:mod"]]),limit:"24",offset:String(page*24)});
          const r=await fetch("https://api.modrinth.com/v2/search?"+p,{signal:ctrl.signal});
          if(!r.ok) throw new Error("Modrinth catalog request failed");
          const data=await r.json();
          results=(data.hits||[]).map(x=>({id:x.project_id,title:x.title,description:x.description,downloads:x.downloads,image:x.icon_url,url:"https://modrinth.com/"+(packs?"modpack/":"mod/")+x.slug}));
          moreAvailable=Number(data.total_hits||0)>(page*24+results.length);
        } else {
          const data=await invoke("curseforge_search",{query:q,packs,index:page*24,gameVersion:null});
          results=(data.data||[]).map(x=>({id:String(x.id),title:x.name,description:x.summary||"No description provided.",downloads:x.downloadCount,image:x.logo?.thumbnailUrl,url:x.links?.websiteUrl||"https://www.curseforge.com/minecraft/"+(packs?"modpacks":"mc-mods")+"/"+x.slug}));
          moreAvailable=Number(data.pagination?.totalCount||0)>(page*24+results.length);
        }
        if(!ctrl.signal.aborted) { setHits(results); setHasNext(moreAvailable); }
      } catch(e) { if(!ctrl.signal.aborted) setError(typeof e==="string"?e:(e.message||"Unable to load catalog. Check your connection and retry.")); }
      finally { if(!ctrl.signal.aborted) setBusy(false); }
    },250);
    return ()=>{clearTimeout(timer);ctrl.abort();};
  },[q,packs,provider,page,active]);

  useEffect(()=>{try{localStorage.setItem("nova.installedPacks",JSON.stringify(installed))}catch(e){setPackActionMessage("Could not save installed modpack list: "+(e.message||e))}},[installed]);
  const choosePackVersion=async(project)=>{
    setVersionProject(project);setPackVersions([]);setSelectedPackVersion("");setPackVersionsBusy(true);setPackActionMessage("");
    if(!packs){
      try{
        const instances=await invoke("list_instances");
        const compatible=(Array.isArray(instances)?instances:[]).filter(x=>x.loader&&x.loader!=="vanilla");
        if(!compatible.length)throw new Error("Create a Fabric, Forge, NeoForge, or Quilt instance first. Mods cannot be installed into Vanilla profiles.");
        setTargetInstances(compatible);
        const preferred=(()=>{try{return localStorage.getItem("nova.modsTargetInstance")||""}catch{return ""}})();
        setSelectedTargetInstance(compatible.some(x=>x.instanceName===preferred)?preferred:compatible[0].instanceName);
      }catch(e){setPackActionMessage(typeof e==="string"?e:(e.message||"Could not load your installed instances."));setVersionProject(null);setPackVersionsBusy(false);return;}
    }
    try{
      let versions=[];
      if(provider==="modrinth"){
        const response=await fetch("https://api.modrinth.com/v2/project/"+encodeURIComponent(project.id)+"/version");
        if(!response.ok)throw new Error("Could not load versions from Modrinth.");
        const data=await response.json();
        versions=(data||[]).map(v=>({id:v.id,name:v.name||v.version_number||"Version",versionNumber:v.version_number||"",gameVersions:v.game_versions||[],loaders:v.loaders||[],releaseType:v.version_type||"release",published:v.date_published||""}));
      }else{
        const data=await invoke("curseforge_modpack_versions",{modId:String(project.id)});
        versions=(data||[]).map(v=>({id:String(v.id),name:v.displayName||v.fileName||("File "+v.id),versionNumber:v.fileName||"",gameVersions:v.gameVersions||[],loaders:[],releaseType:"release",published:v.fileDate||""}));
      }
      if(!versions.length)throw new Error("This project has no downloadable versions.");
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
    setInstallDialog({name:project.title,step:0,progress:null,status:"Starting installation",detail:"Preparing your isolated modpack instance…",done:false,error:""});
    try{
      if(!packs){
        const target=targetInstances.find(x=>x.instanceName===selectedTargetInstance);
        if(!target)throw new Error("Choose an existing modded instance.");
        setInstallDialog({name:project.title,step:0,progress:null,status:"Installing mod",detail:"Checking compatibility and downloading the selected JAR…",done:false,error:""});
        const result=await invoke("install_mod_to_instance",{provider,projectId,versionId:selectedPackVersion,instanceName:selectedTargetInstance});
        setInstallDialog({name:project.title,step:2,progress:100,status:"Mod installed",detail:result.fileName+" was added to "+(target.name||target.instanceName)+". It will load the next time Minecraft starts.",done:true,error:""});
        setPackActionMessage(project.title+" installed into "+(target.name||target.instanceName)+".");
        return;
      }
      setInstallDialog({name:project.title,step:0,progress:null,status:"Downloading modpack",detail:"Getting the selected release and its manifest…",done:false,error:""});
      const metadata=provider==="modrinth"
        ? await invoke("install_modrinth_modpack",{projectId,instanceName,versionId:selectedPackVersion})
        : await invoke("install_curseforge_modpack",{modId:projectId,instanceName,fileId:Number(selectedPackVersion)});
      // Public modpack files can be downloaded without a Minecraft login.
      if(!signedIn){
        const previous=installed[storageKey];
        if(previous?.instanceName&&previous.instanceName!==metadata.instanceName){
          try{await invoke("delete_instance",{instanceName:previous.instanceName});}catch{/* Keep the new pack. */}
        }
        if(project.image){try{const icons=JSON.parse(localStorage.getItem("nova.packIcons")||"{}");icons[metadata.instanceName]=project.image;localStorage.setItem("nova.packIcons",JSON.stringify(icons));}catch{}}
      setInstalled(old=>({...old,[storageKey]:{...metadata,iconUrl:project.image||""}}));
        setInstallDialog({name:metadata.name||project.title,step:1,progress:100,status:"Modpack downloaded",detail:"All available modpack files are saved. Sign in with Microsoft only when you're ready to install Minecraft and play.",done:true,error:""});
        setPackActionMessage((metadata.name||project.title)+" downloaded. Microsoft sign-in is only needed to prepare and launch Minecraft.");
        return;
      }
      setInstallDialog({name:metadata.name||project.title,step:1,progress:null,status:"Preparing Minecraft",detail:"Installing Minecraft "+metadata.minecraftVersion+" and the Java runtime…",done:false,error:""});
      await invoke("prepare_modpack_instance",{instanceName:metadata.instanceName});
      const previous=installed[storageKey];
      if(previous?.instanceName&&previous.instanceName!==metadata.instanceName){
        try{await invoke("delete_instance",{instanceName:previous.instanceName});}catch{/* Keep the new working install. */}
      }
      if(project.image){try{const icons=JSON.parse(localStorage.getItem("nova.packIcons")||"{}");icons[metadata.instanceName]=project.image;localStorage.setItem("nova.packIcons",JSON.stringify(icons));}catch{}}
      setInstalled(old=>({...old,[storageKey]:{...metadata,iconUrl:project.image||""}}));
      setInstallDialog({name:metadata.name||project.title,step:2,progress:100,status:"Installation complete",detail:"Minecraft "+metadata.minecraftVersion+" · "+metadata.loader+(metadata.loaderVersion?" "+metadata.loaderVersion:"")+" is ready to play.",done:true,error:""});
      setPackActionMessage((metadata.name||project.title)+" is fully installed and ready to play.");
    }catch(e){
      const reason=typeof e==="string"?e:(e.message||(packs?"Modpack installation failed.":"Mod installation failed."));
      if(packs){try{await invoke("delete_instance",{instanceName});}catch{/* Ignore cleanup errors; show the original installation failure. */}}
      setInstallDialog(current=>({...current,done:true,error:reason,status:packs?"Installation failed":"Mod installation failed",detail:packs?"The incomplete instance was cleaned up where possible. You can safely retry.":"No existing instance was deleted. Check compatibility and try again."}));
      setPackActionMessage((project.title)+" installation failed: "+reason);
    }finally{setInstallingId("");}
  };
  const launchPack=async metadata=>{
    if(!signedIn){setPackActionMessage("Your modpack files are downloaded. Sign in with Microsoft when you are ready to install Minecraft and play.");return;}
    setInstallingId(String(metadata.projectId));setPackActionMessage(`Preparing ${metadata.name}…`);
    try{
      await invoke("launch_minecraft",{version:metadata.minecraftVersion,loader:metadata.loader,loaderBuild:metadata.loaderVersion,instanceName:metadata.instanceName,memoryMb});
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
      <div className="catalog-search"><Search size={17}/><input value={q} onChange={e=>{setQ(e.target.value);setPage(0);}} placeholder={packs?"Search modpacks, themes, adventures…":"Search mods, performance, shaders, utilities…"}/>{q&&<button className="clear-search" onClick={()=>{setQ("");setPage(0);}} aria-label="Clear search"><X size={14}/></button>}</div>
      <div className="catalog-source-pill"><span className="source-dot"/> {provider==="modrinth"?"MODRINTH INDEX":"CURSEFORGE API"}</div>
    </div>
    {busy&&<div className="catalog-state"><div className="loading-orbit"/><b>Finding great projects</b><span>Searching {provider==="modrinth"?"Modrinth":"CurseForge"}…</span></div>}
    {error&&<div className="catalog-note error"><CircleHelp size={16}/><span>{error}</span></div>}
    {packActionMessage&&<div className="catalog-note"><span>{packActionMessage}</span></div>}
    {!busy&&!error&&hits.length===0&&<div className="catalog-state"><div className="empty-library"><Blocks size={25}/></div><b>No projects found</b><span>Try another search or switch catalog source.</span></div>}
    {!busy&&hits.length>0&&<div className="catalog-results-label"><span>{hits.length} PROJECTS ON THIS PAGE</span><span>PAGE {String(page+1).padStart(2,"0")}</span></div>}
    <div className="catalog-grid">{hits.map((x,i)=><article className="catalog-card" key={x.id}>
      <div className="project-card-top">{x.image?<img src={x.image} alt="" loading="lazy"/>:<div className="catalog-placeholder"><Blocks size={20}/></div>}<span className="project-source">{provider==="modrinth"?"MODRINTH":"CURSEFORGE"}</span><span className="project-rank">{String(page*24+i+1).padStart(2,"0")}</span></div>
      <h3>{x.title}</h3><p>{x.description}</p>
      <div className="catalog-meta"><span><ArrowDownToLine size={12}/>{Number(x.downloads||0).toLocaleString()}</span><a href={x.url} target="_blank" rel="noreferrer">View project <ArrowUpRight size={12}/></a></div>
      <div className="pack-install-row">{packs&&installed[provider+":"+String(x.id)]
        ? <button className="primary-btn" onClick={()=>launchPack(installed[provider+":"+String(x.id)])} disabled={Boolean(installingId)}><Play size={14} fill="currentColor"/> {installingId===String(x.id)?"Launching…":signedIn?"Install & Play":"Downloaded · Sign in to play"}</button>
        : <button className="primary-btn" onClick={()=>choosePackVersion(x)} disabled={Boolean(installingId)}><ArrowDownToLine size={14}/> {installingId===String(x.id)?"Installing…" :packs?"Install Modpack":"Install to instance"}</button>}</div>
    </article>)}</div>}
    {versionProject&&<div className="pack-version-backdrop" role="presentation" onClick={()=>!packVersionsBusy&&setVersionProject(null)}>
      <section className={"pack-version-dialog"+(packs?" modpack-version-dialog":"")} role="dialog" aria-modal="true" aria-labelledby="pack-version-title" onClick={e=>e.stopPropagation()}>
        <div className="pack-version-dialog-top"><div><div className="eyebrow">{packs?"NOVA MODPACK INSTALLER":"NOVA MOD INSTALLER"}</div><h3 id="pack-version-title">{packs?"Choose your modpack version":"Choose a mod version"}</h3><p>{versionProject.title} · {provider==="modrinth"?"Modrinth":"CurseForge"}</p></div><button className="pack-version-close" onClick={()=>setVersionProject(null)} aria-label="Close version picker"><X size={17}/></button></div>
        {packVersionsBusy?<div className="catalog-state compact"><div className="loading-orbit"/><span>Finding available releases…</span></div>
          : <>{!packs&&<div className="pack-version-target"><label className="pack-version-label" htmlFor="target-instance-select">INSTALL INTO INSTANCE</label><select id="target-instance-select" className="pack-version-select" value={selectedTargetInstance} onChange={e=>setSelectedTargetInstance(e.target.value)}>{targetInstances.map(v=><option key={v.instanceName} value={v.instanceName}>{v.name||v.instanceName} · {v.minecraftVersion} · {v.loader}</option>)}</select></div>}
          <div className="pack-version-list-heading"><span>{packs?"AVAILABLE MODPACK RELEASES":"AVAILABLE MOD RELEASES"}</span><span>{packVersions.length} VERSION{packVersions.length===1?"":"S"}</span></div>
          <div className="pack-version-list" role="listbox" aria-label={packs?"Available modpack versions":"Available mod versions"}>
            {packVersions.map((v,index)=><button type="button" role="option" aria-selected={selectedPackVersion===v.id} className={"pack-version-option"+(selectedPackVersion===v.id?" selected":"")} key={v.id} onClick={()=>setSelectedPackVersion(v.id)}>
              <span className="pack-version-radio">{selectedPackVersion===v.id&&<CheckCircle2 size={17}/>}</span>
              <span className="pack-version-option-main"><b>{v.name}</b><span className="pack-version-option-sub">{v.versionNumber&&v.versionNumber!==v.name?v.versionNumber:"Release "+(packVersions.length-index)}</span><span className="pack-version-chips">{(v.gameVersions||[]).slice(0,4).map(game=><i key={game}>MC {game}</i>)}{(v.loaders||[]).slice(0,3).map(loader=><i key={loader}>{loader}</i>)}{!(v.gameVersions||[]).length&&<i>Game version not listed</i>}</span></span>
              <span className="pack-version-option-date">{v.published?new Date(v.published).toLocaleDateString():"Release"}</span>
            </button>)}
          </div>
          <div className="pack-version-actions"><button className="secondary-btn" onClick={()=>setVersionProject(null)}>Cancel</button><button className="primary-btn" onClick={installPack} disabled={!selectedPackVersion||(!packs&&!selectedTargetInstance)}><ArrowDownToLine size={14}/>{packs?"Install Modpack":"Install Mod"}</button></div></>}
      </section>
    </div>}
    <div className="catalog-pagination"><button className="secondary-btn" disabled={page===0||busy} onClick={()=>setPage(p=>Math.max(0,p-1))}>← Previous</button><span>{String(page+1).padStart(2,"0")} <i/> {provider==="modrinth"?"DISCOVER":"CURSEFORGE"}</span><button className="secondary-btn" disabled={busy||!hasNext} onClick={()=>setPage(p=>p+1)}>Next →</button></div>
  </section>;
}
function LocalInstances({ versions, memoryMb, setPage }) {
  const [items,setItems]=useState([]);
  const [packs,setPacks]=useState({});
  const [packIcons,setPackIcons]=useState(()=>{try{return JSON.parse(localStorage.getItem("nova.packIcons")||"{}")}catch{return{}}});
  const [name,setName]=useState("");
  const [version,setVersion]=useState("");
  const [versionSearch,setVersionSearch]=useState("");
  const [versionFilter,setVersionFilter]=useState("release");
  const [loader,setLoader]=useState("fabric");
  const [action,setAction]=useState("");
  const [instancePanel,setInstancePanel]=useState(null);\n  const [selectedInstance,setSelectedInstance]=useState(null);
  const [showCustom,setShowCustom]=useState(false);
  const [creating,setCreating]=useState(false);
  useEffect(()=>{
    let alive=true;
    invoke("list_instances").then(found=>{
      if(!alive || !Array.isArray(found)) return;
      const icons=(()=>{try{return JSON.parse(localStorage.getItem("nova.packIcons")||"{}")}catch{return{}}})();
      const custom=found.filter(x=>x.source==="custom").map(x=>({...x,id:x.instanceName}));
      const packEntries=found.filter(x=>(x.source==="modrinth"||x.source==="curseforge")&&x.projectId)
        .map(x=>({...x,iconUrl:x.iconUrl||icons[x.instanceName]||""}));
      const packMap=Object.fromEntries(packEntries.map(x=>[x.source+":"+String(x.projectId),x]));
      setItems(custom);setPacks(packMap);setPackIcons(icons);
    }).catch(error=>{if(alive)setAction(typeof error==="string"?error:(error.message||"Could not recover installed instances from disk."));});
    return()=>{alive=false;};
  },[]);
  useEffect(()=>{if(!version&&versions.length)setVersion(versions.find(v=>v.type==="release")?.id||versions[0].id);},[versions,version]);
  const visibleVersions=versions.filter(v=>(versionFilter==="all"||v.type===versionFilter)&&v.id.toLowerCase().includes(versionSearch.trim().toLowerCase())).slice(0,300);
  const selectedVersionVisible=visibleVersions.some(v=>v.id===version);
  useEffect(()=>{try{localStorage.setItem("nova.instances",JSON.stringify(items))}catch(e){setAction("Could not save instance list: "+(e.message||e))}},[items]);
  const add=async()=>{
    if(!name.trim()||!version||!selectedVersionVisible)return;
    const instanceName=name.trim().toLowerCase().replace(/[^a-z0-9]+/g,"-").replace(/^-|-$/g,"").slice(0,48);
    setCreating(true);setAction("Creating "+name.trim()+"…");
    try{
      const metadata=await invoke("create_instance",{instanceName,displayName:name.trim(),version,loader});
      setItems(a=>[...a,{id:metadata.instanceName,...metadata}]);setName("");setShowCustom(false);
      setAction((metadata.name||"Custom instance")+" created. Install & Play when you're ready.");
    }catch(e){setAction(typeof e==="string"?e:(e.message||"Could not create this instance."));}
    finally{setCreating(false);}
  };
  const playPack=async pack=>{
    setAction("Preparing "+pack.name+"…");
    try{await invoke("launch_minecraft",{version:pack.minecraftVersion||pack.version,loader:pack.loader||"vanilla",loaderBuild:pack.loaderVersion||"latest",instanceName:pack.instanceName,memoryMb});setAction(pack.name+" started.");}
    catch(e){setAction(typeof e==="string"?e:(e.message||"Could not launch this instance."));}
  };
  const playCustom=async item=>{
    setAction("Preparing "+item.name+"…");
    try{await invoke("launch_minecraft",{version:item.minecraftVersion||item.version,loader:item.loader||"vanilla",loaderBuild:item.loaderVersion||"latest",instanceName:item.instanceName,memoryMb});setAction(item.name+" started.");}
    catch(e){setAction(typeof e==="string"?e:(e.message||"Could not launch this instance."));}
  };
  const openInstancePanel=async(instance,tab)=>{
    setInstancePanel({instance,tab,loading:true,content:[],logs:"",error:""});
    try{
      if(tab==="content"){
        const content=await invoke("list_instance_content",{instanceName:instance.instanceName});
        setInstancePanel(current=>current?{...current,loading:false,content:Array.isArray(content)?content:[]}:current);
      }else{
        const result=await invoke("get_game_logs",{instanceName:instance.instanceName});
        setInstancePanel(current=>current?{...current,loading:false,logs:result?.minecraftLog||"No Minecraft logs found."}:current);
      }
    }catch(error){
      setInstancePanel(current=>current?{...current,loading:false,error:typeof error==="string"?error:(error.message||"Could not load instance data.")}:current);
    }
  };
  const removePack=async(id,pack)=>{
    setAction("Deleting "+pack.name+"…");
    try{
      await invoke("delete_instance",{instanceName:pack.instanceName});
      setPacks(old=>{const next={...old};delete next[id];return next;});
      setPackIcons(old=>{const next={...old};delete next[pack.instanceName];try{localStorage.setItem("nova.packIcons",JSON.stringify(next));}catch{}return next;});
      setAction(pack.name+" deleted.");
    }catch(e){setAction(typeof e==="string"?e:(e.message||"Could not delete this instance."));}
  };
  const removeCustom=async item=>{
    setAction("Deleting "+item.name+"…");
    try{await invoke("delete_instance",{instanceName:item.instanceName});setItems(old=>old.filter(x=>x.instanceName!==item.instanceName));setAction(item.name+" deleted.");}
    catch(e){setAction(typeof e==="string"?e:(e.message||"Could not delete this instance."));}
  };
  const installedPacks=Object.entries(packs);
  return <section className="local-instances">
    <div className="instances-topline"><div><div className="eyebrow">YOUR GAME LIBRARY</div><h2>Your modpacks</h2><p>Open a pack to launch it, manage its mods, or view its logs.</p></div><button className="primary-btn custom-instance-trigger" onClick={()=>setShowCustom(true)}><Plus size={15}/> Custom instance</button></div>
    {installedPacks.length>0&&<><div className="section-heading"><div><div className="eyebrow">MODPACK LIBRARY</div><h2>Installed modpacks</h2></div><span className="instance-count">{installedPacks.length} PACK{installedPacks.length===1?"":"S"}</span></div><div className="instance-grid">{installedPacks.map(([id,pack])=><article className="instance-card" key={id} role="button" tabIndex={0} onClick={()=>setSelectedInstance(pack)} onKeyDown={e=>{if(e.key==="Enter"||e.key===" "){e.preventDefault();setSelectedInstance(pack);}}} style={{cursor:"pointer"}}>
      <div className="instance-card-art">{pack.iconUrl?<img src={pack.iconUrl} alt="" loading="lazy" onError={e=>{e.currentTarget.style.display="none";}}/>:<Box size={30}/>}</div>
      <div className="instance-card-copy"><div className="instance-card-kicker">MODPACK · {pack.source}</div><b>{pack.name}</b><span>Minecraft {pack.minecraftVersion} · {pack.loader}{pack.loaderVersion?" "+pack.loaderVersion:""}</span></div>
      <div className="instance-card-actions"><button className="primary-btn" onClick={e=>{e.stopPropagation();playPack(pack);}}><Play size={13} fill="currentColor"/> Play</button><button className="secondary-btn" onClick={e=>{e.stopPropagation();openInstancePanel(pack,"content");}}><Package size={13}/> Content</button><button className="secondary-btn" onClick={e=>{e.stopPropagation();openInstancePanel(pack,"logs");}}><ScrollText size={13}/> Logs</button><button className="delete-entry" onClick={e=>{e.stopPropagation();removePack(id,pack);}}>Delete</button></div>
    </article>)}</div></>}
    {action&&<div className="catalog-note">{action}</div>}
    {selectedInstance&&<div className="custom-instance-backdrop" role="presentation" onClick={()=>setSelectedInstance(null)}><section className="custom-instance-dialog" role="dialog" aria-modal="true" aria-labelledby="selected-instance-title" onClick={e=>e.stopPropagation()} style={{maxWidth:820}}>
      <div className="custom-dialog-top"><div><div className="eyebrow">INSTALLED MODPACK · {selectedInstance.source}</div><h2 id="selected-instance-title">{selectedInstance.name}</h2><p>Minecraft {selectedInstance.minecraftVersion} · {selectedInstance.loader}{selectedInstance.loaderVersion?" "+selectedInstance.loaderVersion:""}</p></div><button className="icon-btn" onClick={()=>setSelectedInstance(null)} aria-label="Close"><X size={17}/></button></div>
      <div className="instance-detail-art" style={{display:"flex",alignItems:"center",gap:18,padding:"18px 0"}}>{selectedInstance.iconUrl?<img src={selectedInstance.iconUrl} alt="" style={{width:92,height:92,objectFit:"cover",borderRadius:16}}/>:<div className="instance-card-art"><Box size={32}/></div>}<div><div className="eyebrow">YOUR LIBRARY</div><b>{selectedInstance.name}</b><p style={{margin:"6px 0 0",opacity:.72}}>Your own isolated modpack profile.</p></div></div>
      <div className="instance-card-actions" style={{justifyContent:"flex-start",flexWrap:"wrap",gap:10}}>
        <button className="primary-btn" onClick={()=>playPack(selectedInstance)}><Play size={14} fill="currentColor"/> Launch</button>
        <button className="secondary-btn" onClick={()=>{try{localStorage.setItem("nova.modsTargetInstance",selectedInstance.instanceName)}catch{}setSelectedInstance(null);setPage("mods");}}><Blocks size={14}/> Add mods</button>
        <button className="secondary-btn" onClick={()=>{setSelectedInstance(null);openInstancePanel(selectedInstance,"content");}}><Package size={14}/> Content</button>
        <button className="secondary-btn" onClick={()=>{setSelectedInstance(null);openInstancePanel(selectedInstance,"logs");}}><ScrollText size={14}/> Logs</button>
      </div>
      <div className="catalog-note" style={{marginTop:18}}>Choose <b>Add mods</b> to browse Modrinth or CurseForge and install compatible mods into this exact instance.</div>
      <div className="pack-version-actions"><button className="secondary-btn" onClick={()=>setSelectedInstance(null)}>Back to instances</button></div>
    </section></div>}
    {instancePanel&&<div className="custom-instance-backdrop" role="presentation" onClick={()=>setInstancePanel(null)}><section className="custom-instance-dialog" role="dialog" aria-modal="true" aria-labelledby="instance-panel-title" onClick={e=>e.stopPropagation()} style={{maxWidth:760}}>
      <div className="custom-dialog-top"><div><div className="eyebrow">{instancePanel.tab==="content"?"INSTANCE CONTENT":"MINECRAFT LOGS"}</div><h2 id="instance-panel-title">{instancePanel.instance.name||instancePanel.instance.instanceName}</h2><p>{instancePanel.tab==="content"?"Installed mod files in this instance.":"Latest Minecraft log for this instance."}</p></div><button className="icon-btn" onClick={()=>setInstancePanel(null)} aria-label="Close"><X size={17}/></button></div>
      {instancePanel.loading?<div className="catalog-note">Loading {instancePanel.tab==="content"?"installed mods":"Minecraft logs"}…</div>:instancePanel.error?<div className="catalog-note error">{instancePanel.error}</div>:instancePanel.tab==="content"?<div style={{maxHeight:440,overflowY:"auto",display:"grid",gap:8}}>
        <div className="feature-status">{instancePanel.content.length} MOD FILE{instancePanel.content.length===1?"":"S"}</div>
        {instancePanel.content.length===0?<div className="catalog-note">No mod JAR files found in this instance yet.</div>:instancePanel.content.map((file,index)=><div key={file.name+index} style={{display:"flex",alignItems:"center",justifyContent:"space-between",gap:12,padding:"10px 12px",border:"1px solid var(--border-color,rgba(255,255,255,.08))",borderRadius:10}}><span style={{display:"flex",alignItems:"center",gap:10,minWidth:0}}><Package size={16}/><b style={{fontSize:13,overflowWrap:"anywhere"}}>{file.name}</b></span><span className="feature-status" style={{whiteSpace:"nowrap"}}>{file.size<1048576?(file.size/1024).toFixed(0)+" KB":(file.size/1048576).toFixed(1)+" MB"}</span></div>)}
      </div>:<pre className="catalog-note" style={{whiteSpace:"pre-wrap",overflowWrap:"anywhere",maxHeight:460,overflowY:"auto",fontSize:12,lineHeight:1.6}}>{instancePanel.logs}</pre>}
      <div className="pack-version-actions"><button className="secondary-btn" onClick={()=>openInstancePanel(instancePanel.instance,instancePanel.tab)} disabled={instancePanel.loading}>Refresh</button><button className="primary-btn" onClick={()=>setInstancePanel(null)}>Done</button></div>
    </section></div>}
    {showCustom&&<div className="custom-instance-backdrop" role="presentation" onClick={()=>setShowCustom(false)}><section className="custom-instance-dialog" role="dialog" aria-modal="true" aria-labelledby="custom-instance-title" onClick={e=>e.stopPropagation()}>
      <div className="custom-dialog-top"><div><div className="eyebrow">NOVA INSTANCE BUILDER</div><h2 id="custom-instance-title">Create custom instance</h2><p>Choose a Minecraft version, pick a loader, and give this profile a name.</p></div><button className="icon-btn" onClick={()=>setShowCustom(false)} aria-label="Close"><X size={17}/></button></div>
      <div className="custom-form-grid"><label className="custom-field custom-name-field"><span>INSTANCE NAME</span><input autoFocus value={name} onChange={e=>setName(e.target.value)} onKeyDown={e=>e.key==="Enter"&&add()} placeholder="e.g. Survival, PvP, Modded"/></label>
      <label className="custom-field"><span>MOD LOADER</span><select value={loader} onChange={e=>setLoader(e.target.value)}><option value="fabric">Fabric</option><option value="forge">Forge</option><option value="neoforge">NeoForge</option><option value="quilt">Quilt</option><option value="vanilla">Vanilla</option></select></label></div>
      <div className="custom-version-head"><div><div className="eyebrow">GAME VERSION</div><b>Choose your Minecraft version</b></div><span>{visibleVersions.length} available</span></div>
      <div className="custom-version-tools"><div className="custom-version-search"><Search size={15}/><input value={versionSearch} onChange={e=>setVersionSearch(e.target.value)} placeholder="Search versions…"/></div><div className="custom-version-filters"><button className={versionFilter==="release"?"selected":""} onClick={()=>setVersionFilter("release")}>Releases</button><button className={versionFilter==="snapshot"?"selected":""} onClick={()=>setVersionFilter("snapshot")}>Snapshots</button><button className={versionFilter==="all"?"selected":""} onClick={()=>setVersionFilter("all")}>All</button></div></div>
      <div className="custom-version-list">{visibleVersions.map(v=><button key={v.id} className={"custom-version-option"+(version===v.id?" selected":"")} onClick={()=>setVersion(v.id)}><span className="version-option-icon"><Box size={15}/></span><span className="version-option-copy"><b>{v.id}</b><small>{v.type==="snapshot"?"Snapshot":"Stable release"}</small></span>{version===v.id?<CheckCircle2 size={17}/>:<span className="version-option-radio"/>}</button>)}
        {visibleVersions.length===0&&<div className="custom-no-versions">No versions match that search.</div>}
      </div>
      <div className="custom-dialog-footer"><div><span>SELECTED VERSION</span><b>{selectedVersionVisible?version:"Choose a version"}</b></div><button className="primary-btn" onClick={add} disabled={!name.trim()||!selectedVersionVisible||creating}>{creating?"Creating…":<><Plus size={14}/> Create instance</>}</button></div>
    </section></div>}
  </section>;
}

function LogViewer({ text: raw, maxHeight=260 }) {
  const rows=String(raw||"").split(/\r?\n/).filter(Boolean).slice(-500);
  const parsed=rows.map((line,index)=>{
    let time="";
    let message=line;
    const unix=line.match(/^\[unix:(\d+)\]\s*(.*)$/);
    if(unix){time=new Date(Number(unix[1])*1000).toLocaleTimeString();message=unix[2];}
    else {
      const stamp=line.match(/^\[([^\]]+)\]\s*(.*)$/);
      if(stamp){time=stamp[1];message=stamp[2];}
    }
    const levelMatch=message.match(/\b(TRACE|DEBUG|INFO|WARN(?:ING)?|ERROR|FATAL)\b/i);
    let level=levelMatch?levelMatch[1].toUpperCase():"INFO";
    if(level==="WARNING")level="WARN";
    if(!levelMatch){
      if(/\b(error|failed|failure|exception|could not|cannot|crash|fatal)\b/i.test(message))level="ERROR";
      else if(/\b(warn|warning|retry|deprecated|missing)\b/i.test(message))level="WARN";
    }
    return {key:index,time,level,message};
  });
  return <div className="nova-log-viewer" style={{maxHeight}} role="log" aria-live="polite">
    {parsed.length===0?<div className="nova-log-empty">No log entries yet.</div>:parsed.map(row=><div className={"nova-log-row level-"+row.level.toLowerCase()} key={row.key}>
      <span className="nova-log-time">{row.time||"—"}</span><span className="nova-log-level">{row.level}</span><span className="nova-log-message">{row.message}</span>
    </div>)}
  </div>;
}

function SettingsPage({ memoryMb, setMemoryMb, updateInfo, updateBusy, updateError, updaterLogs, refreshUpdaterLogs, gameLogs, refreshGameLogs, onCheckUpdates, onInstallUpdate }) {
  return <section className="local-instances">
    <div className="eyebrow">NOVA PREFERENCES</div><h2>Launcher settings</h2>
    <p>Choose how much memory NOVA gives Minecraft when it launches. The setting is saved on this PC.</p>
    <div className="instance-create">
      <label htmlFor="nova-ram-setting">MAXIMUM MEMORY</label>
      <select id="nova-ram-setting" value={String(memoryMb)} onChange={e=>setMemoryMb(Number(e.target.value))}>
        <option value="2048">2 GB</option><option value="3072">3 GB</option><option value="4096">4 GB (default)</option><option value="6144">6 GB</option><option value="8192">8 GB</option>
      </select>
    </div>
    <div className="catalog-note">Restart Minecraft after changing this setting. Leave enough memory for Windows and other apps.</div>
    <div className="section-heading"><div><div className="eyebrow">GAME DIAGNOSTICS</div><h2>Minecraft launch logs</h2></div><button className="secondary-btn" onClick={refreshGameLogs}>Refresh game logs</button></div>
    <div className="feature-card">
      <b>Launcher & download activity</b>
      <p>Sign-in refresh, Minecraft preparation, downloads, launch errors and game exit status. No access tokens are written to this log.</p>
      <LogViewer text={gameLogs?.launcherLog||"No launcher activity logged yet."} maxHeight={260}/>
      <span className="feature-status">{gameLogs?.launcherLogPath||"%LOCALAPPDATA%\\NOVA Client\\logs\\game-launch.log"}</span>
    </div>
    <div className="feature-card">
      <b>Minecraft game log</b>
      <p>The game's own latest.log, including startup errors, mod errors and crash details.</p>
      <LogViewer text={gameLogs?.minecraftLog||"Minecraft latest.log has not been created yet."} maxHeight={320}/>
      <span className="feature-status">{gameLogs?.minecraftLogPath||"minecraft\\logs\\latest.log"}</span>
    </div>
    <div className="section-heading"><div><div className="eyebrow">NOVA SOFTWARE</div><h2>Updates</h2></div></div>
    <div className="feature-card updater-card">
      <div className="feature-icon blue"><ArrowDownToLine size={18}/></div>
      <b>{updateInfo?.available ? "A new NOVA version is ready" : "Keep NOVA up to date"}</b>
      <p>{updateInfo ? (updateInfo.available ? "Version "+updateInfo.latestVersion+" is available. Your current version is "+updateInfo.currentVersion+"." : (updateInfo.releasePublished === false ? "NOVA has not published its first release yet. Automatic updates will activate after a release is published." : "You're running the latest published version ("+updateInfo.currentVersion+").")) : "Check GitHub for the latest official NOVA release."}</p>
      {updateInfo?.available && <div className="catalog-note"><b>Release notes</b><p style={{whiteSpace:"pre-wrap",marginTop:8}}>{updateInfo.releaseNotes}</p></div>}
      {updateError && <div className="catalog-note error">{updateError}</div>}
      <div className="section-heading"><div><div className="eyebrow">DIAGNOSTICS</div><h2>Updater logs</h2></div><button className="secondary-btn" onClick={refreshUpdaterLogs}>Refresh logs</button></div>
      <pre className="catalog-note" style={{whiteSpace:"pre-wrap",overflowWrap:"anywhere",maxHeight:240,overflowY:"auto",fontSize:12,lineHeight:1.6}}>{updaterLogs}</pre>
      <span className="feature-status">LOG FILE: %LOCALAPPDATA%\NOVA Client\logs\updater.log</span>
      <div className="pack-install-row">
        <button className="secondary-btn" onClick={onCheckUpdates} disabled={updateBusy}>{updateBusy?"Checking…":"Check for updates"}</button>
        {updateInfo?.available && <button className="primary-btn" onClick={onInstallUpdate} disabled={updateBusy}><ArrowDownToLine size={14}/>{updateBusy?"Preparing update…":"Accept & restart"}</button>}
      </div>
      <span className="feature-status">{updateInfo ? "CURRENT VERSION "+updateInfo.currentVersion : "UPDATE STATUS NOT CHECKED"}</span>
    </div>
  </section>;
}

function PvpPage() {
  const modules = [
    ["FPS Counter","Displays the live client frame rate."],
    ["Keystrokes","Shows W, A, S and D key presses."],
    ["Clock","Displays the local time."],
    ["Coordinates","Shows your current block XYZ position."],
    ["Direction","Shows the compass direction you're facing."],
    ["Armor Status","Shows remaining durability for each equipped armor piece."]
  ];
  return <section className="local-instances">
    <div className="eyebrow">NOVA IN-GAME CLIENT</div>
    <h2>PvP toolkit</h2>
    <p>These HUD modules are included in the NOVA Fabric client. Open the in-game menu with <b>Right Shift</b> to switch modules on or off. Changes are saved automatically.</p>
    <div className="feature-grid">
      {modules.map(([name,description])=><article className="feature-card" key={name}>
        <div className="feature-icon purple"><Gamepad2 size={18}/></div>
        <b>{name}</b><p>{description}</p><span className="feature-status">IN CLIENT BUILD</span>
      </article>)}
    </div>
    <div className="catalog-note">No combat automation, aim assistance, or server-side modifications are included.</div>
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
  const [installDialog, setInstallDialog] = useState(null);
  const [downloadToastVisible, setDownloadToastVisible] = useState(false);
  const [downloadToastDismissed, setDownloadToastDismissed] = useState(false);
  const [downloadDetailsOpen, setDownloadDetailsOpen] = useState(false);
  const updateInstallDialog = (next) => { setInstallDialog(next); if(next?.status === "Starting installation"){setDownloadToastDismissed(false);setDownloadToastVisible(true);} else if(!downloadToastDismissed){setDownloadToastVisible(true);} };
  const [selectedVersion, setSelectedVersion] = useState("");
  const [versions, setVersions] = useState([]);
  const [versionsLoading, setVersionsLoading] = useState(true);
  const [loaderType, setLoaderType] = useState("vanilla");
  const [memoryMb, setMemoryMb] = useState(()=>{try{const n=Number(localStorage.getItem("nova.ramMb")||4096);return [2048,3072,4096,6144,8192].includes(n)?n:4096;}catch{return 4096;}});
  useEffect(()=>{try{localStorage.setItem("nova.ramMb",String(memoryMb));}catch{}},[memoryMb]);
  useEffect(()=>{
    let alive=true;
    let unlisten;
    listen("nova-pack-progress",event=>{
      const progress=event.payload||{};
      setInstallDialog(current=>current?{
        ...current,
        progress:typeof progress.percent==="number"?progress.percent:null,
        status:progress.phase==="preparing"?"Preparing Minecraft":progress.phase==="archive"?"Downloading modpack archive":current.status==="Installing mod"?"Downloading mod":"Downloading modpack files",
        detail:progress.detail||current.detail
      }:current);
    }).then(stop=>{if(alive)unlisten=stop;else stop();});
    return()=>{alive=false;unlisten?.();};
  },[]);

  useEffect(() => { refreshUpdaterLogs(); refreshGameLogs(); }, []);
  useEffect(() => {
    let alive = true;
    invoke("check_for_updates").then(info => { if (alive) { setUpdateInfo(info); setUpdateError(""); } })
      .catch(error => { if (alive) setUpdateError(typeof error === "string" ? error : (error.message || "Could not check for updates.")); });
    return () => { alive = false; };
  }, []);
  const refreshUpdaterLogs = async () => {
    try { setUpdaterLogs(await invoke("get_updater_logs")); }
    catch (error) { setUpdaterLogs(typeof error === "string" ? error : (error.message || "Could not load updater logs.")); }
  };
  const refreshGameLogs = async () => {
    try { setGameLogs(await invoke("get_game_logs", {instanceName:null})); }
    catch (error) { setGameLogs({launcherLog:typeof error === "string" ? error : (error.message || "Could not load launcher logs."),minecraftLog:""}); }
  };
  const handleCheckUpdates = async () => {
    setUpdateBusy(true); setUpdateError("");
    try { setUpdateInfo(await invoke("check_for_updates")); }
    catch (error) { setUpdateError(typeof error === "string" ? error : (error.message || "Could not check for updates.")); }
    finally { setUpdateBusy(false); refreshUpdaterLogs(); }
  };
  const handleInstallUpdate = async () => {
    setUpdateBusy(true); setUpdateError("");
    try { await invoke("install_update"); }
    catch (error) { setUpdateError(typeof error === "string" ? error : (error.message || "Could not install the update.")); setUpdateBusy(false); refreshUpdaterLogs(); }
  };
  const [launching, setLaunching] = useState(false);
  const [updateInfo, setUpdateInfo] = useState(null);
  const [updateBusy, setUpdateBusy] = useState(false);
  const [updateError, setUpdateError] = useState("");
  const [updaterLogs, setUpdaterLogs] = useState("Loading updater logs…");
  const [gameLogs, setGameLogs] = useState({launcherLog:"Loading launcher logs…",minecraftLog:"Loading Minecraft logs…"});
  const [signedIn, setSignedIn] = useState(false);
  const [accountName, setAccountName] = useState("");
  const [loginBusy, setLoginBusy] = useState(false);
  const [loginUrl, setLoginUrl] = useState("");
  const [message, setMessage] = useState("");
  const handleSignOut = async () => {
    try {
      await invoke("sign_out_minecraft");
      setSignedIn(false);
      setAccountName("");
      setMessage("You have been signed out of your Microsoft account.");
    } catch (error) {
      setMessage(typeof error === "string" ? error : (error.message || "Could not sign out."));
    }
  };
  useEffect(() => {
    let alive = true;
    invoke("minecraft_versions").then(manifest => {
      const entries = Array.isArray(manifest?.versions) ? manifest.versions.filter(v => typeof v.id === "string" && v.id.trim()).map(v => ({id:v.id,type:v.type || "release"})) : [];
      if (!entries.length) throw new Error("Minecraft returned an empty version list.");
      if (alive) { setVersions(entries); setSelectedVersion(manifest.latest?.release && entries.some(v=>v.id===manifest.latest.release) ? manifest.latest.release : entries[0].id); }
    }).catch(error => { if (alive) setMessage(typeof error === "string" ? error : (error.message || "Could not load Minecraft versions.")); })
      .finally(() => { if (alive) setVersionsLoading(false); });
    let offLink;
    let offComplete;
    let offGameExit;
    (async () => {
      const stopLink = await listen("nova-login-link", event => { if (alive) { setLoginUrl(event.payload); setMessage("Microsoft sign-in opened in your browser. Complete the sign-in there. If it did not open, use the link below."); } });
      if (alive) offLink = stopLink; else stopLink();
      const stopComplete = await listen("nova-login-complete", event => { if (alive) { setSignedIn(true); setAccountName(event.payload); setLoginUrl(""); setMessage("Microsoft account connected."); } });
      if (alive) offComplete = stopComplete; else stopComplete();
      const stopGameExit = await listen("nova-game-exit", event => {
        if (alive) {
          const result = event.payload || {};
          setMessage(result.success
            ? "Minecraft has closed."
            : "Minecraft exited unexpectedly. Check the game log for the crash details.");
          refreshGameLogs();
        }
      });
      if (alive) offGameExit = stopGameExit; else stopGameExit();
      try {
        const name = await invoke("minecraft_account_name");
        if (alive && name) { setSignedIn(true); setAccountName(name); }
      } catch (error) { if (alive) setMessage(String(error)); }
    })();
    return () => { alive = false; offLink?.(); offComplete?.(); offGameExit?.(); };
  }, []);
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
    try { await invoke("launch_minecraft", { version: selectedVersion, loader: loaderType, memoryMb }); setMessage("Minecraft has been started. Keep NOVA open while the game loads."); }
    catch (error) { setMessage(typeof error === "string" ? error : (error.message || "Minecraft could not be started.")); }
    finally { setLaunching(false); refreshGameLogs(); }
  };
  const titles = { home: ["Overview", "Everything you need, in one place."], instances: ["Instances", "Your game versions, neatly organized."], mods: ["Mods", "Build the Minecraft setup you want."], modpacks: ["Modpacks", "Discover a new way to play."], pvp: ["PvP features", "A focused toolkit for competitive play."], settings: ["Settings", "Make NOVA feel like yours."] };
  const [title, subtitle] = titles[page] || titles.home;
  return <div className="app-shell"><Sidebar page={page} setPage={setPage} signedIn={signedIn} accountName={accountName} onSignOut={handleSignOut}/><main className="main-area"><Topbar title={title} subtitle={subtitle} onShowDownloads={()=>{if(installDialog){setDownloadToastVisible(true);setDownloadDetailsOpen(true);}}}/>{updateInfo?.available && <div role="alert" style={{margin:"16px 28px 0",padding:"14px 18px",border:"1px solid rgba(100,160,255,.32)",borderRadius:14,background:"linear-gradient(110deg,rgba(45,105,190,.20),rgba(24,42,72,.72))",display:"flex",alignItems:"center",justifyContent:"space-between",gap:16,flexWrap:"wrap"}}><div style={{display:"flex",alignItems:"center",gap:12,minWidth:220}}><div className="feature-icon blue"><ArrowDownToLine size={18}/></div><div><b style={{display:"block"}}>Update available · NOVA {updateInfo.latestVersion}</b><span style={{fontSize:12,opacity:.78}}>Accept to install the update and restart the launcher.</span></div></div><button className="primary-btn" onClick={handleInstallUpdate} disabled={updateBusy}><ArrowDownToLine size={14}/>{updateBusy?"Downloading update…":"Accept & restart"}</button></div>}<div className="page-content">{page === "home" && <Home onNavigate={setPage} selectedVersion={selectedVersion} setSelectedVersion={setSelectedVersion} versions={versions} versionsLoading={versionsLoading} loaderType={loaderType} setLoaderType={setLoaderType} launching={launching} onLaunch={handleLaunch} onLogin={handleLogin} signedIn={signedIn} accountName={accountName} loginBusy={loginBusy} loginUrl={loginUrl} message={message}/>}<div style={{display:page==="mods"?"block":"none"}}><Catalog packs={false} active={page==="mods"} memoryMb={memoryMb} signedIn={signedIn} installDialog={installDialog} setInstallDialog={updateInstallDialog}/></div><div style={{display:page==="modpacks"?"block":"none"}}><Catalog packs={true} active={page==="modpacks"} memoryMb={memoryMb} signedIn={signedIn} installDialog={installDialog} setInstallDialog={updateInstallDialog}/></div>{page === "instances" && <LocalInstances versions={versions} memoryMb={memoryMb} setPage={setPage}/>}{page === "settings" && <SettingsPage memoryMb={memoryMb} setMemoryMb={setMemoryMb} updateInfo={updateInfo} updateBusy={updateBusy} updateError={updateError} updaterLogs={updaterLogs} refreshUpdaterLogs={refreshUpdaterLogs} gameLogs={gameLogs} refreshGameLogs={refreshGameLogs} onCheckUpdates={handleCheckUpdates} onInstallUpdate={handleInstallUpdate}/>}{page === "pvp" && <PvpPage/>}{!["home","mods","modpacks","instances","settings","pvp"].includes(page) && <EmptyPage page={page} setPage={setPage}/>}</div><footer className="main-footer"><span>© 2026 NOVA CLIENT</span><span>BUILT FOR THE NEXT SESSION <Sparkles size={12}/></span></footer></main>{installDialog&&downloadToastVisible&&<aside className="nova-download-toast" role="status" aria-live="polite">
    <div className="nova-download-toast-head"><div className={"nova-download-toast-icon"+(installDialog.done?(installDialog.error?" failed":" complete"):"")}><ArrowDownToLine size={17}/></div><div className="nova-download-toast-heading"><b>{installDialog.name}</b><span>{installDialog.done?(installDialog.error?"Installation failed":"Download complete"):installDialog.status||"Downloading files"}</span></div><button className="nova-download-toast-close" title="Hide notification (download continues)" onClick={()=>{setDownloadToastDismissed(true);setDownloadToastVisible(false);}}><X size={15}/></button></div>
    <div className="nova-download-toast-track"><div className={"nova-download-toast-fill"+(!installDialog.done?" active":"")+(installDialog.error?" failed":"")} style={{width:(installDialog.done?100:(typeof installDialog.progress==="number"?installDialog.progress:34))+"%"}}/></div>
    <div className="nova-download-toast-foot"><span>{typeof installDialog.progress==="number"?installDialog.progress+"%":installDialog.done?"Finished":"Working in background"}</span><button onClick={()=>setDownloadDetailsOpen(v=>!v)}>{downloadDetailsOpen?"Hide details":"Details"}</button></div>
    {downloadDetailsOpen&&<div className="nova-download-toast-details">{installDialog.detail}{installDialog.error&&<p>{installDialog.error}</p>}</div>}
  </aside>}</div>;
}

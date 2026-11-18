// ===== HOME OS · helpers (paste at top of every use_figma script) =====
const fam="Plus Jakarta Sans"; for (const s of ["Regular","Medium","SemiBold","Bold","ExtraBold"]) await figma.loadFontAsync({family:fam,style:s});
const VID = {base:"VariableID:2:13",vidrio:"VariableID:2:14",fuerte:"VariableID:2:15",hover:"VariableID:2:16",panel:"VariableID:2:17",velo:"VariableID:2:18",foco:"VariableID:2:19",focoTxt:"VariableID:2:20",borde:"VariableID:2:21",bordeHover:"VariableID:2:22",t1:"VariableID:2:23",t2:"VariableID:2:24",t3:"VariableID:2:25",acento:"VariableID:2:26",ok:"VariableID:2:27",aviso:"VariableID:2:28",error:"VariableID:2:29",sinapi:"VariableID:2:30"};
const VARS={}; for (const [k,id] of Object.entries(VID)) VARS[k]=await figma.variables.getVariableByIdAsync(id);
const paint = k => figma.variables.setBoundVariableForPaint({type:'SOLID',color:{r:1,g:1,b:1}},'color',VARS[k]);
const TS = {hora:"S:f4003efc547a17439de6c6c6be8bbf9fe3d4fb04,",titulo:"S:0aaab5e2792fb5ae71a5b1f4b2f31f040e756f18,",seccion:"S:f34fe550d8a77f47d1154ffb2b608d9275388641,",grande:"S:43611caeced9343384fd07483f4ac23b2563498b,",cuerpo:"S:907320fe58144260d594a2182046e8cf319a9f76,",peq:"S:ef9c36cc3eb00dbda90be97723c30e54d856c080,",etq:"S:605dcd9be2eb32093b92beadd420adb3af9c967f,",etqP:"S:99d049e8b59591f24aef6b276272c1efdab1b17d,",mono:"S:769c6081544c245dfe2965f7f23611ad436bba44,"};
const text = async (str, style="cuerpo", color="t1", width) => { const t=figma.createText(); await t.setTextStyleIdAsync(TS[style]); t.characters=str; t.fills=[paint(color)]; if (width){ t.resize(width, t.height); t.textAutoResize="HEIGHT"; } return t; };
const WP = {dunas:"f9c0374265096214e80eedeed40c2468739c6d71",bosque:"e279286bdcabdd4fc4a7bb7623fd6bebbf4ea272",aurora:"3a1aeb9fb46851014b582e571fcf314b9447dd8d",ciudad:"958fdc43a87366b6ed74afd9bc324cdee52087bd",cumbre:"60fff109eb992c1fe5d54924d1f805ff0686739e",yosemite:"9d55e09bb08295efd60a0d63caa524c40932de7b"};
const COMP = {tile:"4:19",boton:"4:95",pestana:"4:104",pildora:"4:145",tarjeta:"4:194",deslizador:"5:55",interruptor:"5:60",fila:"5:93",cursor:"5:102",toast:"5:130",insignia:"5:146",tecla:"5:153",botonIcono:"6:73",barra:"6:74",ayuda:"5:103"};
const ICON = {inicio:"3:6",apps:"3:12",entrada:"3:17",buscar:"3:21",ajustes:"3:25",volumen:"3:30",silencio:"3:35",brillo:"3:46",reposo:"3:49",temporizador:"3:54",airplay:"3:58",imagen:"3:63",sonido:"3:68",wifi:"3:74",bluetooth:"3:77",encendido:"3:81",chevron:"3:84",check:"3:87",info:"3:92",aviso:"3:97",cerrar:"3:101",tv:"3:105",reloj:"3:109",clima:"3:112",notificacion:"3:116",mover:"3:124",ocultar:"3:130",idioma:"3:135",sistema:"3:147",candado:"3:151",puntero:"3:155",mando:"3:159",microfono:"3:164",mas:"3:168",atras:"3:172",pantalla_off:"3:179",hdmi:"3:185",recargar:"3:191",recientes:"3:196",red_off:"3:205",editar:"3:209",estrella:"3:212"};
// Component property keys
const K = {tileNombre:"Nombre#4:0",botonEtq:"Etiqueta#4:5",botonConIcono:"Con icono#4:21",botonIcono:"Icono#4:37",pestEtq:"Etiqueta#4:53",pilNombre:"Nombre#4:58",pilDetalle:"Detalle#4:63",tarTitulo:"Título#4:68",tarDetalle:"Detalle#4:77",tarIcono:"Icono#4:86",desValor:"Valor#5:0",desIcono:"Icono#5:5",filaTitulo:"Título#5:10",filaValor:"Valor#5:15",filaConIcono:"Con icono#5:20",filaIcono:"Icono#5:25",filaFlecha:"Con flecha#5:30",toastTitulo:"Título#5:37",toastTexto:"Texto#5:41",teclaLetra:"Letra#5:45",biIcono:"Icono#6:0",barraHora:"Hora#6:5",barraClima:"Clima#6:6",ayTecla:"Tecla#5:35",ayAccion:"Acción#5:36"};
const _sets={}; const setOf = async k => _sets[k] || (_sets[k]=await figma.getNodeByIdAsync(COMP[k]));
// inst("tile",{Estado:"Foco"},{[K.tileNombre]:"Netflix"})
const inst = async (k, variant={}, props={}) => { const n=await setOf(k); let i; if (n.type==="COMPONENT_SET") { i=n.defaultVariant.createInstance(); } else i=n.createInstance(); const all={...variant,...props}; if (Object.keys(all).length) i.setProperties(all); return i; };
// Recolor every stroke inside an icon (needed after INSTANCE_SWAP in Foco variants: swapped icons stay white)
const tint = (node, colorKey) => { node.findAll(n=>'strokes' in n && n.type!=='INSTANCE' && n.type!=='TEXT' && n.strokes.length>0 && n.parent && n.parent.type!=='COMPONENT_SET').forEach(n=>{}); const icons = node.type==='INSTANCE' && node.name.startsWith('Icono') ? [node] : node.findAll(n=>n.type==='INSTANCE' && (n.mainComponent?.name||'').startsWith('Icono/')); for (const ic of icons) ic.findAll(n=>'strokes' in n && n.type!=='INSTANCE' && n.strokes.length>0).forEach(n=>n.strokes=[paint(colorKey)]); };
const icon = async (key, color="t1", size=32) => { const i=(await figma.getNodeByIdAsync(ICON[key])).createInstance(); if (size!==32) i.resize(size,size); i.findAll(n=>'strokes' in n && n.strokes.length>0).forEach(n=>n.strokes=[paint(color)]); return i; };
const shadowFoco = [{type:'DROP_SHADOW',color:{r:0,g:0,b:0,a:0.5},offset:{x:0,y:24},radius:48,spread:0,visible:true,blendMode:'NORMAL'}];
const blur = r => ({type:'BACKGROUND_BLUR',radius:r,visible:true});
// Glass panel (auto-layout)
const glass = (dir="VERTICAL", props={}) => { const f=figma.createAutoLayout(dir,props); f.fills=[paint("panel")]; f.strokes=[paint("borde")]; f.strokeWeight=1.5; f.cornerRadius=props.cornerRadius??40; f.effects=[blur(60)]; return f; };
// New 1920x1080 screen with wallpaper + legibility gradients. slot = index on page row.
const screen = (name, slot, wp="cumbre", {dim=0, row=0}={}) => {
  const f=figma.createFrame(); f.name=name; f.resize(1920,1080); f.x=slot*2020; f.y=row*1700; f.clipsContent=true;
  f.fills=[{type:'IMAGE',imageHash:WP[wp],scaleMode:'FILL'}];
  const g=figma.createRectangle(); g.name="Velo legibilidad"; g.resize(1920,1080); f.appendChild(g);
  g.fills=[{type:'GRADIENT_LINEAR',gradientTransform:[[0,1,0],[-1,0,1]],gradientStops:[{position:0,color:{r:0,g:0,b:0,a:0.55}},{position:0.45,color:{r:0,g:0,b:0,a:0.22}},{position:0.62,color:{r:0,g:0,b:0,a:0.3}},{position:1,color:{r:0,g:0,b:0,a:0.85}}]}];
  if (dim) { const d=figma.createRectangle(); d.name="Atenuado"; d.resize(1920,1080); d.fills=[{type:'SOLID',color:{r:0,g:0,b:0},opacity:dim}]; f.appendChild(d); }
  return f;
};
// Hint bar at the bottom: hints([["OK","Abrir"],["Atrás","Volver"]])
const hints = async (frame, list) => { const r=figma.createAutoLayout("HORIZONTAL",{name:"Ayudas de mando",itemSpacing:40}); r.fills=[]; for (const [k,a] of list) r.appendChild(await inst("ayuda",{},{[K.ayTecla]:k,[K.ayAccion]:a})); frame.appendChild(r); r.x=96; r.y=1080-54-r.height; return r; };
// Pointer cursor at (x,y); tipo Flecha|Mano|Espera
const cursor = async (frame, x, y, tipo="Mano") => { const c=await inst("cursor",{Tipo:tipo}); frame.appendChild(c); c.x=x; c.y=y; c.name="Puntero"; return c; };
// Annotation panel below a screen: notes(frame, "Título", ["línea", ...], [["Comprobado","texto API"], ...])
const notes = async (frame, title, lines, apis=[]) => {
  const n=figma.createAutoLayout("VERTICAL",{name:"Notas · "+frame.name,itemSpacing:16,paddingLeft:48,paddingRight:48,paddingTop:40,paddingBottom:40});
  n.fills=[{type:'SOLID',color:{r:0.09,g:0.09,b:0.12}}]; n.cornerRadius=28; n.primaryAxisSizingMode="AUTO"; n.counterAxisSizingMode="FIXED"; n.resize(1920,100);
  frame.parent.appendChild(n); n.x=frame.x; n.y=frame.y+1080+40;
  n.appendChild(await text(title,"seccion","t1"));
  for (const l of lines) n.appendChild(await text("• "+l,"cuerpo","t2",1820));
  for (const [tipo,line] of apis) { const r=figma.createAutoLayout("HORIZONTAL",{itemSpacing:16}); r.fills=[]; r.counterAxisAlignItems="CENTER"; r.appendChild(await inst("insignia",{Tipo:tipo})); r.appendChild(await text(line,"mono","t1",1560)); n.appendChild(r); }
  return n;
};
const APP_ART = {"Netflix":[[0.9,0.04,0.08],[0.35,0.0,0.02]],"YouTube":[[1,0.0,0.2],[0.45,0.0,0.05]],"Prime Video":[[0.0,0.66,0.88],[0.0,0.2,0.35]],"Disney+":[[0.12,0.5,0.88],[0.04,0.12,0.36]],"Apple TV":[[0.25,0.25,0.27],[0.05,0.05,0.06]],"HBO Max":[[0.45,0.2,0.9],[0.12,0.04,0.3]],"Spotify":[[0.12,0.84,0.38],[0.03,0.25,0.1]],"Plex":[[0.9,0.63,0.05],[0.3,0.2,0.0]],"LG Channels":[[0.65,0.0,0.3],[0.2,0.0,0.1]],"Navegador":[[0.3,0.5,0.95],[0.1,0.15,0.35]],"Amazon Alexa":[[0.0,0.75,0.95],[0.0,0.25,0.4]],"Galería":[[0.95,0.5,0.3],[0.35,0.12,0.05]]};
// tile("Netflix","Foco") — app tile with brand-ish gradient (no logos: trademarks)
const tile = async (name, estado="Reposo") => { const t=await inst("tile",{Estado:estado},{[K.tileNombre]:name}); const a=t.findOne(n=>n.name==="Arte"); const c=APP_ART[name]||[[0.3,0.32,0.4],[0.1,0.1,0.14]]; a.fills=[{type:'GRADIENT_LINEAR',gradientTransform:[[0.7,0.7,0],[-0.7,0.7,0.5]],gradientStops:[{position:0,color:{r:c[0][0],g:c[0][1],b:c[0][2],a:1}},{position:1,color:{r:c[1][0],g:c[1][1],b:c[1][2],a:1}}]}]; t.name="App/"+name; return t; };
// Top bar with an active tab: bar(frame, "Inicio"|"Apps"|"Entradas"|"Buscar"|null, {focusTab, hoverTab, hoverAction})
const bar = async (frame, active="Inicio", opt={}) => { const b=await inst("barra"); frame.appendChild(b); b.x=96; b.y=54; for (const t of b.findAll(n=>n.type==='INSTANCE' && n.name.startsWith('Pestaña/'))) { const nm=t.name.split('/')[1]; let e = nm===active?"Activa":"Reposo"; if (opt.focusTab===nm) e="Foco"; if (opt.hoverTab===nm) e="Hover"; t.setProperties({Estado:e}); } if (opt.hoverAction||opt.focusAction) for (const a of b.findAll(n=>n.type==='INSTANCE' && n.name.startsWith('Acción/'))) { const nm=a.name.split('/')[1]; if (nm===opt.hoverAction) a.setProperties({Estado:"Hover"}); if (nm===opt.focusAction) { a.setProperties({Estado:"Foco"}); tint(a,"focoTxt"); } } return b; };
// ===== end helpers =====

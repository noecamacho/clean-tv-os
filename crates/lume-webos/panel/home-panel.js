/* HOME OS quick panel: a key filter for LG's compositor (webOS 6, surface-manager).
 *
 * scripts/lg-panel.sh copies this folder to /var/lib/home-os/panel and registers
 * homePanelKey first in com.webos.surfacemanager.keyFilters. The compositor loads this file
 * with Qt.include, calls init(), then hands every remote key to homePanelKey before LG's own
 * filters. A short press of Settings opens HOME's panel (HomePanel.qml) over whatever is on
 * screen; holding it opens LG's settings, as before. While the panel is open it takes the
 * arrows, OK and Back; any other key closes it and goes on to the app.
 *
 * ES5 only (LG's QML engine). Every entry point catches its own errors: a throw here must never
 * leave the remote dead. Globals from LG's loader and engine: lunaCall, genericTimer,
 * compositorWindow, KeyPolicy, WebOS, Qt.
 */

// scripts/lg-panel.sh installs every version in its own folder and writes it here: LG's QML
// engine caches components by path, so a reinstall in place kept running the old QML.
var homePanelDir = '/var/lib/home-os/panel';
var homePanel = null; // created on the first Settings press
var homePanelLang = 'es';
var homePanelDown = false; // Settings is held
var homePanelLong = false; // this hold already opened LG's settings
var homePanelPress = 0; // counts Settings presses, so an old long-press timer can tell
var homePanelIdle = 0; // counts activity, so an old auto-close timer can tell
var homePanelWrites = {}; // level -> value waiting for the write timer
var homePanelWriteQueued = false;
// Writes per level: a read that went out before the last write must not undo it on screen.
var homePanelWriteCount = { brightness: 0, volume: 0 };
var homePanelTestSeq = -1;
// Where the sound goes. Over HDMI ARC the TV's own volume does not reach the soundbar (the
// remote talks to it another way; master/volumeUp and com.webos.audio/volumeUp were tried and
// the Sonos ignored both), so with a Sonos soundbar the slider drives the Sonos itself over its
// local UPnP API. scripts/lg-panel.sh finds the soundbar and writes its address here.
// shortcut: the address is fixed at install; reinstall (or reserve it in the router) if it moves.
var homePanelSonos = '';
var homePanelSoundOutput = '';
// Arrow steps pressed while a level was still unknown: applied when the read answers.
var homePanelPending = { brightness: 0, volume: 0 };
var homePanelPolling = false;

// Picture modes per content type, in LG's menu order. The TV refuses a mode that does not apply
// (photo in SDR, any dolbyHdr* outside Dolby Vision): cycling skips refusals.
var HOME_PANEL_MODE_LISTS = {
  sdr: ['vivid', 'normal', 'eco', 'cinema', 'sports', 'game', 'filmMaker', 'expert1', 'expert2'],
  hdr: ['hdrVivid', 'hdrStandard', 'hdrCinemaBright', 'hdrCinema', 'hdrGame', 'hdrFilmMaker'],
  dolbyHdr: ['dolbyHdrVivid', 'dolbyHdrStandard', 'dolbyHdrCinemaBright', 'dolbyHdrCinema', 'dolbyHdrDark', 'dolbyHdrGame']
};
var homePanelMode = ''; // the raw pictureMode on screen

// Night mode. Picture settings live per input, content type and picture mode, and the TV only
// writes the ones on screen: night values are applied (and the owner's values kept) per mode,
// each time a mode is on screen. State and schedule persist in com.webos.service.config.
var HOME_PANEL_NIGHT_KEYS = ['backlight', 'peakBrightness', 'dynamicContrast', 'eyeComfortMode'];
var homePanelNight = { on: false, saved: {}, manual: false };
var homePanelSchedule = { enabled: true, from: '22:00', to: '07:00', level: 0 };
var homePanelInWindow = null; // last schedule check; null until the first, which may act
var homePanelSyncing = false;
var homePanelScreenOffArmed = false; // OK went down on "Screen off": act on its release
var homePanelError = ''; // the last load error, reported by the test channel
var homePanelSeen = []; // the last keys the filter saw, reported by the test channel

var HOME_PANEL_LONG_MS = 600;
var HOME_PANEL_IDLE_MS = 10000;
var HOME_PANEL_WRITE_MS = 80;
var HOME_PANEL_STEP = 5; // as HOME's Control Center
// Focus, by rows (docs/design/panel-rapido-lg.md, night mode moved down at the owner's request):
// 0 brightness | 3 volume | 4 picture mode | 1 night mode, 5 screen off, 6 more settings. No wrap-around at the edges; a row of two
// halves remembers the half last used. (2, the schedule button, was removed at the owner's
// request: the schedule runs by itself, set with scripts/lg-panel.sh night.)
var HOME_PANEL_ROWS = [[0], [3], [4], [1, 5, 6]];
var HOME_PANEL_SLIDERS = { 0: 'brightness', 3: 'volume' };
var homePanelHalf = {}; // row -> the half (0/1) last focused

var HOME_PANEL_TEXT = {
  es: { brightness: 'Brillo', volume: 'Volumen', screenOff: 'Apagar pantalla', settings: 'Más ajustes', muted: 'Silenciado',
        tvSpeakers: 'Altavoces de la TV', bluetooth: 'Bluetooth', optical: 'Óptico', arc: 'HDMI ARC',
        pictureMode: 'Modo de imagen', night: 'Modo noche', off: 'Desactivado', on: 'Activado',
        nightPrefix: 'Noche · ', nightBlocksAuto: 'Primero desactiva Modo noche', offline: 'Sin conexión',
        until: 'Hasta las ', schedule: 'Horario' },
  en: { brightness: 'Brightness', volume: 'Volume', screenOff: 'Screen off', settings: 'More settings', muted: 'Muted',
        tvSpeakers: 'TV speakers', bluetooth: 'Bluetooth', optical: 'Optical', arc: 'HDMI ARC',
        pictureMode: 'Picture mode', night: 'Night mode', off: 'Off', on: 'On',
        nightPrefix: 'Night · ', nightBlocksAuto: 'Turn off Night mode first', offline: 'Not connected',
        until: 'Until ', schedule: 'Schedule' }
};

// Picture modes as LG's menus name them (the HDR and Dolby variants share the base name).
var HOME_PANEL_MODES = {
  es: { vivid: 'Vívido', standard: 'Estándar', normal: 'Estándar', eco: 'Eco', cinema: 'Cine', cinemaBright: 'Cine brillante',
        sports: 'Deportes', game: 'Juegos', filmMaker: 'Filmmaker', photo: 'Foto', expert1: 'Experto (claro)',
        expert2: 'Experto (oscuro)', dark: 'Oscuro' },
  en: { vivid: 'Vivid', standard: 'Standard', normal: 'Standard', eco: 'Eco', cinema: 'Cinema', cinemaBright: 'Cinema Home',
        sports: 'Sports', game: 'Game', filmMaker: 'Filmmaker', photo: 'Photo', expert1: 'Expert (bright)',
        expert2: 'Expert (dark)', dark: 'Dark' }
};

// What the Sonos soundbar receives (GetZoneInfo HTAudioIn), as the SoCo library names the codes.
var HOME_PANEL_SONOS_FORMATS = {
  2: 'Stereo', 7: 'Dolby 2.0', 18: 'Dolby 5.1', 59: 'Dolby Atmos (DD+)', 61: 'Dolby Atmos (TrueHD)',
  63: 'Dolby Atmos', 33554434: 'PCM 2.0', 33554488: 'Dolby 2.0', 33554490: 'Dolby Digital Plus 2.0',
  33554492: 'Dolby TrueHD 2.0', 33554494: 'PCM 2.0', 84934658: 'PCM 5.1', 84934713: 'Dolby 5.1',
  84934714: 'Dolby Digital Plus 5.1', 84934716: 'Dolby TrueHD 5.1', 84934718: 'PCM 5.1', 84934721: 'DTS 5.1',
  118489090: 'PCM 7.1', 118489146: 'Dolby Digital Plus 7.1', 118489148: 'Dolby TrueHD 7.1'
};

function homePanelLog(text) {
  homePanelError = text;
  console.info('home panel: ' + text);
}

function homePanelParse(payload) {
  return typeof payload === 'string' ? JSON.parse(payload) : payload;
}

function homePanelSend(service, method, payload, callback) {
  lunaCall.send(service, method, JSON.stringify(payload), function (reply) {
    if (!callback) return;
    try {
      callback(homePanelParse(reply));
    } catch (error) {
      homePanelLog(method + ': ' + error);
    }
  });
}

// Called by LG's compositor once the file is loaded.
// oxlint-disable-next-line no-unused-vars
function init() {
  try {
    homePanelRemoveStale();
    homePanelSend('com.webos.settingsservice', '/getSystemSettings', { keys: ['localeInfo'] }, function (r) {
      var ui = r && r.settings && r.settings.localeInfo && r.settings.localeInfo.locales
        ? r.settings.localeInfo.locales.UI || '' : '';
      homePanelLang = ui.indexOf('es') === 0 ? 'es' : 'en';
      if (homePanel) homePanel.strings = HOME_PANEL_TEXT[homePanelLang];
    });
    homePanelSend('com.webos.service.config', '/getConfigs', { configNames: ['com.webos.surfacemanager.homePanelNight'] }, function (r) {
      var n = r && r.configs ? r.configs['com.webos.surfacemanager.homePanelNight'] : null;
      if (n && typeof n.on === 'boolean') homePanelNight = { on: n.on, saved: n.saved || {}, manual: n.manual === true };
      homePanelTick();
    });
    lunaCall.subscribe(
      'com.webos.service.config',
      '/getConfigs',
      JSON.stringify({ configNames: ['com.webos.surfacemanager.homePanelSchedule'] }),
      function (payload) {
        try {
          var r = homePanelParse(payload);
          var c = r && r.configs ? r.configs['com.webos.surfacemanager.homePanelSchedule'] : null;
          if (c) {
            homePanelSchedule = {
              enabled: c.enabled !== false, from: c.from || '22:00', to: c.to || '07:00',
              level: typeof c.level === 'number' ? c.level : 0
            };
            // A new schedule acts at its next edge, not at once (that would undo a change by hand).
          }
          homePanelShowNight();
        } catch (error) {
          homePanelLog('schedule: ' + error);
        }
      },
      true
    );
    // Test channel: scripts/lg-panel.sh test writes {"cmd":…,"n":…} here (see homePanelTest).
    lunaCall.subscribe(
      'com.webos.service.config',
      '/getConfigs',
      JSON.stringify({ configNames: ['com.webos.surfacemanager.homePanelTest'] }),
      function (payload) {
        try {
          var r = homePanelParse(payload);
          var cmd = r && r.configs ? r.configs['com.webos.surfacemanager.homePanelTest'] : null;
          if (!cmd || typeof cmd.n !== 'number') return;
          // The first answer is whatever was left there: remember it, do not replay it.
          if (homePanelTestSeq < 0) {
            homePanelTestSeq = cmd.n;
            return;
          }
          if (cmd.n === homePanelTestSeq) return;
          homePanelTestSeq = cmd.n;
          homePanelTest(cmd);
        } catch (error) {
          homePanelLog('test: ' + error);
        }
      },
      true
    );
  } catch (error) {
    homePanelLog('init: ' + error);
  }
}

// A reload of the filter list (or of this file) leaves the old panel in the views: drop it.
function homePanelRemoveStale() {
  var parent = homePanelParent();
  if (!parent) return;
  for (var i = parent.children.length - 1; i >= 0; i--) {
    if (parent.children[i].objectName === 'homeOsPanel') parent.children[i].destroy();
  }
}

function homePanelParent() {
  // The loader hands filters proxy objects, not the views; the compositor window's root item is
  // reachable, and what is added to it last draws over everything, video included.
  if (typeof compositorWindow !== 'undefined' && compositorWindow && compositorWindow.contentItem)
    return compositorWindow.contentItem;
  return null;
}

function homePanelEnsure() {
  if (homePanel) return homePanel;
  var parent = homePanelParent();
  if (!parent) {
    homePanelLog('no views root');
    return null;
  }
  var component = Qt.createComponent('file://' + homePanelDir + '/HomePanel.qml');
  if (component.status !== 1 /* Component.Ready */) {
    homePanelLog('cannot load HomePanel.qml: ' + component.errorString());
    return null;
  }
  homePanel = component.createObject(parent, {
    dir: 'file://' + homePanelDir,
    strings: HOME_PANEL_TEXT[homePanelLang],
    // The views root (the panel is its sibling, not inside it): what the glass blurs.
    backdropSource: parent.children.length ? parent.children[0] : null
  });
  if (!homePanel) {
    homePanelLog('cannot create the panel');
    return null;
  }
  homePanel.action.connect(homePanelAction);
  homePanel.levelPicked.connect(homePanelSet);
  return homePanel;
}

function homePanelIsOpen() {
  return !!homePanel && homePanel.shown;
}

function homePanelOpen() {
  var panel = homePanelEnsure();
  if (!panel) return;
  panel.focusIndex = 0;
  homePanelPending = { brightness: 0, volume: 0 };
  homePanelShowNight();
  panel.brightnessMessage = '';
  panel.open();
  homePanelNightSync();
  homePanelReadInfo();
  // The pointer that appears with ⚙ is the remote's input service's doing ("Cursor Show(SHOW
  // KEY_SETTINGS Press)" in mrcu/getCursorHistory); hiding it from the compositor did not work.
  // The only switch is mrcu/setHotkeyCursorPolicy, whose button codes are undocumented: left as is.
  homePanelRead();
  homePanelTouch();
}

function homePanelClose() {
  if (homePanel) homePanel.close();
}

// What is on screen and in the speakers, for the two lines over the controls: read once per
// opening (videooutput, the picture mode of the current input, the soundbar's input format).
function homePanelReadInfo() {
  homePanel.videoBadges = [];
  homePanel.audioBadges = [];
  homePanel.audioDevice = '';
  var video = { kind: '', size: '', rate: '' };
  var showVideo = function () {
    homePanel.videoBadges = [video.kind || 'SDR', video.size, video.rate].filter(function (x) { return x; });
  };
  homePanelSend('com.webos.service.videooutput', '/getStatus', {}, function (r) {
    var main = null;
    for (var i = 0; r && r.video && i < r.video.length; i++) if (r.video[i].sink === 'MAIN' && r.video[i].connected) main = r.video[i];
    // Glass: blur only over graphics. A video on screen is on a hardware plane the compositor
    // cannot read (the snapshot would be empty there), so over video the glass stays clear.
    homePanel.frost = main ? 0 : 1;
    if (homePanel.frost > 0) homePanel.refreshBackdrop();
    if (main) {
      var hdr = main.videoInfo ? main.videoInfo.hdrType || '' : '';
      video.kind = hdr === 'DolbyVision' ? 'Dolby Vision' : /HDR10|HLG/i.test(hdr) ? hdr : 'SDR';
      var h = parseInt(main.height, 10);
      video.size = h >= 2160 ? '4K' : h > 0 ? h + 'p' : '';
      var fps = parseFloat(main.frameRate);
      video.rate = fps > 0 ? Math.round(fps) + 'p' : '';
    }
    showVideo();
  });
  homePanelSend('com.webos.settingsservice', '/getSystemSettings', { category: 'picture', keys: ['pictureMode'] }, function (r) {
    var raw = r && r.settings ? String(r.settings.pictureMode || '') : '';
    // dolbyHdrCinemaBright -> cinemaBright, hdrVivid -> vivid
    var base = raw.replace(/^dolbyHdr|^hdr/, '');
    base = base.charAt(0).toLowerCase() + base.slice(1);
    homePanelMode = raw;
    if (homePanelModeTarget >= 0) return; // a change is on its way: this read must not undo it
    homePanel.pictureMode = homePanelModeName(raw);
    homePanel.modePending = false;
    homePanelShowModePosition();
  });
  homePanelSend('com.webos.service.audio', '/master/getVolume', {}, function (r) {
    var out = r && r.volumeStatus ? r.volumeStatus.soundOutput || '' : '';
    homePanelSoundOutput = out || homePanelSoundOutput;
    var t = HOME_PANEL_TEXT[homePanelLang];
    if (homePanelUsesSonos()) {
      homePanel.audioDevice = 'Sonos';
      homePanelSoap('DeviceProperties', 'DeviceProperties', 'GetZoneInfo', '', function (xml) {
        var m = /<HTAudioIn>(\d+)</.exec(xml || '');
        homePanel.audioBadges = homePanelFormatBadges(m ? HOME_PANEL_SONOS_FORMATS[parseInt(m[1], 10)] : '');
      });
    } else {
      homePanel.audioDevice = out.indexOf('tv_speaker') === 0 ? t.tvSpeakers
        : out.indexOf('bt') === 0 ? t.bluetooth : out.indexOf('optical') >= 0 ? t.optical
        : out.indexOf('arc') >= 0 ? t.arc : '';
    }
  });
}

function homePanelModeName(raw) {
  // dolbyHdrCinemaBright -> cinemaBright, hdrVivid -> vivid
  var base = String(raw || '').replace(/^dolbyHdr|^hdr/, '');
  base = base.charAt(0).toLowerCase() + base.slice(1);
  return HOME_PANEL_MODES[homePanelLang][base] || raw;
}

function homePanelModeRange() {
  return homePanelMode.indexOf('dolbyHdr') === 0 ? 'dolbyHdr' : homePanelMode.indexOf('hdr') === 0 ? 'hdr' : 'sdr';
}

// Shows where the mode on screen is in its list (the dots under the selector).
function homePanelShowModePosition() {
  if (!homePanel) return;
  var list = HOME_PANEL_MODE_LISTS[homePanelModeRange()];
  homePanel.modeCount = list.length;
  homePanel.modeIndex = list.indexOf(homePanelMode);
}

// Left/Right on the picture-mode row: only the name changes at once; the mode is written 600 ms
// after the last press, so walking past "Vívido" on the way to "Filmmaker" never lights the room.
// No wrap-around. A mode the TV refuses here is dropped from the list for this session.
var homePanelModeTarget = -1;
var homePanelModeTimer = 0;
function homePanelCycleMode(direction) {
  if (!homePanelMode) return; // not read yet: never guess where the list starts
  var list = HOME_PANEL_MODE_LISTS[homePanelModeRange()];
  var from = homePanelModeTarget >= 0 ? homePanelModeTarget : list.indexOf(homePanelMode);
  var to = Math.max(0, Math.min(list.length - 1, (from < 0 ? 0 : from) + direction));
  if (to === from) return;
  homePanelModeTarget = to;
  if (homePanel) {
    homePanel.pictureMode = homePanelModeName(list[to]);
    homePanel.modePending = list[to] !== homePanelMode;
    homePanel.modeIndex = to;
    homePanel.modeCount = list.length;
  }
  var mine = ++homePanelModeTimer;
  genericTimer.set(600, function () {
    if (mine === homePanelModeTimer) homePanelApplyMode(direction);
  });
}

function homePanelApplyMode(direction) {
  var range = homePanelModeRange();
  var list = HOME_PANEL_MODE_LISTS[range];
  var target = list[homePanelModeTarget];
  homePanelModeTarget = -1;
  if (!target || target === homePanelMode) {
    if (homePanel) homePanel.modePending = false;
    return;
  }
  homePanelSend('com.webos.settingsservice', '/setSystemSettings', { category: 'picture', settings: { pictureMode: target } }, function (r) {
    if (r && r.returnValue) {
      homePanelMode = target;
      if (homePanel) {
        homePanel.pictureMode = homePanelModeName(target);
        homePanel.modePending = false;
        homePanel.brightness = -1; // each mode has its own OLED light: "…" until it is read
      }
      homePanelShowModePosition();
      homePanelRead();
      homePanelNightSync();
    } else {
      // Refused for this content: forget it and try the next one the same way.
      list.splice(list.indexOf(target), 1);
      var at = list.indexOf(homePanelMode);
      homePanelModeTarget = Math.max(0, Math.min(list.length - 1, list.indexOf(target) >= 0 ? at : at + direction));
      if (list[homePanelModeTarget] === homePanelMode) {
        if (homePanel) {
          homePanel.pictureMode = homePanelModeName(homePanelMode);
          homePanel.modePending = false;
        }
        homePanelModeTarget = -1;
        homePanelShowModePosition();
        return;
      }
      homePanelApplyMode(direction);
    }
  });
}

// "Dolby Digital Plus 5.1" -> ["Dolby Digital Plus", "5.1"]; "Dolby Atmos (DD+)" -> ["Dolby Atmos", "DD+"].
function homePanelFormatBadges(format) {
  if (!format) return [];
  var m = /^(.*?)\s*\(([^)]+)\)$/.exec(format) || /^(.*?)\s+(\d\.\d)$/.exec(format);
  return m ? [m[1], m[2]] : [format];
}

// Reads both levels; the panel shows them as they arrive.
function homePanelRead() {
  var writes = homePanelWriteCount.brightness;
  // Asked apart: a key the current input lacks fails the whole reply.
  homePanelSend('com.webos.settingsservice', '/getSystemSettings', { category: 'picture', keys: ['energySaving'] }, function (r) {
    if (homePanel && r && r.settings) homePanel.autoBrightness = r.settings.energySaving === 'auto';
    if (homePanel && homePanel.autoBrightness) homePanelPollAuto();
  });
  homePanelSend('com.webos.settingsservice', '/getSystemSettings', { category: 'picture', keys: ['backlight'] }, function (r) {
    if (writes !== homePanelWriteCount.brightness) return;
    // The compositor's bus hands numbers back as text ("40"): accept both.
    var level = r && r.settings ? parseInt(r.settings.backlight, 10) : NaN;
    // In Auto the setting stays put while the TV dims by itself: the real level comes from
    // the panel driver (homePanelPollAuto), so the setting is only shown outside Auto.
    if (homePanel && homePanel.autoBrightness) return;
    if (homePanel && level >= 0 && level <= 100) homePanelShow('brightness', level);
    else homePanelLog('backlight: ' + JSON.stringify(r).slice(0, 160));
  });
  homePanelReadVolume();
}

function homePanelReadVolume() {
  var writes = homePanelWriteCount.volume;
  var show = function (level) {
    if (writes !== homePanelWriteCount.volume) return;
    level = parseInt(level, 10);
    if (homePanel && level >= 0 && level <= 100) homePanelShow('volume', level);
  };
  homePanelSend('com.webos.service.audio', '/master/getVolume', {}, function (r) {
    if (r && r.volumeStatus) homePanelSoundOutput = r.volumeStatus.soundOutput || '';
    if (homePanelUsesSonos()) {
      homePanelPoll();
      homePanelSonosCall('GetVolume', '', function (xml) {
        var m = /<CurrentVolume>(\d+)</.exec(xml || '');
        if (homePanel) homePanel.volumeOffline = !m;
        if (m) show(m[1]);
      });
      homePanelSonosCall('GetMute', '', function (xml) {
        var m = /<CurrentMute>(\d)</.exec(xml || '');
        if (m && writes === homePanelWriteCount.volume && homePanel) homePanel.muted = m[1] === '1';
      });
    } else if (r && r.volumeStatus) {
      if (homePanel) homePanel.muted = r.volumeStatus.muteStatus === true || r.volumeStatus.muteStatus === 'true';
      show(r.volumeStatus.volume);
    }
  });
}

function homePanelUsesSonos() {
  return homePanelSonos !== '' && homePanelSoundOutput.indexOf('external_arc') === 0;
}

// One RenderingControl call to the Sonos (UPnP SOAP on port 1400); `done` gets the reply text,
// or nothing after 1.5 s.
function homePanelSonosCall(action, args, done) {
  homePanelSoap('MediaRenderer/RenderingControl', 'RenderingControl',
    action, '<InstanceID>0</InstanceID><Channel>Master</Channel>' + args, done);
}

function homePanelSoap(path, service, action, args, done) {
  var xhr = new XMLHttpRequest();
  var finished = false;
  var finish = function (text) {
    if (finished) return;
    finished = true;
    if (done) done(text);
  };
  xhr.onreadystatechange = function () {
    if (xhr.readyState === 4) finish(xhr.status === 200 ? xhr.responseText : '');
  };
  xhr.open('POST', 'http://' + homePanelSonos + ':1400/' + path + '/Control');
  xhr.setRequestHeader('Content-Type', 'text/xml; charset="utf-8"');
  xhr.setRequestHeader('SOAPACTION', '"urn:schemas-upnp-org:service:' + service + ':1#' + action + '"');
  xhr.send('<?xml version="1.0"?><s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" ' +
    's:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/"><s:Body><u:' + action +
    ' xmlns:u="urn:schemas-upnp-org:service:' + service + ':1">' + args + '</u:' + action + '></s:Body></s:Envelope>');
  genericTimer.set(1500, function () {
    if (!finished) xhr.abort();
    finish('');
  });
}

// Shows a level read from the TV (or the Sonos), then any step pressed before it was known.
function homePanelShow(name, level) {
  homePanel[name] = level;
  var pending = homePanelPending[name];
  homePanelPending[name] = 0;
  if (pending) homePanelSet(name, level + pending * homePanelStep(name));
}

// One arrow press: the Sonos goes 0-100 in fine steps, 5 would jump too far at listening levels.
function homePanelStep(name) {
  return name === 'volume' && homePanelUsesSonos() ? 2 : HOME_PANEL_STEP;
}

// The remote's volume keys reach the soundbar without passing the TV: while the panel is open,
// ask the Sonos every second so the slider follows them. Closed, nothing runs.
function homePanelPoll() {
  if (homePanelPolling || !homePanelUsesSonos()) return;
  homePanelPolling = true;
  var tick = function () {
    if (!homePanelIsOpen() || !homePanelUsesSonos()) {
      homePanelPolling = false;
      return;
    }
    homePanelReadVolume();
    genericTimer.set(1000, tick);
  };
  genericTimer.set(1000, tick);
}

// Restarts the auto-close countdown.
function homePanelTouch() {
  var mine = ++homePanelIdle;
  genericTimer.set(HOME_PANEL_IDLE_MS, function () {
    if (mine === homePanelIdle) homePanelClose();
  });
}

// Shows the new level at once and writes it within HOME_PANEL_WRITE_MS: a held arrow sends
// one write per tick at most, with the last value.
function homePanelSet(name, value) {
  value = Math.max(0, Math.min(100, Math.round(value)));
  if (!homePanel) return;
  homePanelWrites[name] = value;
  if (name === 'volume' && homePanel.muted) homePanelMute(false); // as on tvOS, moving the level unmutes
  if (name === 'brightness' && homePanelNight.on) {
    // With Night mode on, the light the owner picks becomes the night level (kept for next time).
    homePanelSchedule.level = value;
    homePanelSaveSchedule();
  } else if (name === 'brightness' && homePanel.autoBrightness) {
    // Moving the light by hand takes it back from the sensor, as in HOME.
    homePanel.autoBrightness = false;
    homePanelSend('com.webos.settingsservice', '/setSystemSettings', { category: 'picture', settings: { energySaving: 'off' } });
  }
  homePanel[name] = value;
  homePanelWriteCount[name]++;
  homePanelTouch();
  if (homePanelWriteQueued) return;
  homePanelWriteQueued = true;
  genericTimer.set(HOME_PANEL_WRITE_MS, homePanelFlush);
}

function homePanelFlush() {
  homePanelWriteQueued = false;
  var writes = homePanelWrites;
  homePanelWrites = {};
  if (typeof writes.brightness === 'number') {
    homePanelSend('com.webos.settingsservice', '/setSystemSettings', { category: 'picture', settings: { backlight: writes.brightness } });
  }
  if (typeof writes.volume === 'number') {
    if (homePanelUsesSonos()) homePanelSonosCall('SetVolume', '<DesiredVolume>' + writes.volume + '</DesiredVolume>');
    else homePanelSend('com.webos.service.audio', '/master/setVolume', { volume: writes.volume });
  }
}

function homePanelAction(name) {
  try {
    if (name === 'close') {
      homePanelClose();
    } else if (name === 'screenOff') {
      homePanelClose();
      // After the closing animation and any key still in flight.
      genericTimer.set(300, function () {
        homePanelSend('com.webos.service.tvpower', '/power/turnOffScreen', { standbyMode: 'active' });
      });
    } else if (name === 'settings') {
      homePanelClose();
      homePanelSend('com.webos.applicationManager', '/launch', { id: 'com.palm.app.settings' });
    } else if (name === 'night') {
      homePanelSetNight(!homePanelNight.on);
    } else if (name === 'modeNext' || name === 'modePrevious') {
      homePanelCycleMode(name === 'modeNext' ? 1 : -1);
    } else if (name === 'touch') {
      homePanelTouch();
    }
  } catch (error) {
    homePanelLog('action: ' + error);
  }
}

// Night button: "Desactivado", "Activado", or "Hasta las 07:00" when the schedule is driving it.
function homePanelNightDetail() {
  var t = HOME_PANEL_TEXT[homePanelLang];
  if (!homePanelNight.on) return t.off;
  return homePanelSchedule.enabled && homePanelInWindow ? t.until + homePanelSchedule.to : t.on;
}

function homePanelShowNight() {
  if (!homePanel) return;
  homePanel.night = homePanelNight.on;
  homePanel.nightDetail = homePanelNightDetail();
}

function homePanelSaveSchedule() {
  homePanelSend('com.webos.service.config', '/setConfigs', { configs: { 'com.webos.surfacemanager.homePanelSchedule': homePanelSchedule } });
}


// `bySchedule`: the schedule's own switch at an edge. A change by hand stays until the next edge,
// remembered in the saved state so a restart or a reinstall does not undo it.
function homePanelSetNight(on, bySchedule) {
  homePanelNight.on = on;
  homePanelNight.manual = !bySchedule;
  homePanelSaveNight();
  homePanelShowNight();
  homePanelNightSync();
}

function homePanelSaveNight() {
  homePanelSend('com.webos.service.config', '/setConfigs', { configs: { 'com.webos.surfacemanager.homePanelNight': homePanelNight } });
}

// Applies night values to the mode on screen (keeping the owner's), or gives a mode its own
// values back. Runs on toggle, on opening and every minute while there is something to do.
function homePanelNightSync() {
  var hasSaved = false;
  for (var k in homePanelNight.saved) hasSaved = true;
  if (homePanelSyncing || (!homePanelNight.on && !hasSaved)) return;
  homePanelSyncing = true;
  homePanelSend('com.webos.settingsservice', '/getSystemSettings', { category: 'picture', keys: HOME_PANEL_NIGHT_KEYS }, function (r) {
    homePanelSyncing = false;
    if (!r || !r.settings || !r.dimension) return;
    var d = r.dimension;
    var key = (d.input || '') + '|' + (d.dynamicRange || 'sdr') + '|' + (d.pictureMode || '');
    var saved = homePanelNight.saved[key];
    if (homePanelNight.on && !saved) {
      var mine = {};
      for (var i = 0; i < HOME_PANEL_NIGHT_KEYS.length; i++) mine[HOME_PANEL_NIGHT_KEYS[i]] = r.settings[HOME_PANEL_NIGHT_KEYS[i]];
      homePanelNight.saved[key] = mine;
      homePanelSaveNight(); // kept before anything changes: a restart must not lose them
      var level = Math.min(parseInt(mine.backlight, 10) || 100, homePanelSchedule.level);
      homePanelSend('com.webos.settingsservice', '/setSystemSettings', { category: 'picture', settings: {
        backlight: level, peakBrightness: 'off', dynamicContrast: 'off', eyeComfortMode: 'on' } }, function () {
        if (homePanel) homePanel.brightness = level;
      });
    } else if (homePanelNight.on && saved && parseInt(r.settings.backlight, 10) > homePanelSchedule.level) {
      // The night level went down since this mode was dimmed: follow it (never up, the owner
      // may have lowered it further by hand).
      homePanelSend('com.webos.settingsservice', '/setSystemSettings', { category: 'picture', settings: { backlight: homePanelSchedule.level } }, function () {
        if (homePanel) homePanel.brightness = homePanelSchedule.level;
      });
    } else if (!homePanelNight.on && saved) {
      var back = {};
      for (var j = 0; j < HOME_PANEL_NIGHT_KEYS.length; j++) {
        var name = HOME_PANEL_NIGHT_KEYS[j];
        if (saved[name] !== undefined && saved[name] !== null) back[name] = name === 'backlight' ? parseInt(saved[name], 10) : saved[name];
      }
      homePanelSend('com.webos.settingsservice', '/setSystemSettings', { category: 'picture', settings: back }, function (w) {
        if (!w || !w.returnValue) return; // keep them to try again next time
        delete homePanelNight.saved[key];
        homePanelSaveNight();
        if (homePanel && typeof back.backlight === 'number') homePanel.brightness = back.backlight;
      });
    }
  });
}

function homePanelMinutes(hhmm) {
  var p = String(hhmm).split(':');
  return (parseInt(p[0], 10) || 0) * 60 + (parseInt(p[1], 10) || 0);
}

// Every minute: the schedule turns night mode on or off when the time crosses its edges (a
// manual change stays until the next edge), and night values follow the mode on screen.
function homePanelTick(once) {
  try {
    if (homePanelSchedule.enabled) {
      var now = new Date();
      var m = now.getHours() * 60 + now.getMinutes();
      var from = homePanelMinutes(homePanelSchedule.from), to = homePanelMinutes(homePanelSchedule.to);
      var inside = from <= to ? m >= from && m < to : m >= from || m < to;
      if (inside !== homePanelInWindow) {
        // At an edge, the schedule switches. On the first check after loading, it only catches
        // up if nobody changed night mode by hand since the last edge.
        var edge = homePanelInWindow !== null;
        if ((edge || !homePanelNight.manual) && inside !== homePanelNight.on) homePanelSetNight(inside, true);
        else if (edge) homePanelNight.manual = false;
        homePanelInWindow = inside;
      }
    }
    homePanelNightSync();
    homePanelShowNight();
  } catch (error) {
    homePanelLog('tick: ' + error);
  }
  if (!once) genericTimer.set(60000, homePanelTick);
}

// Mutes or unmutes where the sound goes: the Sonos itself over ARC, else the TV.
function homePanelMute(on) {
  homePanel.muted = on;
  homePanelWriteCount.volume++; // a read in flight must not undo it on screen
  if (homePanelUsesSonos()) homePanelSonosCall('SetMute', '<DesiredMute>' + (on ? 1 : 0) + '</DesiredMute>');
  else homePanelSend('com.webos.service.audio', '/master/muteVolume', { mute: on });
}

// LG's "Energy saving: Auto" follows the room light (the C1 has a light sensor).
// With Auto on, the level the panel really shows: pqcontroller's driver value (measured on the C1:
// setting 45, real 23 then 21 in a dark room; 0 with energy saving "max"). Asked every 2 s while
// the panel is open in Auto, as the TV keeps adjusting.
var homePanelAutoPolling = false;
function homePanelPollAuto() {
  if (homePanelAutoPolling) return;
  homePanelAutoPolling = true;
  var tick = function () {
    if (!homePanelIsOpen() || !homePanel.autoBrightness) {
      homePanelAutoPolling = false;
      return;
    }
    homePanelSend('com.webos.service.pqcontroller', '/getInfo', { key: 'backlight' }, function (r) {
      var level = r ? parseInt(r.backlight, 10) : NaN;
      if (homePanel && homePanel.autoBrightness && level >= 0 && level <= 100) homePanel.brightness = level;
    });
    genericTimer.set(2000, tick);
  };
  tick();
}

// The level before Auto, given back when Auto goes off (LG's energy saving may move it).
var homePanelBeforeAuto = -1;
function homePanelToggleAuto() {
  if (homePanelNight.on) {
    // Auto and the night level would fight; say so for 1.5 s instead of doing nothing silently.
    homePanel.brightnessMessage = HOME_PANEL_TEXT[homePanelLang].nightBlocksAuto;
    genericTimer.set(1500, function () { if (homePanel) homePanel.brightnessMessage = ''; });
    return;
  }
  var on = !homePanel.autoBrightness;
  homePanel.autoBrightness = on;
  var settings = { energySaving: on ? 'auto' : 'off' };
  if (on) homePanelBeforeAuto = homePanel.brightness;
  else if (homePanelBeforeAuto >= 0) settings.backlight = homePanelBeforeAuto;
  homePanelSend('com.webos.settingsservice', '/setSystemSettings', { category: 'picture', settings: settings }, function () {
    if (on) {
      homePanelPollAuto();
    } else {
      if (homePanelBeforeAuto >= 0 && homePanel) homePanel.brightness = homePanelBeforeAuto;
      homePanelBeforeAuto = -1;
    }
  });
}

function homePanelIsOk(key) {
  return key === Qt.Key_Return || key === Qt.Key_Enter || key === WebOS.Key_webOS_LocalEnter;
}

function homePanelRowOf(f) {
  for (var r = 0; r < HOME_PANEL_ROWS.length; r++) if (HOME_PANEL_ROWS[r].indexOf(f) >= 0) return r;
  return 0;
}

function homePanelMove(key) {
  var f = homePanel.focusIndex;
  var row = homePanelRowOf(f);
  var cells = HOME_PANEL_ROWS[row];
  if (key === Qt.Key_Up || key === Qt.Key_Down) {
    var to = Math.max(0, Math.min(HOME_PANEL_ROWS.length - 1, row + (key === Qt.Key_Down ? 1 : -1)));
    var next = HOME_PANEL_ROWS[to];
    f = next[Math.min(next.length - 1, homePanelHalf[to] || 0)];
  } else if (cells.length > 1) {
    var at = Math.max(0, Math.min(cells.length - 1, cells.indexOf(f) + (key === Qt.Key_Right ? 1 : -1)));
    f = cells[at];
    homePanelHalf[row] = at;
  }
  homePanel.focusIndex = f;
}

// The handler named in the key filter list.
// oxlint-disable-next-line no-unused-vars
function homePanelKey(key, pressed, autoRepeat) {
  try {
    if (homePanelSeen.length >= 12) homePanelSeen.shift();
    homePanelSeen.push(key.toString(16) + (pressed ? 'd' : 'u') + (autoRepeat ? 'r' : '') + (homePanelIsOpen() ? '*' : ''));
    return homePanelHandle(key, pressed, autoRepeat);
  } catch (error) {
    homePanelLog('key ' + key + ': ' + error);
    // With the panel on screen the key was meant for it: never let it reach the app (an OK
    // would pause the video). Closed, LG and the app keep the key.
    return homePanelIsOpen() ? KeyPolicy.Accepted : KeyPolicy.NextPolicy;
  }
}

function homePanelHandle(key, pressed, autoRepeat) {
  if (key === WebOS.Key_webOS_Settings) {
    // Without a panel (it failed to load), Settings stays LG's.
    if (!homePanelEnsure()) return KeyPolicy.NextPolicy;
    if (pressed && !autoRepeat) {
      homePanelDown = true;
      homePanelLong = false;
      var press = ++homePanelPress;
      genericTimer.set(HOME_PANEL_LONG_MS, function () {
        if (press !== homePanelPress || !homePanelDown) return;
        homePanelLong = true;
        homePanelAction('settings');
      });
    } else if (!pressed) {
      homePanelDown = false;
      if (!homePanelLong) {
        if (homePanelIsOpen()) homePanelClose();
        else homePanelOpen();
      }
    }
    return KeyPolicy.Accepted;
  }

  if (!homePanelIsOpen()) return KeyPolicy.NextPolicy;

  // The Magic Remote reports switching between pointer and buttons as keys: CursorHide comes
  // with the first button press after pointing. They are not presses and must not close it.
  if (key === WebOS.Key_webOS_CursorShow || key === WebOS.Key_webOS_CursorHide) return KeyPolicy.NextPolicy;

  switch (key) {
    case Qt.Key_VolumeUp:
    case Qt.Key_VolumeDown:
    case Qt.Key_VolumeMute:
      // LG changes the volume and shows its own indicator; the slider follows.
      if (!pressed) genericTimer.set(250, homePanelReadVolume);
      homePanelTouch();
      return KeyPolicy.NextPolicy;
    case WebOS.Key_webOS_Back:
    case WebOS.Key_webOS_Exit:
      if (pressed) homePanelClose();
      return KeyPolicy.Accepted;
    case Qt.Key_Up:
    case Qt.Key_Down:
    case Qt.Key_Left:
    case Qt.Key_Right:
      if (pressed) {
        var slider = HOME_PANEL_SLIDERS[homePanel.focusIndex];
        if (homePanel.focusIndex === 4 && (key === Qt.Key_Left || key === Qt.Key_Right)) {
          homePanelCycleMode(key === Qt.Key_Right ? 1 : -1);
        } else if (slider && (key === Qt.Key_Left || key === Qt.Key_Right)) {
          if (slider === 'volume' && homePanel.volumeOffline) {
            homePanelReadVolume(); // the Sonos did not answer: ask again, change nothing
          } else if (homePanel[slider] < 0) {
            // Unknown level: never guess from 0. Keep the step for when the read answers.
            // Counted in presses: the step size depends on where the sound goes, known later.
            homePanelPending[slider] += key === Qt.Key_Right ? 1 : -1;
          } else {
            homePanelSet(slider, homePanel[slider] + (key === Qt.Key_Right ? 1 : -1) * homePanelStep(slider));
          }
        } else {
          homePanelMove(key);
        }
        homePanelTouch();
      }
      return KeyPolicy.Accepted;
  }
  if (homePanelIsOk(key)) {
    if (pressed && !autoRepeat) {
      var f = homePanel.focusIndex;
      if (f === 0) homePanelToggleAuto();
      else if (f === 1) homePanelSetNight(!homePanelNight.on);
      else if (f === 3) homePanelMute(!homePanel.muted);
      else if (f === 5) homePanelScreenOffArmed = true;
      else if (f === 6) homePanelAction('settings');
      // 4 (picture mode): Left/Right change it; OK does nothing until a list sheet exists.
      homePanelTouch();
    } else if (!pressed && homePanelScreenOffArmed) {
      // Turned off on the press, the release woke the panel again: act after it.
      homePanelScreenOffArmed = false;
      homePanelAction('screenOff');
    }
    return KeyPolicy.Accepted;
  }
  // Anything else (Home, channels, numbers…) closes the panel and goes on as usual.
  if (pressed) homePanelClose();
  return KeyPolicy.NextPolicy;
}

// Test channel (scripts/lg-panel.sh test): replays remote keys through the real handler and
// writes the panel state to com.webos.surfacemanager.homePanelState.
var HOME_PANEL_TEST_KEYS = {
  settings: 'Key_webOS_Settings', back: 'Key_webOS_Back', exit: 'Key_webOS_Exit',
  up: 'Key_Up', down: 'Key_Down', left: 'Key_Left', right: 'Key_Right', ok: 'Key_Return',
  volup: 'Key_VolumeUp', home: 'Key_webOS_Home',
  cursorhide: 'Key_webOS_CursorHide', cursorshow: 'Key_webOS_CursorShow'
};

function homePanelTest(cmd) {
  var keys = cmd.keys || [];
  var hold = cmd.hold || 0;
  for (var i = 0; i < keys.length; i++) {
    var name = HOME_PANEL_TEST_KEYS[keys[i]];
    var code = name && name.indexOf('Key_webOS_') === 0 ? WebOS[name] : Qt[name];
    if (code === undefined) continue;
    homePanelHandle(code, true, false);
    if (!hold) homePanelHandle(code, false, false);
  }
  if (hold) {
    // A long press: release after `hold` ms (only for the last key).
    var last = HOME_PANEL_TEST_KEYS[keys[keys.length - 1]];
    genericTimer.set(hold, function () {
      homePanelHandle(last.indexOf('Key_webOS_') === 0 ? WebOS[last] : Qt[last], false, false);
      homePanelReport(cmd.n);
    });
    return;
  }
  // Give the reads a moment before reporting.
  genericTimer.set(400, function () {
    homePanelReport(cmd.n);
  });
}

function homePanelReport(n) {
  var state = homePanel
    ? { n: n, shown: homePanel.shown, visible: homePanel.visible, focus: homePanel.focusIndex,
        brightness: homePanel.brightness, volume: homePanel.volume, lang: homePanelLang,
        auto: homePanel.autoBrightness, muted: homePanel.muted, offline: homePanel.volumeOffline, msg: homePanel.brightnessMessage, modeAt: [homePanel.modeIndex, homePanel.modeCount, homePanel.modePending], night: homePanelNight, schedule: homePanelSchedule, mode: homePanelMode, video: homePanel.videoBadges.join('|'), audio: homePanel.audioBadges.join('|') + ' / ' + homePanel.audioDevice, output: homePanelSoundOutput, sonos: homePanelSonos,
        cursor: compositorWindow.cursorVisible,
        size: [homePanel.width, homePanel.height, homePanel.opacity, homePanel.z], error: homePanelError,
        root: [homePanel.parent.width, homePanel.parent.height, homePanel.parent.children.length],
        seen: homePanelSeen }
    : { n: n, created: false, error: homePanelError, seen: homePanelSeen };
  homePanelSend('com.webos.service.config', '/setConfigs', { configs: { 'com.webos.surfacemanager.homePanelState': state } });
}




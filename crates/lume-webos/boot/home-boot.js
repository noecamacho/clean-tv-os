/* HOME early launch on a cold (snapshot) boot of the LG OLED55C1PSA. ES5 for LG's QML engine.
 * LG's compositor loads key filters ~16 s into a boot, long before the Homebrew init.d hook
 * (~30 s). This one opens HOME (app.home.tv) as soon as SAM takes normal apps (minimal-boot-done,
 * ~18 s), over the input bootd opened. Keys pass through unchanged.
 * Technique from Luma's home-filter.js (com.ecklf.luma, MIT); written anew for HOME.
 * Installed by scripts/lg-boot-filter.sh, registered after homePanelKey.
 */
var homeBootState = ''; // "" until bootd answers, then "booting" or "done"
var homeBootFront = '';
var homeBootLaunches = 0;
var homeBootScreen = false; // tvpower "Active": a Quick Start+ background boot has no screen
var homeBootMinimal = false; // minimal-boot-done: SAM launches normal apps

function homeBootParse(payload) {
  return typeof payload === 'string' ? JSON.parse(payload) : payload;
}

function homeBootOpen() {
  if (homeBootState !== 'booting' || !homeBootMinimal || !homeBootScreen || homeBootLaunches >= 2) return;
  if (homeBootFront === 'app.home.tv') return;
  // Only over nothing or an input: a power-on with an app key keeps its app.
  if (homeBootFront && !/^com\.webos\.app\.(livetv|hdmi\d|externalinput\..*|home)$/.test(homeBootFront))
    return;
  homeBootLaunches++;
  lunaCall.send('com.webos.applicationManager', '/launch', '{"id":"app.home.tv"}', function () {});
  // SAM can accept a launch right after minimal-boot-done without showing it: try once more.
  genericTimer.set(6000, homeBootOpen);
}

// Called by LG's compositor once the file is loaded.
function init() {
  try {
    lunaCall.subscribe('com.webos.bootManager', '/getBootStatus', '{}', function (payload) {
      try {
        var r = homeBootParse(payload);
        if (!r || !r.returnValue || !r.signals) return;
        // Only during a boot: reloading the filter while the TV is in use never takes the screen.
        if (homeBootState === '') homeBootState = r.signals['boot-done'] ? 'done' : 'booting';
        if (r.signals['boot-done']) homeBootState = 'done';
        if (r.signals['minimal-boot-done']) homeBootMinimal = true;
        homeBootOpen();
      } catch (error) {
        console.warn('home boot filter: ' + error);
      }
    }, true);
    lunaCall.subscribe('com.webos.service.tvpower', '/power/getPowerState', '{"subscribe":true}', function (payload) {
      try {
        var r = homeBootParse(payload);
        homeBootScreen = !!r && r.state === 'Active' && !r.processing;
        homeBootOpen();
      } catch (error) {
        console.warn('home boot filter: ' + error);
      }
    }, true);
    lunaCall.subscribe('com.webos.applicationManager', '/getForegroundAppInfo', '{}', function (payload) {
      try {
        var r = homeBootParse(payload);
        homeBootFront = (r && r.appId) || '';
        homeBootOpen();
      } catch (error) {
        console.warn('home boot filter: ' + error);
      }
    }, true);
  } catch (error) {
    console.warn('home boot filter: ' + error);
  }
}

// The handler named in the key filter chain; keys pass through.
function homeBootKey() {
  return KeyPolicy.NextPolicy;
}

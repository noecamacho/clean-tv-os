# APIs de TCL y Realtek en la 55Q5K

Mapa de las APIs Java del fabricante que existen en esta tele, hecho solo con análisis estático (`dexdump` de los jar extraídos por `adb pull`, manifiestos con `aapt2`, `service list`, `dumpsys package`). Firmware analizado: `TCL/G10_4K_GB_NF_32BIT/G10:14/UTT2.250416.001/AU02:user/release-keys`, Android 14.

**El mapa es estático; lo ejecutado está en «Comprobado en la tele».** Cada fila dice lo que hay en el bytecode, no que la llamada funcione. Aviso único que vale para todo el documento: aunque la clase cargue y el método no compruebe permisos, SELinux puede negar a `untrusted_app` el `find` del servicio binder que hay debajo, y eso solo se ve en la tele (`avc: denied` en logcat). La única ruta con prueba real es la que HOME ya usa: `TvBacklightManager` (véase `LumeBridge.tclBacklight` / `setTclBacklight`).

## Comprobado en la tele (2026-11-15, HOME 0.9.5, uid de app normal)

| Llamada | Resultado |
| --- | --- |
| `TvBacklightManager.getInstance().getBacklight()` | Funciona. Devuelve 0–100; coincide con «Brillo» del menú de imagen. |
| `setBacklight(EN_APPLY_CURRENT, n, EN_TCL_ACT_EXEC_SAVE)` | Funciona: `TclDisplayServer` lo ejecuta y la lectura siguiente devuelve `n` (probado 100 → 85 → 100). Es lo que usa el deslizador de Brillo del Centro de control. |
| `getLightSensorOnoff()` | Funciona; devuelve 0. |
| `setLightSensorOnoff(1, EN_TCL_ACT_EXEC_SAVE)` | Devuelve `true`, pero el servidor registra `on_off = 0` y la lectura sigue en 0: en la 55Q5K el sensor no se activa. HOME abre entonces el panel de imagen. |
| `Settings.System.SCREEN_BRIGHTNESS` | Se puede escribir con `WRITE_SETTINGS` y llega a la luz 0 del HAL de Android, pero **no cambia el panel**. Solo queda como respaldo para otras marcas. |

SELinux no bloqueó el servicio `tcl_tv_display` para HOME. El modo de imagen actual se ve en el registro como `cur_pic_mode_name_type = 10`; la tabla de valores sigue sin conocerse, así que HOME no lo cambia.

## Cómo usar esto desde HOME

### Tres clases de librería

| Nivel | Qué es | Librerías | Qué puede hacer una app normal |
| --- | --- | --- | --- |
| **A** | Shared library registrada (`pm list libraries`), fuera del BOOTCLASSPATH, sin sección `hiddenapi_class_data` en el dex | `com.tcl.tv.display`, `com.tcl.tvinput`, `com.tcl.hdmi`, `com.tcl.hbbtv`, `com.tcl.ginga.porting`, `com.tcl.gingad`, `com.tcl.tbrowser`, `tromlib`, `rtk-framework`, `tcl.factory.hal`, `tcl.factory.hal.ext` | `<uses-library android:required="false">` + reflexión normal |
| **B** | Jar del BOOTCLASSPATH. Todos los miembros propios están marcados `blocked` en `hiddenapi_class_data` (comprobado decodificando la sección: `tcl-framework` 4304 miembros `blocked`, `tcl.os.TDeviceInfo` 46/46, `tcl.media.TAudioManager` 103/103, `android.media.TclAudioManager` 109/109) | `/apex/com.tcl.tcore/javalib/tcl-framework.jar`, clases TCL dentro de `/system/framework/framework.jar`, `/apex/com.tcl.motor/javalib/tcl-motor-framework.jar`, `/apex/com.tcl.hera/javalib/*.jar`, `/system_ext/framework/rtk-mediaplayer.jar`, `/system/framework/tcl.webkit.jar` | La clase se ve, pero con `targetSdk 36` la reflexión sobre miembros `blocked` falla (`NoSuchMethodException`) salvo que la política de API ocultas del dispositivo esté relajada. **La premisa "lo que está en BOOTCLASSPATH se llama por reflexión" no se sostiene aquí.** |
| **C** | Jar solo de `system_server` (SYSTEMSERVERCLASSPATH) | `/apex/com.tcl.tcore/javalib/tcl-service.jar`, `/apex/com.tcl.apex.tvinput/javalib/com.tcl.hdmi-service.jar`, `/apex/com.tcl.motor/javalib/tcl-motor-service.jar`, `rtk-framework-window-service` (registrada como librería, pero su contenido es `com.android.server.wm.*`) | No se puede llamar. Aquí viven las comprobaciones de permisos de los servicios. |

Puente entre A y B: `tromlib` (`/system_ext/framework/tromlib.jar`, registrada, sin flags) envuelve unas pocas clases de nivel B. En AOSP el código cargado desde `/system_ext/framework` cuenta como dominio de plataforma para las API ocultas, así que sus llamadas internas a clases `blocked` deberían pasar (comportamiento de AOSP, no verificado en esta tele). Es la única vía limpia hacia `TDeviceInfo` y `TclAudioManager`.

`rtk-mediaplayer` y `tcl.webkit` tienen XML de librería, pero están en BOOTCLASSPATH con flags `blocked`: cuentan como nivel B. El XML de `rtk-mediaplayer` apunta a `/product/framework/rtk-mediaplayer.jar`, que no existe; el jar real está en `/system_ext/framework/`.

### Patrón (ejemplo real: brillo del panel)

Manifiesto, dentro de `<application>` (ya está para `com.tcl.tv.display`; cada librería nueva necesita su línea):

```xml
<uses-library android:name="com.tcl.tv.display" android:required="false" />
```

Llamada por reflexión, siempre en `try/catch (Throwable)` porque en otra marca la clase no existe:

```java
Class<?> manager = Class.forName("com.tcl.tv.pq.TvBacklightManager");
Class apply = Class.forName("com.tcl.tv.pq.EnApplyType");
Class act = Class.forName("com.tcl.tv.pq.EnTclActType");
Object instance = manager.getMethod("getInstance").invoke(null);
int now = (Integer) manager.getMethod("getBacklight").invoke(instance);
Object ok = manager.getMethod("setBacklight", apply, int.class, act).invoke(instance,
        Enum.valueOf(apply, "EN_APPLY_CURRENT"), 60, Enum.valueOf(act, "EN_TCL_ACT_EXEC_SAVE"));
```

Qué ocurre por dentro (leído en el bytecode):

1. `TvBacklightManager.getInstance()` llama a `PqSDKManager.getInstance(null)`.
2. `PqSDKManager` prueba `Class.forName("com.tcl.tvmanager.TTvPictureManager")`. Esa clase no está en ningún jar extraído, así que cae en la rama `tvsdk` (`com.tcl.tv.pq.tvsdk.TvSdkManagerImpl`).
3. `tvsdk.TvBacklightManagerImpl.<init>` hace `Class.forName("android.os.ServiceManager").getMethod("getService", String).invoke(null, "tcl_tv_display")` y `ITvDisplayManagerService.Stub.asInterface(...)`.
4. `setBacklight(EnApplyType,int,EnTclActType)` pasa los `ordinal()` de los enums a `ITvDisplayManagerService.setBacklight(int,int,int)`.

Los managers no comprueban nada del llamador y tragan `RemoteException` (devuelven `-1` o `false`). El servicio `tcl_tv_display` es un binario nativo (`/system_ext/bin/tcl_tv_display_service`, `user root`, `seclabel u:r:tcl_display_service:s0`) que adb no deja leer: lo que comprueba por dentro no se conoce. Toda la sección "Imagen" depende de ese mismo servicio, así que lo que valga para el brillo es el mejor indicio para el resto.

Enums compartidos (constantes en orden alfabético del volcado; el `ordinal()` real no se ha leído, usar siempre `Enum.valueOf`):

- `com.tcl.tv.pq.EnApplyType`: `EN_APPLY_ALL_PICMODE`, `EN_APPLY_CURRENT`, `EN_APPLY_MAX`
- `com.tcl.tv.pq.EnTclActType`: `EN_TCL_ACT_EXEC`, `EN_TCL_ACT_EXEC_SAVE`, `EN_TCL_ACT_SAVE`, `EN_TCL_ACT_MAX`
- `com.tcl.tv.pq.EnPicModeFieldType`: `EN_PICTURE_MODE_FIELD_GLOBAL`, `_GLOBAL_OFF`, `_NORMAL`, `_SUB`, `_MAX`
- `com.tcl.tv.pq.EnPqMemcType`: `EN_PQ_MEMC_OFF`, `_LOW`, `_MID`, `_HIGH`, `_24PFILM`, `_CUSTOME`, `_MAX`
- `com.tcl.tv.pq.EnColorTmpType`: `EN_COLOR_TMP_COOL`, `EN_COLOR_TMP_STANDARD`, `EN_COLOR_TMPE_WARM` (sic), `EN_COLOR_TMP_PERSONAL`, `EN_COLOR_TMP_CUSTOM_1`, `EN_PQ_COLOR_TMP_STANDARD_TO_COOL1..4`, `EN_PQ_COLOR_TMP_STANDARD_TO_WARM1..4`, `EN_PQ_COLOR_TMP_NUM`
- `com.tcl.tv.pq.EnResetFunctionType`: `EN_RESET_PICMODE`, `EN_RESET_ALL_DATA`, `EN_RESET_DISPLAY_SETTING`, `EN_RESET_MOTION_CLARITY`, `EN_RESET_COLOR_SPACE`, `EN_RESET_CUST_GAMMA`, `EN_RESET_CUST_WHITE_BALANCE` (cada uno con variante `_APPLY_ALL`), `EN_RESET_MAX`
- `com.tcl.tv.pq.TvPqManager$EnInputSource`: `EN_TCL_NONE`, `EN_TCL_ATV`, `EN_TCL_DTV`, `EN_TCL_AV1`, `EN_TCL_AV2`, `EN_TCL_HDMI1..4`, `EN_TCL_DV`, `EN_TCL_DV2..4`, `EN_TCL_YPBPR`, `EN_TCL_YPBPR2..3`, `EN_TCL_VGA`, `EN_TCL_STORAGE`, `EN_TCL_STORAGE2`, `EN_TCL_OSD`, `EN_TCL_KTV`, `EN_TCL_MAX`
- `com.tcl.tv.display.TvDisplayManager$EnActionControl`: `EN_ACT_EXEC`, `EN_ACT_EXEC_SAVE`, `EN_ACT_SAVE`, `EN_ACT_MAX`
- `com.tcl.tv.display.TvDisplayManager$EnMuteType`: `EN_MUTE_NORMAL`, `EN_MUTE_AVOUT`, `EN_MUTE_STILL_IMG`, `EN_MUTE_TEST_PATTERN`, `EN_MUTE_AUTO_GAMMA`, `EN_MUTE_MAX`
- `com.tcl.tv.display.TvDisplayManager$EnSignalInfoType`: `EN_SIGNAL_INFO_ALLM`, `EN_SIGNAL_INFO_DYNAMIC_RANGE`, `EN_SIGNAL_INFO_VRR`, `EN_SIGNAL_INFO_MAX`
- `com.tcl.tv.display.TvDisplayManager$EnHdrType`: `EN_HDR_TYPE_SDR`, `_HDR10`, `_HDR10_PLUS`, `_HLG`, `_HLG18`, `_DOLBY`, `_PRIMIUM`, `_UNKNOW`

Leyenda de las tablas: **Cargable** = nivel A, B o C de arriba. **Comprobaciones vistas** = lo que se leyó en el cuerpo del método y, cuando el servidor es Java, en su implementación; "ninguna" significa que no aparece ni cadena de permiso ni `getCallingUid`, no que el servicio acepte la llamada. *(inf.)* marca una descripción deducida solo del nombre.

## Imagen

Todo en `com.tcl.tv.display` (115 clases). Managers: `TvBacklightManager`, `TvPqManager` (132 métodos públicos), `TvPanelManager`, `TvDisplayManager`, `TvDDCManager`, `TvPqDebugManager`; todos con `static getInstance()` sin argumentos. En `TvPqManager` casi todos los métodos llevan un primer `int` cuyo significado no se ha podido leer (sin nombres de parámetros en el dex; *(inf.)* id de ventana o de fuente, véase `TvDisplayManager.getWindowID(int)`).

| clase.método(firma) | Qué hace | Librería | Cargable | Comprobaciones vistas |
| --- | --- | --- | --- | --- |
| `TvBacklightManager.getBacklight() : int` | Brillo del panel. En uso por HOME | `com.tcl.tv.display` | A | Ninguna en la librería; servicio nativo `tcl_tv_display` no legible |
| `TvBacklightManager.setBacklight(EnApplyType,int,EnTclActType) : boolean` | Fija el brillo del panel. En uso por HOME | ídem | A | ídem |
| `TvBacklightManager.getLightSensorOnoff() : int` / `setLightSensorOnoff(int,EnTclActType) : boolean` | Sensor de luz ambiente *(inf.)* | ídem | A | ídem |
| `TvBacklightManager.getDBCLevel() : int` / `setDBCLevel(EnApplyType,int,EnTclActType) : boolean` | Control dinámico de retroiluminación *(inf.)* | ídem | A | ídem |
| `TvBacklightManager.getMixDimming() : int` / `setMixDimming(EnApplyType,int,EnTclActType) : boolean` | Atenuación mixta *(inf.)* | ídem | A | ídem |
| `TvPqManager.getPictureMode() : int` | Modo de imagen actual; llama a `ITvDisplayManagerService.getPictureMode()` | ídem | A | ídem |
| `TvPqManager.setPictureMode(int,int,EnPicModeFieldType,EnTclActType) : boolean` | Cambia el modo de imagen; llama a `setPictureMode(int,int,int,int)` | ídem | A | ídem |
| `TvPqManager.setPictureMode(int,EnApplyType,int,EnPicModeFieldType,EnTclActType) : boolean` | Variante; llama a `setPictureModeByType(int,int,int,int,int)` | ídem | A | ídem |
| `TvPqManager.getPictureType(int) : int` / `setPictureType(int,EnApplyType,int,EnTclActType) : boolean` | Tipo de imagen *(inf.)* | ídem | A | ídem |
| `TvPqManager.getBrightness(int) : int` / `setBrightness(int,EnApplyType,int,EnTclActType) : boolean` | Brillo de vídeo (no el del panel) | ídem | A | ídem |
| `getContrast(int)` / `setContrast(int,EnApplyType,int,EnTclActType)`; igual forma para `Saturation`, `Sharpness`, `Hue` | Ajustes básicos | ídem | A | ídem |
| `TvPqManager.getColorTemperature(int) : int` / `setColorTemperature(int,EnApplyType,EnColorTmpType,EnTclActType) : boolean` | Temperatura de color | ídem | A | ídem |
| `TvPqManager.getLowBlueRay(int) : int` / `setLowBlueRay(int,int,EnTclActType) : boolean` | Luz azul baja | ídem | A | ídem |
| `TvPqManager.getMemcConfig(int) : int` / `setMemcConfig(int,EnApplyType,EnPqMemcType,EnTclActType) : boolean` | MEMC por nivel | ídem | A | ídem |
| `getMemcBlur(int)` / `setMemcBlur(int,EnApplyType,int,EnTclActType)`; igual para `MemcJudder`, `MotionClarity`, `LedMotionClarity` | MEMC fino *(inf.)* | ídem | A | ídem |
| `TvPqManager.getGameModeConfig(EnInputSource) : int` / `setGameModeConfig(EnInputSource,int,EnTclActType) : boolean` | Modo juego por entrada | ídem | A | ídem |
| `TvPqManager.getGamePicutreModeType(int) : int` (sic) / `setGamePictureModeType(int,int,EnTclActType) : boolean` | Modo de imagen dentro de juego *(inf.)* | ídem | A | ídem |
| `getALLMMode(int)` / `setALLMMode(int,int,EnTclActType)`; `getVRRMode(int)` / `setVRRMode(int,int,EnTclActType)`; `getVrrRefreshRate() : float`; `getLowlantency(int)` / `setLowlantency(int,EnApplyType,int,EnTclActType)` | ALLM, VRR, baja latencia | ídem | A | ídem |
| `getDynamicToneMapping(int)` / `setDynamicToneMapping(int,EnApplyType,int,EnTclActType)`; `getHDRToneMappingAdjust(int)` / `set…`; `getSdr2Hdr(int)` / `setSdr2Hdr(…)`; `getVideoEOTFMode(int)` / `setVideoEOTFMode(int,EnApplyType,EnVideoEotfMode,EnTclActType)`; `getDolbyGameStatus(int)` | HDR | ídem | A | ídem |
| `getDynamicContrast(int)`, `getNR(int)`, `getMpegNR(int)`, `getDeContour(int)`, `getBlackStretch(int)`, `getColorEnhance(int)`, `getDarkEnhance(int)`, `getAipqEnable(int)`, `getAisrEnable(int)`, `get24p(int)`, `getDLGMode(int)`, `getDDMMode(int)`, `getRgbSensor(int)`, `getGammaLevel()` y su `set…` (forma `(int,EnApplyType,int,EnTclActType)` salvo `DLG`/`DDM`: `(int,int,EnTclActType)` y `setGammaLevel(EnApplyType,int,EnTclActType)`) | Mejoras varias | ídem | A | ídem |
| `TvPqManager.getTvVideoInfo(int) : TvVideoInfo` | Información de la señal de vídeo (Parcelable, campos sin nombre legible) | ídem | A | ídem |
| `TvPqManager.getFunctionIsSupport(TvPqManager$EnFunctionSupportType) : boolean` | Capacidades del panel | ídem | A | ídem |
| `TvPqManager.getDefaultData(int,int,TvPqFuncType) : int` | Valor por defecto de una función | ídem | A | ídem |
| `TvPqManager.resetPQFunction(EnResetFunctionType) : boolean` | Restablece ajustes de imagen | ídem | A | ídem |
| `TvPqManager.addCallback(TvPqCallback) : boolean` / `removeCallback(TvPqCallback)`; `TvPqCallback.onEvent(int,int,TvPqCallbackParameter) : boolean` | Avisos de cambio (`EN_UPDATE_PQ_PARAM_CHANGE`, `EN_UPDATE_PQ_PARAM_ERROR`) | ídem | A | ídem |
| `TvPqManager.getOption(int,int,String) : String` / `setOption(int,int,String)` / `setOptionStream(int,int,int,byte[])` | Canal genérico sin documentar | ídem | A | ídem. No tocar a ciegas |
| `TvPqManager.setBacklightConstant(boolean)`, `setMuteConstant(boolean)`, `setStrOnoff(boolean)`, `setCountry(String)`, `setHotLoad(int,int)`, `setFacPMode(int)`, `setFacWarmMode(int)`, `setGammaData(…)`, `set3DlutRGB(…)`, `reinitGamma()`, `setDolbyCfgPath(int,String)`, balances de blanco y curvas | Calibración y fábrica | ídem | A | ídem. **No para apps** |
| `TvPanelManager.getLocalDimming() : int` / `setLocalDimming(EnApplyType,int,EnTclActType) : boolean`; `getMicroDimming()` / `setMicroDimming(…)`; `getPanelFrameRate() : int`; `getLocalDimmingRow()`, `getLocalDimmingCol()`, `getLocalDimmingData() : int[]`; `getViewAngle()` / `setViewAngle(int)` | Panel | ídem | A | ídem |
| `TvDisplayManager.getAspectRatio(int) : EnAspectRatio` / `setAspectRatio(int,EnAspectRatio,EnActionControl) : boolean` | Relación de aspecto | ídem | A | ídem |
| `TvDisplayManager.getTvInputVideoInfo(int) : TvInputVideoInfo`; `getSignalInfo(EnSignalInfoType) : int` | Señal de la entrada (ALLM, rango dinámico, VRR) | ídem | A | ídem |
| `TvDisplayManager.getWindowMute(int,EnMuteType) : boolean` / `setWindowMute(int,EnMuteType,EnMuteLevel,boolean) : boolean` | Silencia (pone en negro) la ventana de vídeo *(inf.)*; no es apagar el panel | ídem | A | ídem |
| `TvDisplayManager.setCallback(TvDisplayCB) : boolean` / `removeDisplayCallback(TvDisplayCB)`; `TvDisplayCB.notify(int,int,int,int,String) : boolean` | Eventos de pantalla (`TvDisplayCallbackEventType`: `EN_EVENT_PANEL_ON_OFF`, `EN_EVENT_DYNAMIC_RANGE_CHANGE`, `EN_EVENT_ALLM_CHANGE`, `EN_EVENT_VSYNC_CHANGE`, `EN_EVENT_FORMATE_CHANGE`, …) | ídem | A | ídem |
| `TvDisplayManager.setDisplayWindow(int,WindowRect)`, `getWindowRect(int)`, `setOverScanValue(…)`, `setVgaParameter(…)`, `autoPcAdjust(…)`, `setWhiteBalancePattern(int,boolean,int,int,int)`, `getMethod(int,String) : Bundle` / `setMethod(int,String,Bundle)` | Geometría de la ventana de vídeo y patrones | ídem | A | ídem. No para un launcher |
| `TvDDCManager.*` (`getHdrMode`, `setHdrMode(int)`, `getEotfPeak`, `importCalFile`, `exportCalFile`, …), `TvPqDebugManager.*` (`dumpThalParam(int)`, `setBypassStatus(int,int)`) | Calibración con sonda y depuración | ídem | A | ídem. **No para apps** |
| `com.realtek.tv.PQ.getBacklight() : int` / `setBacklight(int) : int`; `getPictureMode()` / `setPictureMode(int)`; `getBrightness()` / `setBrightness(int)`; `getLowBlueLevel()` / `setLowBlueLevel(int)`; `enableGameMode(boolean)`; `enableBacklight(boolean)` (284 métodos en total) | Capa Realtek equivalente | `rtk-framework` | A | Va a `vendor.rtk.tv.IAidlRtkPQ` (HAL `vendor.rtk.tv.IAidlRtkTvService/default`). El cuerpo de `setBacklight` contiene `System.exit(int)` (rama de error *(inf.)*): **puede matar el proceso de HOME**. No usar |

Los valores `int` de modo de imagen (estándar, cine, juego…) no son un enum en la librería: hay que leerlos con `getPictureMode()` en la tele.

## Sonido

No hay ninguna librería de nivel A con ajustes de sonido. Todo cuelga de `tcl.media.TAudioManager` (`tcl-framework`) → `android.media.TclAudioManager` (`framework.jar`), que obtiene su binder con `IAudioService.getTclInnerService()` sobre el servicio `audio`. Implementación: `AudioService$TclInnerAudioService` → `com.android.server.audio.TclAudioServiceEx` / `TclSoundEffect` / `TclSoundSetting` (`tcl-service.jar`).

| clase.método(firma) | Qué hace | Librería | Cargable | Comprobaciones vistas |
| --- | --- | --- | --- | --- |
| `TAudioManager.getAudioOption(int) : int` | Lee una opción de `tcl.media.TAudioSystemOptionType`, por ejemplo `TCL_AUDIO_PARAM_TYPE_SOUNDMODE = 12289` (modo de sonido) | `tcl-framework` | B | `TclSoundEffect.getAudioOption`: `"android.permission.MODIFY_AUDIO_ROUTING"` con `checkCallingOrSelfPermission` (`signature\|privileged\|role`) |
| `TAudioManager.setAudioOption(int,int) : void` | Escribe una opción. Modos: `TCL_AUDIO_SYSTEM_SOUND_MODE_STANARD=0` (sic), `MOVIE=1`, `MUSIC=2`, `NEWS=3`, `GAME=4`, `SPORTS=5`, `USER=6`, `SMART=7`, `LIVE=8`, `NIGHT=14`. Otras claves: `_BASS=12291`, `_DIALOG=12292`, `_TREBLE=12293`, `_NIGHTMODE=12294`, `_SURROUND=12290`, `_ONOFF=12288`, `TCL_SOUND_SCENE=12296`, `TCL_ATMOS_ENABLE=12295` | ídem | B | `TclAudioServiceEx.setAudioOption` hace `Binder.clearCallingIdentity()` **antes** de delegar en `TclSoundEffect.setAudioOption`, cuyo `checkCallingOrSelfPermission(MODIFY_AUDIO_ROUTING)` se evalúa entonces como `system_server`. Estáticamente la escritura queda sin comprobación efectiva y la lectura no. El helper `setAudioOption_enforcePermission` existe en el `Stub` pero no se invoca desde `TclInnerAudioService` |
| `TAudioManager.getAudioBalance() : int` / `setAudioBalance(int) : int`; `setAudioBassLevel(double)`, `setAudioTrebleLevel(double)`; `getAudioEqualizerDb(int)` / `setAudioEqualizerDb(int,int)`; `getGraphicEqBands() : int[]`; `resetEffectSettings()` | Balance, graves, agudos, ecualizador | ídem | B | `MODIFY_AUDIO_ROUTING` en `TclSoundEffect` |
| `TAudioManager.getSpdifMode() : int` / `setSpdifMode(int)`; `getSpdifDelay()` / `setSpdifDelay(int)` | Salida digital. `tcl.media.TSpdifType`: `TCL_OFF=0`, `TCL_PCM=1`, `TCL_DDP=2`, `TCL_AUTO=3`, `TCL_PASSTHROUGH=4`, `TCL_DD=5`, `TCL_MAT=6` | ídem | B | `MODIFY_AUDIO_ROUTING` en `TclSoundSetting` |
| `TAudioManager.getTAudioStreamInfo() : TAudioStreamInfo`; `getTDtsStreamInfo()`; `getTAudioCapability(int)`; `isAudioFunctionSupport(int) : boolean`; `isExtSpeakerOn() : boolean`; `getDSPDriverVersion() : String` | Formato de audio en curso y capacidades | ídem | B | `MODIFY_AUDIO_ROUTING` en `TclSoundSetting` |
| `TAudioManager.getStreamDeviceVolume(int,int) : int` / `setStreamDeviceVolume(int,int,int,int,Context) : void` | Volumen por dispositivo; el `set` acaba en `AudioService.setStreamVolume(int,int,int,String)` | ídem | B | Ninguna propia. Equivale a `AudioManager.setStreamVolume`, que HOME ya usa |
| `TAudioManager.isVolumeAdjustmentDisabled() : boolean` / `setVolumeAdjustmentDisabled(boolean)` | Bloqueo del volumen | ídem | B | Lectura: ninguna. Escritura: `MODIFY_AUDIO_ROUTING` |
| `TAudioManager.getTExternalDeviceInfo(int) : TExternalDeviceInfo`; `getExtDeviceConnectionState(int) : int` | Dispositivo de audio externo (barra, ARC) | ídem | B | `getTExternalDeviceInfo`: ninguna (lee `AudioSystem.getParameters`). `getExtDeviceConnectionState`: sin comprobación en `TclAudioServiceEx` |
| `android.media.TclAudioManagerEx.getInstance()`; `isVolumeAdjustmentDisabled() : boolean`; `getTExternalDeviceInfo(int) : TExternalDeviceInfoEx` | Envoltorio de lo anterior | `tromlib` | A | Las del servidor de la fila de arriba (ninguna vista) |
| `android.media.TclAudioManager.setDevMode(int)`; `static switchDevMode2Device(int) : int`; `static toSelectDeviceStr(int) : String` | Dispositivo de salida. Nombres vistos: `TCL_DEVICE_OUT_TYPE_AUTO`, `_SPEAKER`, `_BLUETOOTH_A2DP`, `_SPEAKER_AND_BLUETOOTH_A2DP`, `_WIRED_HEADPHONE`, `_SPEAKER_AND_WIRED_HEADPHONE`, `_HDMI_ARC`, `_LINE`, `_AVOUT`, `_USB`, `_WIFI`, `_SPEAKER_AND_WIFI` | `framework.jar` | B | `TclAudioServiceEx.setDevMode` delega en `AudioService.setDevMode`, cuyo cuerpo no se localizó |
| `TAudioManager.addOnAudioFormatChangedListener(OnAudioFormatChangedListener)`, `addOnSoundBarEventChangedListener(OnSoundbarEventChangedListener)`, `addOnWirelessSpeakerStateListener(…)` y sus `remove…` | Avisos | `tcl-framework` | B | Ninguna vista |
| `android.hardware.hdmi.TclHdmiTvClient.isSupportEarc() : boolean`, `getEarcStatus() : int` (`EN_EARC_STATUS_ARC=0`, `_EARC=1`, `_HANDSHAKE=2`), `setEqMode(int)`, `getTrdDeviceEqMode()`, `setNightMode(int)`, `setBassEnhancer(int)`, `setDialogEnhancer(int)`, `resetSoundSettings()`, `enableSystemCecControl(boolean)` (39 métodos) | eARC/ARC y barra de sonido por CEC | `tcl-framework` | B | Servicio `hdmi_control`. En `TclHdmiControlService$TclBinderService` no se vio comprobación en `getEarcStatus`, `isSupportEarc`, `setEqMode`, `enableSystemCecControl`; `initBinderCall`/`enforceAccessPermission` solo aparecen en `portSelect` y `sendVolumeKeyEvent` |
| `com.realtek.tv.Tv.setAudioVolume(int)`, `setAudioVolumeMute(boolean)`, `getAudioMute(int)`, `isEARCSupport()`, `getEARCEnable()` / `setEARCEnable(boolean)`, `getHDMIArcPort()`; `com.realtek.tv.AQ.*` (88 métodos) | Capa Realtek | `rtk-framework` | A | HAL `vendor.rtk.tv.IAidlRtkTvService/default` |

Features que el firmware declara ausentes en `tcl_features.xml`: `com.tcl.software.hdmi_earc`, `hdmi_cec_link_earc`, `hdmi_cec_control_earc`. Presentes: `audio_arc`, `audio_volume`, `audio_devices`, `audio_effect`.

## Entradas y fuentes

`com.tcl.tvinput` (521 clases, la mayoría `com.tcl.dtv.*` de sintonización) y `com.tcl.hdmi` (594; repite `com.tcl.tvinput.*` y añade `com.tcl.hdmi.*` y `com.tcl.dfx.*`). El binder sale de `ServiceManager.getService("tcl_tvinput")` (`com.tcl.dtv.ITService`) y, como respaldo, de `vendor.tcl.tvinput.extend.ITvExtendService/default`, con `getService(String)` para cada submódulo.

| clase.método(firma) | Qué hace | Librería | Cargable | Comprobaciones vistas |
| --- | --- | --- | --- | --- |
| `com.tcl.tvinput.TTvInputMgr.getInstance(Context) : TTvInputMgr` | Punto de entrada | `com.tcl.tvinput` | A | Ninguna en la librería |
| `TTvInputMgr.getTvinputList() : HashMap` | Lista de entradas *(inf.)* | ídem | A | ídem |
| `TTvInputMgr.getCurrentInputID() : int` / `getLastInputID() : int` | Entrada actual. Lee `Settings.Secure` `"tcl_sys_tv_input_current_device_id"`, sin binder | ídem | A | Ninguna |
| `TTvInputMgr.getTvInputID(int) : String`; `getDeviceIdByTvInputId(String) : int`; `getDeviceIdByPortId(int)`; `getPortIdByDeviceId(int)`; `static getInputSourceType(String) : int` | Traducción entre id TIF, id de dispositivo y puerto. Constantes `SOURCE_TYPE_ATV=1`, `_DTV=2`, `_AV1=3`, `_HDMI1=15`, `_HDMI2=16`, `_HDMI3=17`, `_HDMI4=18`, `_MEDIACODEC=21` | ídem | A | ídem |
| `TTvInputMgr.isSrcInsert(int) : boolean`; `getTvInputSignalStatus(int) : boolean`; `getPowerStatus(int) : boolean` | Cable conectado y señal presente | ídem | A | Binder `ITTvInput` |
| `TTvInputMgr.getSignalInfo(int,int) : int` | Dato de señal por `SIGNAL_TYPE_HDR=0`, `_HRES=1`, `_VRES=2`, `_VSYNC=3`, `_ALLM=4`, `_HDMI_DEVICE=5`, `_HDMI_MODE=6`, `_COLOR_MATRIX=7`, `_PIXEL_FORMAT=8`, `_BIT_DEPTH=9`, `_DOLBY_GAME=11`, `_FRAME_STATE=12`. Valores `HDR_TYPE_SDR=0`, `_HDR10=1`, `_HDR10_PLUS=2`, `_HLG=3`, `_DOLBY=5` | ídem | A | ídem |
| `TTvInputMgr.getHdmiDeviceInfo() : Bundle`; `getHdmiDeviceInfoByDeviceId(int)`; `getVideoFormatInfo() : Bundle`; `getVideoFormatInfoByDeviceId(int)`; `getHdmiSupportFeature(int) : Bundle` | Dispositivo HDMI y formato (claves del `Bundle` sin leer) | ídem | A | ídem |
| `TTvInputMgr.getHdmiEdidVer(int)` / `setHdmiEdidVer(int,int)`; `getHdmi144VrrConfig(int)` / `setHdmi144VrrConfig(int,boolean)`; `setHdmiVrrConfig(int,int)`; `isQmsMode(int)` / `setQmsMode(int,boolean)` | Versión EDID y VRR por puerto (`EDID_VERSION_AUTO=0`, `_1_4=1`, `_2_0=2`, `_2_1=3`) | ídem | A | ídem. Cambiar EDID reinicia el enlace HDMI: no para un Centro de control |
| `com.tcl.tvinput.TEventManager.addEventCallback(int,TEventManager$EventCallback) : boolean` / `removeEventCallback(…)` (estáticos) | Eventos con `TVINPUT_MODULE=11`: `EVENT_SOURCE_PLUGIN_STATUS=1`, `EVENT_SOURCE_CHANGED_START=2`, `_END=3`, `EVENT_SIGNAL_LOCK_STATUS=4`, `EVENT_SIGNAL_FORMAT_CHANGED=5` | ídem | A | ídem |
| `com.tcl.tvinput.TTvSystem.getInstance(Context)`; `getProperty(String,String)` / `setProperty(String,String)`; `systemAuthorized(String) : boolean`; `isScreenCaptureAllowed()` | Propiedades del subsistema de TV | ídem | A | Sin leer |
| `com.tcl.impl.tvsdk.TvHdmiController.getInstance()`; `getHdmiFeatureSupport(int) : Bundle`; `getOption(int,int) : byte[]` / `setOption(int,int,int,byte[])`; `addCallBack(TvHdmiController$TvHdmiCallback)` | HDMI de bajo nivel (`OPTION_EARC_IS_SUPPORT=0`, `OPTION_EARC_STATUS=1`, `HDMI_SUPPORT_ARC=1`, `_EARC=2`, `_ALLM=8`, `_VRR=16`) | `com.tcl.tvinput` / `com.tcl.hdmi` | A | ídem |
| `com.tcl.hdmi.THdmiEdidManager.*`, `com.tcl.dfx.THdmiDfxManager.*` (diagnóstico y reparación HDMI) | EDID y diagnóstico | `com.tcl.hdmi` | A | Sin leer. **No para apps** |
| `com.realtek.tv.Tv.setInputSource(Context,String) : void` | Construye un `Intent` `android.intent.action.VIEW` a `com.realtek.tv/com.realtek.tv.MainActivity` con extra `"change_source"` | `rtk-framework` | A | **Inútil aquí**: el paquete `com.realtek.tv` no está instalado |
| `com.realtek.tv.Tv.getInputSourceList(Context) : List` | Solo `TvInputManager.getTvInputList()` de Android | ídem | A | API pública |

No hay ningún método "cambiar a HDMI n" en `TTvInputMgr`. El cambio de fuente pasa por TIF (`TvView.tune` sobre `com.tcl.tvinput/.TvPassThroughService`, protegido con `BIND_TV_INPUT`) o por las apps del sistema. Componentes exportados declarados en sus manifiestos, sin permiso:

| Componente | Acciones |
| --- | --- |
| `com.tcl.suspension/.ui.main.InputActivity` (alias) | `com.android.tv.action.VIEW_INPUTS`, `com.tcl.suspension.VIEW_INPUTS` |
| `com.tcl.suspension/.ui.main.MainActivity` | `com.tcl.suspension.action.main` |
| `com.tcl.tv/com.tcl.player.TVActivity` | `android.intent.action.VIEW`, `tcl.sys.intent.action.TV` |
| `com.tcl.dashboard/.MainActivity` | `com.tcl.dashboard.MAIN` |
| `com.tcl.gamebar/.activity.MainActivity` | `com.tcl.gamebar.main`, `com.tcl.settings.action.start.ACTION_GAMEBAR_HDMI` |
| `com.android.tv.settings/com.tcl.settings.PictureActivity` | `android.settings.DISPLAY_SETTINGS` |
| `com.android.tv.settings/com.tcl.settings.SoundActivity` | `com.android.tv.action.VIEW_SOUND_SETTINGS` |
| `com.android.tv.settings/com.tcl.settings.channel.InputControlActivity` | `com.android.tv.settings.CEC` |
| `com.android.tv.settings/com.tcl.settings.device.SystemActivity` | `com.android.tv.settings.gamemode` |

## Energía

| clase.método(firma) | Qué hace | Librería | Cargable | Comprobaciones vistas |
| --- | --- | --- | --- | --- |
| `tcl.os.TPowerManager.getInstance()`; `getBackLightState() : int`; `setBackLightState(int) : int`; `setBackLightStateWithReason(int,String) : int` | Apagar y encender la retroiluminación (pantalla apagada con audio) *(inf.)*. Binder: `IPowerManager.getTclInnerService()` sobre `power` | `tcl-framework` | B | `TclPowerManagerServiceEx`: `enforceCallingOrSelfPermission("tcl.permission.ACCESS_EXTENDED_POWER")`, `prot=signature` |
| `TPowerManager.getStrEnabled() : boolean` / `setStrEnabled(boolean) : boolean` | Suspensión a RAM | ídem | B | `getStrEnabled`: solo `clearCallingIdentity`. `setStrEnabled`: ninguna vista |
| `tcl.app.TAlarmManager.getInstance(Context)`; `getPowerOnTime() : long`; `setPowerOnTime(long,PendingIntent) : void` | Encendido programado | ídem | B | `setPowerOnTime`: `"android.permission.SET_TIME"` (`signature\|privileged\|role`) |
| `tcl.os.TRunningTimeManager.getSystemRunningTime() : long`; `resetSystemRunningTime()` | Horas de uso | ídem | B | `"tcl.permission.ACCESS_EXTENDED_RUNNINGTIME"`, `prot=signature` |
| `com.realtek.tv.SystemControl.getInstance()`; `setPanelOnOff(boolean) : boolean`; `workingToPanelOff()`; `panelOffToWorking()`; `panelOffToLowPowerMode()`; `lowPowerModeToWorking()`; `getMntPowerMode()` / `setMntPowerMode(int)`; `setLedsBlink(boolean,int,int)` | Estados de energía del panel | `rtk-framework` | A | HAL `vendor.rtk.tv.IAidlRtkTvService/default`. Riesgo real de dejar la pantalla negra |

No aparece ningún método de temporizador de apagado en las librerías. Lo que existe es interno de Ajustes: receptor `com.tcl.settings.receiver.ShortCutMenuReceiver` (acciones `com.tcl.action.sleep`, `com.tcl.showsetting`, `com.tcl.showsource`, `com.tcl.ShortcutKey`) protegido con `com.tcl.settings.permission.ShortCutMenuReceiver` (`signature`), y `TimerManagerBroadcastReceiver` (`com.tcl.PowerOffTimeCount`) con `com.tcl.settings.permission.TimerManagerBroadcastReceiver` (`signature`). Los servicios `com.realtek.power.IEnergyMode/default`, `IHibernate`, `ILowPower`, `IPowerOnReason` están registrados pero no hay cliente Java en los jar analizados.

## Red

| clase.método(firma) | Qué hace | Librería | Cargable | Comprobaciones vistas |
| --- | --- | --- | --- | --- |
| `android.net.TclConnectivityManager.<init>(Context)`; `getNetworkQuality() : int`; `registerNetQoeCallback(ITclConnectivityManager$NetworkQoeCallback)` / `unregister…`; `reportAppQoe(int,int,int,String)` | Calidad de red. Servicio `tcl_connectivity` | `tcl-framework` | B | No se localizó comprobación en `TclConnectivityService` |
| `tcl.net.wifi.TWifiManager.getInstance(Context)`; `allowAutojoinGlobal(boolean)` | Auto-conexión Wi‑Fi | ídem | B | Sin leer |
| `tcl.net.wifi.p2p.TWifiP2pManager.getInstance()`; `setP2pListen(boolean) : boolean`; `setMiracastMode(int)`; `getP2pType() : int`; `setDeviceName(…)` | Wi‑Fi Direct y Miracast | ídem | B | Sin leer |
| `android.heraeye.impl.wifi.HeraWifiImpl`, `android.heraeye.impl.ethernet.HeraEthernetImpl` (`handleEvent(Parcel)`, `handleFunction(int,HeData)`) | Diagnóstico interno "Hera" | `tcl-he-wifi`, `tcl-he-ethernet` | B | **No para apps** |

Para Wi‑Fi y Ethernet no hay nada mejor que las API públicas (`ConnectivityManager`, `WifiManager`).

## Bluetooth

Solo `android.heraeye.impl.bluetooth.HeraBluetoothImpl` (`tcl-bluetooth.jar`, nivel B): reparación del mando (`saveRcRepairInfo(HeData)`, `sendRcConnectionErrorMsg(HeData)`), no una API de usuario. Usar `BluetoothAdapter` y los permisos de Android. Acciones de sistema vistas en Ajustes: `tcl.sys.intent.action.BT_ON_REQUEST`, `tcl.sys.intent.action.BT_PAIR_REQUEST`.

## Información del dispositivo

| clase.método(firma) | Qué hace | Librería | Cargable | Comprobaciones vistas |
| --- | --- | --- | --- | --- |
| `tcl.os.TDeviceInfoEx.getInstance()`; `getProjectName() : String`; `getSoftwareVersion() : String`; `getClientType() : String`; `getBoardInfo() : String`; `getBrand() : String`; `getProjectID() : int`; `getConfigSubVersion() : String` | Modelo interno, versión de firmware, placa. Delegan en `tcl.os.TDeviceInfo` → servicio `deviceinfo` (`getBrand` lee una propiedad con `TSystemProperties.get`) | `tromlib` | A | `DeviceInfoManagerService`: ninguna en estos métodos |
| `tcl.os.TComponentInfoEx.getInstance()`; `getComponentVersion(int) : String`; `getComponentInfo(int) : String`; `getComponentSubVersion() : String` | Versiones por partición: `SYSTEM_COMPONENT=1`, `PRODUCT_COMPONENT=2`, `VENDOR_COMPONENT=3`, `CONFIG_COMPONENT=4` | `tromlib` | A | Servidor `tcl_componentinfomanager` sin leer; existe el permiso `com.tvos.permission.ACQUIRE_COMPONENT_INFO` (`signature`) |
| `tcl.os.TDeviceInfo.getSn() : String`; `getDeviceID()`; `getMACAddress()` | Número de serie, id, MAC | `tcl-framework` | B | `enforceCallingPermission("com.tvos.permission.READ_DEVICE_INFO")`, `signature\|privileged`. No están en `TDeviceInfoEx` |
| `TDeviceInfo.hasFairplayKey()`, `hasMFiKey()`, `hasQDAKey()` | Claves instaladas | ídem | B | `"com.tvos.permission.ACCESS_DEVICE_KEY"`, `signature\|privileged` |
| `TDeviceInfo.setPowerOnMode(int)`; `setDeviceUUID(String)`; `resetFactoryData()` | Escrituras de fábrica | ídem | B | `com.tvos.permission.SET_POWER_MODE`, `SET_DEVICE_UUID`, `ALLOW_FACTORY_RESET` respectivamente (`signature\|privileged`). **No para apps** |
| `TDeviceInfo.cleanE2promInfo()`; `setProjectID(int)`; `setWatchDogEnabled(boolean)` | Escrituras de fábrica | ídem | B | **Ninguna vista** en `DeviceInfoManagerService`. Peligrosas; solo las frena el bloqueo de API ocultas (y SELinux, sin comprobar). **No para apps** |
| `com.tcl.deviceinfomanager.TclDeviceInfoManager.getInstance(Context)`; `getUDID()`, `getOAID()`, `getVAID()`, `getAAID()`, `isSupported()` | Identificadores publicitarios. Servicio `tcl_uniquedeviceid_manager`, que **no** aparece en `service list` | `tcl-framework` | B | Permisos `com.tcl.permission.TCL_DEVICEINFO_GETUDID`, `…SETOAIDSTATUS` (`signature`) |
| `tcl.os.TSystemProperties.get(String) : String`, `getInt`, `getBoolean`, `set(String,String)` | `android.os.SystemProperties` sin más | `tcl-framework` | B | Ninguna; manda SELinux por propiedad |
| `com.realtek.system.RtkProjectConfigs` (245 métodos), `RtkConfigs`, `RtkFrameworkConfigs` | Configuración de proyecto Realtek | `rtk-framework` | A | HAL `vendor.realtek.rtkconfigs.IRtkProjectConfigs/default` e `IRtkTvConfigs/default` |

## Sistema e interfaz

| clase.método(firma) | Qué hace | Librería | Cargable | Comprobaciones vistas |
| --- | --- | --- | --- | --- |
| `tcl.os.lock.TLockManager.getInstance()`; `isPasswordInitialized(int)`, `checkPassword(String,int)`, `setPassword(String,int)`, `cleanupPassword(int)`; variantes `…SystemPassword` | PIN de control parental. Servicio `lock_manager` | `tcl-framework` | B | Implementación no localizada. Permisos existentes: `com.tvos.permission.ACCESS_SYSTEM_PASSWORD`, `MODIFY_SYSTEM_PASSWORD` (`signature\|privileged`) |
| `tcl.app.TAccessControlManager.getInstance()`; `getAppLockPackages(int) : List`; `addAppLockPackages(List,int)`; `removeAppLockPackages(List,int)` | Bloqueo de apps | ídem | B | Sin leer |
| `com.android.server.mode.TclModeManager.getInstance()`; `checkIfInGameMode()`, `checkIfInHomeMode()`, `checkIfInSportMode()`, `checkIfInInterceptedMode()` | Modo global actual. Servicio `tcl_mode_manager` | ídem | B | Implementación no localizada |
| `tcl.view.TSurfaceControl.screenshot(Rect,int,int,int) : Bitmap` | Captura con `ScreenCapture.captureDisplay` | ídem | B | Ninguna propia; Android exige `READ_FRAME_BUFFER` |
| `tcl.view.TWindowManager.getTvMaskShowStatus()`, `setTvMaskShowOverlay(boolean,String)`, `updateUhdActivityList(String,ArrayList)` | Máscara de TV y lista de actividades 4K | ídem | B | Sin leer. Permiso existente: `com.tvos.permission.ACCESS_EXTENDED_WINDOW` (`signature`) |
| `tcl.app.TActivityTaskManager.moveToTMultiTask(Context,Rect)`, `resizeTMultiTask(Context,Rect)` | Multiventana TCL | ídem | B | Sin leer |
| `tcl.os.tguard.TGuardManager.*` (`killPidByUser`, `reclaimMem`, `getAppType(String)`, …) | Gestor de memoria y procesos. Servicio `tguard` | ídem | B | Sin leer. **No para apps** |
| `tcl.widget.TToast`, `TDialog`, `TLoadingDialog`, `TButton`, `TSeekBar`, … | Widgets con el estilo de TCL | ídem | B | No aplica |
| `android.provider.TclSettings` (`$Global`, `$Secure`, `$System`) | Solo añade "último paquete que modificó la clave": `CALL_METHOD_GET_GLOBAL_MOD_PKG = "GET_global_modify_pkg"`, `…SECURE…`, `…SYSTEM…`, `EXTRA_LAST_MODIFY_PACKAGE`. No define claves | ídem | B | Usa el `SettingsProvider` normal |

El panel rápido, el menú de fuentes y las notificaciones de TCL no tienen API Java en estas librerías: son apps (`com.tcl.dashboard`, `com.tcl.suspension`, `com.tcl.messagebox`) que se cierran entre sí con el servicio de acción `com.tcl.quickmenu.dismiss`. Permisos `normal` declarados que una app puede pedir: `com.tcl.suspension.read`, `com.tcl.settings.permission.NOTIFICATION_READ`, `com.tcl.settings.permission.NOTIFICATION_WRITE`, `com.tcl.ShortcutKey`, `com.tcl.dashboard.ACCESS_MESSAGE_EXT_SERVICE`.

## Otros (existen; no son para apps)

| Área | Clases | Librería | Cargable | Comprobaciones vistas |
| --- | --- | --- | --- | --- |
| Motor de la cámara / pantalla giratoria | `tcl.hardware.motor.TMotorManager.getInstance(Context)`: `rotate(int,int)`, `reset(int)`, `getMotorStatus(int)`, `getScreenRotatingState()`, `setLockMode(boolean)`, `upgradeMCUFirmware(…)` | `tcl-motor-framework` | B | `TMotorManagerService`: `"com.tvos.permission.MOTOR"` (`signature`). El servicio `TMotorManagerService` no aparece en `service list` |
| Cámara | `tcl.tvcamera.TCameraManager` (`getCameraStatus()`, `setCameraStatus(boolean)`, `setZoomStatus(boolean)`, `setAIdetect(AiType)`, `startFwUpgrade(String)`), `tcl.tvcamera.TCameraManagerEx` (`tromlib`: `getCameraStatus()`, `getFramerate()`, `setFramerate(int)`, `setZoomStatus(boolean)`) | `tcl-framework`, `tromlib` | B, A | `TCameraService`: `"com.tvos.permission.USB_CAMERA"` (`signature`) en `setStatus`, `startFwUpgrade`, `startMCUUpgrade` |
| Hotel | `tcl.os.hotel.THotelManager.setBootLogo(int,String)`; `TDeviceInfo.getHotelModeStatus()`, `verifyHotelPassword(String)` | `tcl-framework` | B | `"com.tvos.permission.TCL_HOTEL_SET_BOOT_LOGO"` (`signature`) |
| Fábrica | `vendor.tcl.factory.V1_0.IFactory` (HIDL, 88 clases), `vendor.tcl.factory.ext.V1_0.IFactoryExt`; `com.realtek.tv.Factory` (83 métodos); `TDeviceInfo.getFactoryPModeStatus()` | `tcl.factory.hal`, `tcl.factory.hal.ext`, `rtk-framework` | A | HAL de vendor. Puede dejar la tele inservible |
| Luces | `tcl.hardware.lights.TLightsManager.setLightState(int,int,int,int,int)`, `getLightState(int) : int[]` | `tcl-framework` | B | `"android.permission.CONTROL_DEVICE_LIGHTS"` (`signature\|privileged`) |
| Firmware y periféricos | `tcl.hardware.TFirmwareManager`, `tcl.hardware.TSubScreenManager`, `tcl.flex.TFlexSpeakerManager` | `tcl-framework` | B | Permisos `com.tvos.permission.UPGRADE_FIRMWARE`, `SUBSCREEN` (`signature`) |
| Almacenamiento, paquetes, copia | `tcl.os.storage.TStorageManager`, `tcl.pm.TPackageManager`, `tcl.backup.TBackupManager` | `tcl-framework` | B | `tcl.permission.STORAGE_DEVICE_SLEEP` (`signature`) |
| Reproductores | `tcl.media.mediaplayer.TMediaPlayer` (56 métodos), `tcl.media.pictureplayer.TPicturePlayer`; `com.realtek.media.RtkMediaPlayer` (70) | `tcl-framework`, `rtk-mediaplayer` | B | Sin leer |
| TV digital | `com.tcl.dtv.*` (escaneo, EPG, PVR, CI, CAS), `com.tcl.analog.TAnalogManager`, `com.realtek.tv.Tv` (618 métodos), `Source` (101), `VSC` (52) | `com.tcl.tvinput`, `rtk-framework` | A | Sin leer. Fuera del alcance de un launcher |
| HbbTV, Ginga, navegador | `com.tcl.hbbtv.HbbtvServer`, `com.tcl.interactive.*`; `com.tcl.gingad.*`, `tv.mopa.ginga.*`; `com.tcl.tbrowser.*`; `tcl.webkit.*` | `com.tcl.hbbtv`, `com.tcl.gingad`, `com.tcl.ginga.porting`, `com.tcl.tbrowser`, `tcl.webkit` | A (`tcl.webkit`: B) | Sin leer |
| Diagnóstico Hera | `android.heraeye.impl.*` | `hera-common`, `tcl-diaghung`, `tcl-nvrhung`, `tcl-perf` | B | Interno |

## Referencia: servicios binder

De `service list` (298 entradas). Interfaz entre corchetes tal como la devuelve el sistema; vacío = servicio nativo sin descriptor publicado.

| Servicio | Interfaz | Cliente Java visto |
| --- | --- | --- |
| `tcl_tv_display` | `com.tcl.tv.display.ITvDisplayManagerService` (191 métodos) | `com.tcl.tv.display` (A). Binario `/system_ext/bin/tcl_tv_display_service` |
| `tcl_tvinput` | `com.tcl.dtv.ITService` | `com.tcl.tvinput`, `com.tcl.hdmi` (A) |
| `hdmi_control` | `android.hardware.hdmi.ITclHdmiControlServiceEx` | `TclHdmiTvClient` (B) |
| `deviceinfo` | `com.tcl.os.ITclDeviceInfoManager` | `TDeviceInfo` (B), `TDeviceInfoEx` (A) |
| `tcl_componentinfomanager` | `com.tcl.os.ITclComponentInfoManager` | `TComponentInfo` (B), `TComponentInfoEx` (A) |
| `lock_manager` | `tcl.os.lock.ITLockManager` | `TLockManager` (B) |
| `tcl_mode_manager` | `android.mode.ITclModeManager` | `TclModeManager` (B) |
| `tcl_connectivity` | `android.net.ITclConnectivityService` | `TclConnectivityManager` (B) |
| `tcl_runningtime_manager` | `android.os.ITRunningTimeManager` | `TRunningTimeManager` (B) |
| `tcl_hotel_manager` | `tcl.os.hotel.ITHotelManager` | `THotelManager` (B) |
| `tguard` | `tcl.os.tguard.ITGuardService` | `TGuardManager` (B) |
| `TCameraService` | `tcl.tvcamera.ITCameraService` | `TCameraManager` (B) |
| `TFirmwareManagerService` | `tcl.hardware.ITFirmwareManager` | `TFirmwareManager` (B) |
| `apprecorder` | `android.apprecord.ITclAppRecordService` | `TclAppRecordManagerImpl` (B) |
| `aio`, `aio_native` | `tcl.aio.ITclAioManager`, `android.aio.ITclAioManagerNative` | `tcl.aio.TAioManager` (`framework.jar`, B) |
| `recordvideoinfo`, `tvod_scene` | `com.tcl.recordvideoinfo.ITclRecordVideoInfoManager`, `com.tcl.vod.ITVodSceneManager` | `framework.jar` (B) |
| `cameramotorservice`, `tstreamingservice` | `tcl.tstreaming.ITCameraMotor`, `tcl.tstreaming.ITStreaming` | Ninguno en los jar analizados |
| `android.heraeye.os.IHeraeye`, `android.heserver`, `hemonitor` | `android.heraeye.os.IHeraeye`, `android.heraeye.IHeService`, `android.heraeye.IHeMonitorManager` | Hera (B) |
| `audio`, `power`, `lights`, `alarm`, `window`, `activity_task`, `mount`, `package` | Interfaz AOSP con `getTclInnerService()` añadido | `android.media.ITclAudioManager`, `com.tcl.os.ITclPowerManager`, `com.tcl.hardware.lights.ITclLightsManager`, `com.tcl.app.ITclAlarmManager`, `android.view.ITclWindowManager`, `android.app.ITclActivityTaskManager`, `com.tcl.storage.ITclStorageManager`, `com.tcl.content.pm.ITclPackageManager` (B) |
| `vendor.rtk.tv.IAidlRtkTvService/default` | ídem | `com.realtek.tv.*` de `rtk-framework` (A) |
| `vendor.realtek.rtkconfigs.IRtkProjectConfigs/default`, `IRtkTvConfigs/default`, `vendor.realtek.hwconfigmanager.IHWConfigManager/default` | ídem | `com.realtek.system.*` (A) |
| `com.realtek.power.IEnergyMode/default`, `IHibernate/default`, `ILowPower/default`, `IPowerOnReason/default` | ídem | Ninguno |
| `vendor.tcl.tvinput.extend.ITvExtendService/default` | (vacío) | `com.tcl.dtv.TService` (A) |
| `vendor.tcl.audio.ITclAudio/default`, `/taudio`, `vendor.tcl.light.ITclLights/default`, `vendor.tcl.display.screencontrol.ITclScreenControl/default`, `vendor.tcl.bootreason.IBootReason/default`, `vendor.tcl.tcldevmanager.ITclDevManager/default`, `vendor.tcl.airplaykey.IAirplayKey/default`, `vendor.tcl.hardware.wifi.supplicant.ITclSupplicant/default`, `vendor.rtk.thal.IRtkThalService/default`, `vendor.realtek.aidl.display.IDisplayManager/default` | (vacío) | Ninguno. HAL de vendor |

## Referencia: content providers de TCL y Realtek

Autoridades de `dumpsys package`; `exported` y permisos leídos de los manifiestos de los APK. Omitidos los `FileProvider` y los inicializadores de androidx. Ninguno se ha consultado.

| Autoridad | Paquete | exported | Permiso de lectura | Permiso de escritura |
| --- | --- | --- | --- | --- |
| `com.tcl.providers.config` | `com.tcl.providers.config` | sí | ninguno | ninguno |
| `com.tcl.suspension` (`SourceOrderProvider`, orden de fuentes *(inf.)*) | `com.tcl.suspension` | sí | `com.tcl.suspension.read` (`normal`) | ninguno declarado |
| `tvlauncher.config`, `tvlauncher.apprecs`, `tvlauncher.widget`, `suspension_config`, `com.tcl.customer` | `com.tcl.partnercustomizer` | sí | ninguno | ninguno |
| `com.tcl.settings.slice`, `com.tcl.settings.preference` | `com.android.tv.settings` | sí | ninguno | ninguno |
| `com.android.tv.settings.device.eco.energymodes` | `com.android.tv.settings` | sí | ninguno | ninguno |
| `com.android.settings.connectivity.networkchangedetection` | `com.android.tv.settings` | sí | `com.android.settings.connectivity.READ_NETWORK_CHANGE_DATA` | ninguno |
| `com.tcl.messagebox` | `com.tcl.messagebox` | sí | `com.tcl.messagebox.access` (`signature\|privileged`) | ídem |
| `com.tcl.ttvs.messagebox.provider` | `com.tcl.ttvs` | sí | `com.tcl.messagebox.access` | ídem |
| `com.tcl.ttvs.process.provider` | `com.tcl.ttvs` | sí | `com.tcl.ttvs.process` (`signature\|privileged`) | ídem |
| `com.tcl.ttvs` | `com.tcl.ttvs` | sí | `com.tcl.permission.START_CHECK_SYSTEMUPDATE` (`signature\|privileged`) | ídem |
| `com.tcl.ttvs.bi.param` | `com.tcl.ttvs` | sí | `com.tcl.bi.param` (`normal`) | ídem |
| `com.tcl.ttvs.identifier`, `com.tcl.ttvs.proxy.business.data`, `com.tcl.xian.StartandroidService.MyContentProvider` | `com.tcl.ttvs` | sí | ninguno | ninguno |
| `com.tcl.miracast.provider` | `com.tcl.miracast` | sí | ninguno | ninguno |
| `com.tcl.hearaid.wireless_speakers` | `com.tcl.hearaid` | sí | `com.tcl.hearaid.wireless_speakers.READ_PROVIDER` | ídem |
| `com.tcl.tv.SearchContentProvider` | `com.tcl.tv` | sí | ninguno | ninguno |
| `org.passive.saas_token.cloud_url.providers` | `com.tcl.tv.tclhome_passive` | sí | ninguno | ninguno |
| `com.tcl.ui_mediaCenter.DataProvider`, `…HistoryDataProvider`, `…ScrapedDataProvider`, `…bookmark.BookMarkProvider` | `com.tcl.ui_mediaCenter` | sí | ninguno | ninguno |
| `com.tcl.store_ext.slice.OverseaStoreModeSliceProvider` | `com.tcl.esticker` | sí | ninguno | ninguno |
| `com.tcl.ocean.instructions.specification.slice` | `com.tcl.ocean.instructions` | sí | ninguno | ninguno |
| `com.tcl.tab.OemTabProvider` | `com.tcl.usercenter` | sí | `com.google.android.apps.tv.launcherx.MULTI_PROCESS_BROADCAST_PERMISSION` | ídem |
| `atv.hotwordmic`, `tvlauncher.mic` | `com.realtek.hw.configprovider` | sí | ninguno | ninguno |
| `com.tcl.guard.provider`, `com.tcl.guard.preference` | `com.tcl.guard` | no | — | — |
| `com.tcl.ttvs.framework.ipc.provider` | `com.tcl.ttvs` | no | — | — |

## Límites

Lo que la plataforma claramente no ofrece a una app sin firma de sistema:

- **Sonido**: ningún ajuste (modo, ecualizador, salida, S/PDIF) está en una librería de nivel A. Todo es `TAudioManager` (API oculta `blocked`) y el servidor pide `android.permission.MODIFY_AUDIO_ROUTING`. HOME solo tiene `AudioManager` y el `Intent` `com.android.tv.action.VIEW_SOUND_SETTINGS`.
- **Pantalla apagada con audio / reposo**: `TPowerManager.setBackLightState` exige `tcl.permission.ACCESS_EXTENDED_POWER` (`signature`). La alternativa Realtek (`SystemControl.setPanelOnOff`) va por un HAL de vendor.
- **Temporizador de apagado**: sin API; los receptores de Ajustes están protegidos con permisos `signature`.
- **Cambio directo de fuente**: no hay método en `TTvInputMgr`; `com.realtek.tv.Tv.setInputSource` apunta a un paquete que no existe aquí. Queda TIF o lanzar `com.android.tv.action.VIEW_INPUTS`.
- **Número de serie, MAC, id de dispositivo**: `com.tvos.permission.READ_DEVICE_INFO` (`signature|privileged`).
- **Control parental, bloqueo de apps, luces, motor, cámara, hotel, firmware, fábrica**: permisos `signature` o `signature|privileged` en todos los casos leídos.
- **Clases del BOOTCLASSPATH de TCL**: marcadas `blocked`; con `targetSdk 36` no son un camino soportado.
- **eARC y enlace CEC↔eARC**: declarados como `unavailable-feature` en `tcl_features.xml`.
- **Claves `tcl_sys_*` de `Settings`**: `TclSettings` no añade ninguna vía de escritura; siguen las reglas del `SettingsProvider` (`WRITE_SECURE_SETTINGS` es `signature|privileged|development|installer|role`).

## Sin comprobar

Todo lo siguiente necesita una prueba en la tele:

1. Que `com.tcl.tvinput`, `com.tcl.hdmi`, `tromlib` y `rtk-framework` carguen con `<uses-library>` igual que `com.tcl.tv.display`, y que SELinux permita a HOME encontrar `tcl_tvinput`, `deviceinfo`, `tcl_componentinfomanager` y `vendor.tcl.tvinput.extend.ITvExtendService/default`.
2. Que `tromlib` se trate como dominio de plataforma (sus llamadas internas a clases `blocked` no fallan). Prueba mínima: `tcl.os.TDeviceInfoEx.getInstance().getProjectName()`.
3. Qué comprueba el servicio nativo `tcl_tv_display` en cada método distinto de `getBacklight`/`setBacklight`. El binario no se pudo leer (`Permission denied`).
4. Los valores `int`: modos de imagen de `getPictureMode()`, el primer parámetro `int` de `TvPqManager`, el argumento de `setLightSensorOnoff` y `setLowBlueRay`, rangos de cada ajuste, y los `ordinal()` de los enums.
5. Las claves de los `Bundle` de `getHdmiDeviceInfo()`, `getVideoFormatInfo()` y `getHdmiSupportFeature(int)`; el contenido del `HashMap` de `getTvinputList()`.
6. Si `Settings.Secure "tcl_sys_tv_input_current_device_id"` es legible por una app con `targetSdk 36`.
7. La política de API ocultas del dispositivo (`Settings.Global hidden_api_policy` y exenciones); no se consultó. Si está relajada, el nivel B pasa a ser alcanzable.
8. La lectura estática de `setAudioOption` (comprobación de `MODIFY_AUDIO_ROUTING` evaluada tras `clearCallingIdentity`). Aunque fuese cierta, llegar al método exige saltarse el bloqueo de API ocultas.
9. Las implementaciones no localizadas: `lock_manager`, `tcl_mode_manager`, `tcl_connectivity`, `AudioService.setDevMode`, y los servicios que `service list` no muestra (`tcl_uniquedeviceid_manager`, `TMotorManagerService`, `TSubScreenManagerService`, `ipel`, `tcl_flex`).
10. Que los `Intent` de la tabla de componentes exportados abran lo que su nombre sugiere, y qué columnas y URIs exponen los providers sin permiso (`com.tcl.providers.config`, `com.tcl.suspension`).
11. Cualquier cosa de `com.realtek.tv.*`: aparte de SELinux, hay `System.exit` en al menos un cuerpo (`PQ.setBacklight`).

## Cómo se obtuvo

Jar extraídos con `adb pull` (27 de TCL/Realtek más `framework.jar` y `services.jar` para seguir las llamadas), `dexdump` de la imagen `lume-toolchain`, y cinco scripts pequeños: firmas públicas por clase, cuerpos de método condensados, nombre de servicio por método, cadenas de permiso por método y decodificador de `hiddenapi_class_data`. Los permisos y su nivel salen de `dumpsys package`; `pm list permissions -f` falla en esta tele con `NullPointerException`. Los volcados no están en el repositorio; se regeneran en unos minutos con los mismos pasos.

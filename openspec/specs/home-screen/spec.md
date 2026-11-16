# Home screen specification

## Purpose

The Home screen presents original beach artwork and a five-column grid of installed TV apps whose first row sits on a glass platter, following the spatial composition of tvOS 26 with Android TV controls.

## Requirements

### Beach hero

Home SHALL draw the selected wallpaper edge to edge, without a fictional feature title or launcher wordmark. Beach Cove is the default; Sunset Beach and Dawn Beach are available in the wallpaper picker. Wallpaper labels SHALL use the system language selected through Android locales.

#### Scenario: Spanish television
- **WHEN** the preferred locale is Spanish
- **THEN** the default wallpaper label reads “Cala turquesa” in the picker and no branding covers the beach on Home

#### Scenario: English television
- **WHEN** the preferred locale is English
- **THEN** the default wallpaper label reads “Beach Cove” in the picker and no branding covers the beach on Home

#### Scenario: Alternate wallpaper
- **WHEN** the user selects another wallpaper
- **THEN** Home samples the selected wallpaper without adding a fictional feature title

### App grid and glass platter

Home SHALL lay apps out as tvOS does: five 16:9 banners per row (212×120 units, 240-unit pitch, 54-unit side margins on the 1280×720 canvas), ordered favorites first and then other installed apps without duplicates. Only the first row SHALL sit on a rounded glass platter that samples the wallpaper. At rest Home SHALL show the wallpaper sharp and whole with only that first row, its platter against the bottom edge, and no app name. Down from the first row SHALL unfold the library: every row moves up at once with the platter and its row at the top, the wallpaper goes out of focus and darkens, the status capsule fades out, and deeper rows scroll one row at a time; Up on the second row SHALL fold it back. The focused banner SHALL grow to 1.12× with a brief overshoot, a wide soft shadow, a thin light rim, a band of light that crosses it once and a brief turn in perspective away from the direction the focus came from, and, while the library is unfolded, SHALL be the only one that shows its app name, centred under it. An app SHALL open with OK, and its Menu action SHALL open the appropriate favorite or catalog menu.

#### Scenario: First row
- **WHEN** Home appears with at least five installed apps
- **THEN** five banners appear at x=54..1226 on a 1280×720 canvas inside the platter, with the next row visible at the bottom

#### Scenario: Focused banner
- **WHEN** a banner receives the focus
- **THEN** it grows over its shadow and its name fades in under it, while no other banner shows a name

#### Scenario: No favorites
- **WHEN** the favorites list is empty
- **THEN** the first shelf tile opens the favorites editor, and installed apps follow it

#### Scenario: Navigation
- **WHEN** focus moves across and below the first row
- **THEN** Right and Left select neighboring cards, Down selects the next row, and Up from the first row selects the status capsule

### Focus pace and rest

Taking the focus SHALL be quicker than leaving it (leaving takes 1.4 times as long), and both SHALL shorten to 0.6 of their duration when the focus moves again within 300 ms. After 10 seconds without keys on Home, banners without the focus SHALL dim and the focused one SHALL grow slightly; any key SHALL undo it at once.

#### Scenario: Rest
- **WHEN** no key is pressed for 10 seconds on Home
- **THEN** every banner but the focused one dims, and the next key restores them

### Accessibility

HOME SHALL follow the TV's own settings instead of adding its own: with Android animations off (animator scale 0) it SHALL drop the overshoot, tilt, light band, jiggle and travelling transitions, keeping fades; with high-contrast text on, glass surfaces SHALL become nearly opaque. The focus tick SHALL sound only when the TV's system sounds are on, panned to where the focus is and quieter while the focus runs.

### Banner size

HOME's settings SHALL offer six banners per row (172×97 units, 200 pitch) instead of five; five SHALL be the default. Both keep the 54-unit side margins and the platter under the first row.

### Status capsule

Home SHALL NOT show a bar of tabs. A glass capsule in the top right corner SHALL show the time, with an amber Wi-Fi glyph in place of the controls glyph while Wi-Fi is off. Up from the first row SHALL give it the focus, when it turns white, and OK SHALL open the Control Center.

### Long press, moving and launch

OK SHALL act when it is released; while it is held, the focused banner or Control Center tile SHALL sink toward its unfocused size. Held for 500 ms it SHALL open the context menu of the focused banner instead, as the Menu key does. While a favorite is being moved, the favorites SHALL jiggle. An app SHALL open with a scale-up animation that starts at its banner.

#### Scenario: Long press
- **WHEN** OK is held for half a second on a favorite
- **THEN** its menu opens with Move, App info and Remove from Home (actions that take something away come last, and actions that do not apply are left out), and releasing OK opens nothing

### Home button

The system default launcher SHALL stay untouched. When the user enables the `HomeKeyService` accessibility service, the Home key and screen-on SHALL open HOME instead of the system launcher; disabling the service SHALL restore the system behaviour. Home pressed while HOME already shows the start of Home SHALL open the Control Center, Home pressed on the Control Center SHALL close it, and Home pressed anywhere else in HOME SHALL return to the start of Home.

#### Scenario: Service enabled
- **WHEN** the service is enabled and the user presses Home in another app
- **THEN** HOME comes to the front

#### Scenario: Service disabled
- **WHEN** the service is disabled
- **THEN** the Home key opens the launcher configured in Android

### Control Center

The Control Center SHALL open from the status capsule or from the Home key, as glass controls floating in the top right corner over a lightly dimmed picture and without a pane, with the focus on Wi-Fi. The controls SHALL come down from the corner while fading in, and go back to it when it closes, after which the focus returns to what had it. The focused control SHALL turn white with dark content. It SHALL provide: Sleep, which puts the TV to sleep through the Home key accessibility service; Wi-Fi and Bluetooth, which show whether the radio is on (a blue disc) and open the TV's panel; Brightness and Volume sliders that Left and Right change in steps of 5 % and that follow the TV's own values while the Control Center is open (brightness is the panel's backlight through the TV maker's picture library where there is one, never below 5 %; OK on it turns automatic brightness on or off where the TV allows it, and opens the TV's picture screen otherwise); AirPlay; a sleep timer that OK steps through 15, 30, 60 and 120 minutes and off; and round buttons for inputs (only when the TV has an input picker), screen mirroring, picture, sound, HOME's settings and Android settings. Settings an app cannot write SHALL open the TV screen that owns them. Back SHALL close it.

#### Scenario: TCL controls
- **WHEN** the user opens Control Center on a TCL 55Q5K and selects Wi-Fi, Bluetooth, picture, or ambient mode
- **THEN** the TV opens its respective network, accessory, display and sound, or ambient mode settings screen

#### Scenario: Inputs unavailable
- **WHEN** the TV has no input picker
- **THEN** the Inputs card is omitted and the remaining cards retain D-pad navigation

### Starting point and ambient screen

Whenever HOME leaves the screen (another app opens, or the TV sleeps) it SHALL be left at its start: Home, folded, the first app in focus, with no menu, move, Control Center or settings open. The Home press that brings it back SHALL show that start and nothing else. After the time chosen in the settings without keys on Home (5, 10 or 30 minutes; 10 by default; or never), or with Back on the first app, HOME SHALL show only the wallpaper, undimmed, and the time; any key SHALL return to the start of Home and do nothing else.

### HOME settings

HOME's settings SHALL be laid out as tvOS Settings: the title centred at the top, a glass emblem at the left with the installed version under it, and a list of rows at the right whose focused row turns white with dark content, grows slightly and casts a shadow. The list SHALL offer Home screen, Wallpaper, key mapping, TV information and About, and, under a "TV settings" heading, picture, sound, network and all Android settings, which open the TV's own screens. Home screen SHALL be a page with the six-per-row switch, changed in place with OK, and the entry to the editor of the main row. Wallpaper SHALL be a page listing every wallpaper with a mark on the current one; while it is open the whole screen SHALL show, undimmed, the wallpaper of the focused row, and OK SHALL choose it. Back on a page SHALL return to the row that opened it, and Back on the list SHALL return to Home. The statistics panel of `device-stats` and the settings layout of `launcher-shell` no longer apply to this fork.

#### Scenario: Wallpaper preview
- **WHEN** the user opens Wallpaper and moves the focus to another row
- **THEN** the screen shows that wallpaper behind the list, and the saved wallpaper only changes when OK is pressed

### Assets and performance

The three beach images SHALL be original generated artwork stored as QOI wallpaper assets. Glass SHALL sample a small copy of the wallpaper blurred once when the wallpaper is loaded, with one tap per pixel, a lens at the rim and a lit rim, without an offscreen pass at frame time or continuous animation at rest. Full-screen veils SHALL be part of the wallpaper pass (per-vertex color) rather than layers over it, and SHALL NOT animate. Each frame SHALL redraw only the part of the buffer that changed since that buffer was last shown (see `display-adaptation`). The installed Android and TCL packages SHALL remain enabled.

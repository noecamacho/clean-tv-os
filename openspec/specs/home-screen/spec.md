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

Home SHALL lay apps out as tvOS does: five 16:9 banners per row (212×120 units, 240-unit pitch, 54-unit side margins on the 1280×720 canvas), ordered favorites first and then other installed apps without duplicates. Only the first row SHALL sit on a rounded glass platter that samples the wallpaper; the other rows lie directly on the wallpaper over a light bottom veil. The next row SHALL peek from the bottom edge and rows SHALL scroll vertically when focused. The focused banner SHALL grow to 1.12× with a brief overshoot, a wide soft shadow and a thin light rim, and SHALL be the only one that shows its app name, centred under it. An app SHALL open with OK, and its Menu action SHALL open the appropriate favorite or catalog menu.

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
- **THEN** Right and Left select neighboring cards, Down selects the next row, and Up from the first row selects the navigation bar

### Navigation controls

Home SHALL show a compact time capsule when the app grid has focus. Pressing Up SHALL reveal a translucent glass navigation bar with Home, Apps, Control Center, Inputs (when available), and Settings controls.

### Home button

The system default launcher SHALL stay untouched. When the user enables the `HomeKeyService` accessibility service, the Home key and screen-on SHALL open HOME instead of the system launcher; disabling the service SHALL restore the system behaviour. Home pressed while HOME already shows the start of Home SHALL open the Control Center, Home pressed on the Control Center SHALL close it, and Home pressed anywhere else in HOME SHALL return to the start of Home.

#### Scenario: Service enabled
- **WHEN** the service is enabled and the user presses Home in another app
- **THEN** HOME comes to the front

#### Scenario: Service disabled
- **WHEN** the service is disabled
- **THEN** the Home key opens the launcher configured in Android

### Control Center

The Control Center SHALL open from its button in the navigation bar or from the Home key, as glass tiles floating in the top right corner, two per row, over a lightly dimmed picture and without a pane. The tiles SHALL come down from the corner while fading in. The focused tile SHALL turn white with dark content. It SHALL provide shortcuts to Wi-Fi, Bluetooth, picture, sound, ambient mode, installed apps, and Android settings. Inputs SHALL appear only when the TV reports an available input picker. Each shortcut SHALL launch the corresponding TV settings or input activity without toggling system state silently. Back SHALL return to the prior launcher tab with the Control Center button focused.

#### Scenario: TCL controls
- **WHEN** the user opens Control Center on a TCL 55Q5K and selects Wi-Fi, Bluetooth, picture, or ambient mode
- **THEN** the TV opens its respective network, accessory, display and sound, or ambient mode settings screen

#### Scenario: Inputs unavailable
- **WHEN** the TV has no input picker
- **THEN** the Inputs card is omitted and the remaining cards retain D-pad navigation

### Assets and performance

The three beach images SHALL be original generated artwork stored as QOI wallpaper assets. Glass SHALL reuse the existing GLES2 wallpaper texture and tile rendering paths with a one-tap refraction shader, without an offscreen blur pass or continuous animation. Each frame SHALL redraw only the part of the buffer that changed since that buffer was last shown (see `display-adaptation`). The installed Android and TCL packages SHALL remain enabled.

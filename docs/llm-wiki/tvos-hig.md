# Guía de diseño de tvOS (HIG) para HOME

Consultado el 2026-11-11. Resumen de lo más importante:

1. La rejilla de HOME ya coincide con la de Apple: la rejilla de cinco columnas del HIG es 320 pt de ancho + 40 pt de separación con 80 pt de margen lateral, es decir 213,3 + 26,7 y 53,3 unidades de HOME (HOME usa 212 + 28 y 54). La escala de foco 1,12 cae dentro de lo que Apple deja deducir de sus tablas de Top Shelf (1,09 a 1,14).
2. Apple **no publica** duraciones, curvas, factor de escala de foco, radios de esquina ni sombras de tvOS. Solo publica reglas cualitativas: el foco se comunica con escala, elevación, iluminación y movimiento, nunca solo con color; enfocar es más rápido que desenfocar; todo se acelera si el foco se mueve deprisa.
3. Lo que más falta en HOME y es barato: el estado de inactividad («el contenido sin foco se atenúa y el elemento enfocado se expande»), el estado de pulsación, los ajustes de accesibilidad (reducir movimiento, reducir transparencia, aumentar contraste, texto grande) y el sonido de foco con paneo y volumen según velocidad.
4. Liquid Glass es para la capa de controles y navegación, no para el contenido; Apple solo lo activa en Apple TV 4K de 2.ª generación o posterior y deja el aspecto anterior en equipos más viejos. La aproximación estática de HOME es coherente con esa degradación.
5. tvOS 27 no cambia el diseño del Home: añade tamaño de texto del sistema (Dynamic Type), barras laterales en Música y Podcasts, y un regulador de Liquid Glass de «ultraclaro» a «totalmente tintado». Top Shelf, carpetas, cambio de apps y perfiles siguen siendo las grandes ausencias de HOME, y varias chocan con límites de Android.

Etiquetas: `[A]` documento de Apple (se indica la página), `[V]` transcripción de una sesión (se indica cuál), `[I]` inferencia propia. Las conversiones a unidades de HOME dividen los puntos de Apple (lienzo 1920×1080 pt) entre 1,5 (lienzo 1280×720).

## Fuentes

Las páginas del HIG se leyeron por el JSON `https://developer.apple.com/tutorials/data/design/human-interface-guidelines/<slug>.json`; funcionó para todas menos dos. La URL pública es `https://developer.apple.com/design/human-interface-guidelines/<slug>`. No se vio ninguna imagen ni vídeo: de las figuras solo se leyó el texto alternativo.

### HIG

| Página (slug) | Qué se leyó |
|---|---|
| `designing-for-tvos`, `focus-and-selection`, `layout`, `materials`, `motion`, `top-shelf`, `remotes`, `app-icons`, `images`, `lockups`, `collections`, `image-views` | JSON completo |
| `context-menus`, `alerts`, `loading`, `launching`, `progress-indicators`, `digit-entry-views`, `feedback` | JSON completo |
| `typography`, `color`, `dark-mode`, `accessibility`, `menus`, `tab-bars`, `buttons`, `playing-audio`, `playing-video`, `scroll-views` | JSON; secciones de tvOS completas y las secciones generales relevantes (tablas de tvOS, Liquid Glass color, contraste, tamaños de control, buenas prácticas). No se leyeron las secciones de otras plataformas |
| `icons`, `sf-symbols`, `sidebars`, `lists-and-tables`, `text-fields`, `search-fields`, `virtual-keyboards`, `sheets`, `action-sheets`, `popovers`, `modality`, `searching`, `entering-data`, `onboarding`, `settings`, `segmented-controls`, `page-controls`, `pickers`, `toggles`, `split-views`, `labels`, `text-views`, `boxes`, `game-controls`, `live-viewing-apps`, `notifications`, `managing-accounts`, `going-full-screen`, `right-to-left`, `privacy`, `writing`, `inclusion`, `shareplay`, `airplay`, `voiceover`, `sign-in-with-apple`, `branding`, `activity-rings`, `rating-indicators`, `siri`, `pull-down-buttons`, `pop-up-buttons`, `live-photos`, `components`, `foundations`, `patterns` | JSON; solo la sección de tvOS y las líneas que mencionan tvOS, Apple TV, Siri Remote o foco. Muchas dicen únicamente «No additional considerations for tvOS» |
| `in-app-purchase`, `tv-provider-authentication` | No accesibles: el JSON devolvió 404 con esos slugs |

### Documentación de desarrollador y guía de usuario de Apple

| URL | Qué se leyó |
|---|---|
| `developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass` | JSON; menciones a tvOS y foco |
| `developer.apple.com/documentation/swiftui/creating-a-tvos-media-catalog-app-in-swiftui` | JSON completo |
| `developer.apple.com/documentation/uikit/about-focus-interactions-for-apple-tv`, `.../uikit/adding-user-focusable-elements-to-a-tvos-app`, `.../uikit/uifocusanimationcoordinator`, `.../uikit/uifocusmovementhint`, `.../uikit/uiimageview/adjustsimagewhenancestorfocused`, `.../swiftui/hovereffect/highlight`, `.../swiftui/hovereffect/lift` | JSON completo (son páginas cortas) |
| `developer.apple.com/documentation/xcode/configuring-your-app-icon` | JSON; menciones a tvOS |
| `developer.apple.com/documentation/tvos-release-notes/tvos-26-release-notes` y `tvos-27-release-notes` | JSON; búsqueda por palabras clave. No contienen cambios de diseño del Home |
| `support.apple.com/guide/tv/…/tvos` (guía de usuario de Apple TV 4K, edición tvOS 27): navegar, reordenar apps, abrir una app, cambiar de app, Centro de control (navegar, ajustes generales, iconos de pestaña, iconos de estado), salvapantallas (activar, configurar, ajustes), apariencia, tamaño de texto, luz y color, ajustes del mando, encender y apagar, perfiles, notificaciones, Buscar, novedades de tvOS 27 | HTML; texto completo de cada página. Mi extractor descartó las líneas de menos de 26 caracteres, así que faltan algunos pasos cortos y los nombres de botones dibujados como icono |
| `apple.com/newsroom/2025/06/apple-tv-brings-a-beautiful-redesign-and-enhanced-home-entertainment-experience/` | HTML; texto completo |
| `apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/` | HTML; párrafos sobre diseño |
| `apple.com/newsroom/2026/09/major-updates-for-apples-software-platforms-are-now-available/` | HTML; párrafos sobre diseño y Apple TV |

### Sesiones (transcripciones)

URL: `https://developer.apple.com/videos/play/wwdcAAAA/NNN/` salvo que se indique otra.

| Sesión | Qué se leyó |
|---|---|
| WWDC25 219, «Meet Liquid Glass» | Transcripción completa |
| WWDC25 356, «Get to know the new design system» | Transcripción: primera mitad completa; el resto solo buscando tvOS (no aparece) |
| WWDC26 221, «Prepare your tvOS apps for Dynamic Type» | Transcripción completa |
| WWDC20 10042, «Build SwiftUI apps for tvOS» (enlazada desde `designing-for-tvos`) | Transcripción completa |
| WWDC19 211, «Mastering the Living Room With tvOS» (enlazada desde `top-shelf`) | Transcripción; párrafos sobre Top Shelf, foco, pantalla completa y barra de pestañas |
| WWDC18 803, «Designing Fluid Interfaces» | Transcripción; frases sobre muelles, interrupción, inercia y Apple TV |
| WWDC23 10158, «Animate with springs» | Transcripción; frases sobre duración y rebote |
| WWDC17 803, «Designing Sound» | Transcripción; pasaje sobre los sonidos de transición de Apple TV |
| WWDC23 10162, «The SwiftUI cookbook for focus» | Transcripción; párrafos sobre tvOS |
| WWDC21 10023, «Direct and reflect focus in SwiftUI» | Transcripción; solo menciones a tvOS |
| WWDC23 10078, «Design considerations for vision and motion» (enlazada desde `motion`) | Transcripción; frases sobre movimiento. Trata de visionOS |
| WWDC24 10145, «Enhance your UI animations and transitions» (enlazada desde `motion`) | Transcripción; frases sobre la transición de zoom. Trata de iOS |
| WWDC25 323 y 284, «Build a SwiftUI/UIKit app with the new design» | Transcripción; solo menciones a TV (hablan de la app TV en iOS) |
| WWDC25 220, «Say hello to the new look of app icons»; WWDC26 250, «Principles of great design»; WWDC26 269, «What’s new in SwiftUI»; WWDC26 278, «Modernize your UIKit app» | Transcripción; búsqueda de tvOS sin resultados útiles |
| WWDC25 102 y WWDC26 102, «Platforms State of the Union» | Transcripción; búsqueda de tvOS sin resultados útiles |
| WWDC26 101, Keynote | Transcripción; tvOS solo se nombra de pasada, sin detalle |
| WWDC25 101, Keynote | No accesible: la página no trae transcripción |
| WWDC17 224, «Focus Interaction in tvOS 11» | Apple ya no publica la página (redirige al índice). Leída en el espejo de terceros `asciiwwdc.com/2017/sessions/224`: frases sobre animación y sonido de foco |
| WWDC16 215, «Focus Interaction on tvOS»; WWDC16 210, «Mastering UIKit on tvOS»; WWDC17 209, «What’s New in tvOS» | Igual: retiradas por Apple, leídas por palabras clave en `asciiwwdc.com` |
| WWDC20 10640, WWDC23 10073 y 10076 (enlazadas desde `focus-and-selection`) | No leídas: tratan del puntero de iPadOS y de visionOS |

No existe una sesión «What’s new in tvOS» ni una sesión de diseño de tvOS en los índices de WWDC25 y WWDC26; la única sesión de tvOS de WWDC26 es la 221.

## Lo que dice Apple

### Foco

- `[A]` (`focus-and-selection`) En tvOS enfocar no selecciona: «selection requires a separate gesture». tvOS usa el efecto de paralaje «to give the focused item an appearance of depth and liveliness».
- `[A]` (`focus-and-selection`) Cinco estados visualmente distintos. Según el texto y las figuras de Apple:
  - **Unfocused**: menos prominente; sombra pequeña, fondo translúcido teñido por el contenido, texto de alto contraste.
  - **Focused**: «elevation to the foreground, illumination, and animation»; más grande, sombra más lejana, fondo blanco opaco y etiqueta negra (en un botón).
  - **Highlighted** (se está pulsando): «instant visual feedback»; mismo tamaño que sin foco, fondo blanco opaco; un botón puede «briefly invert its colors and animate».
  - **Selected**: mismo tamaño que sin foco, fondo blanco opaco, sombra pequeña.
  - **Unavailable**: sin sombra, translúcido, texto de bajo contraste; no se puede enfocar.
- `[A]` (`focus-and-selection`) «Because focusing an item often increases its scale», hay que dar recursos nítidos al tamaño enfocado y cuidar que el elemento ampliado no invada lo que lo rodea.
- `[A]` (`color`, tvOS) «Avoid using only color to indicate focus. Subtle scaling and responsive animation are the primary ways to denote interactivity».
- `[A]` (`focus-and-selection`) No mover el foco sin acción de la persona. Excepción: si el elemento enfocado desaparece y la entrada es direccional, pasar el foco a un vecino a un paso de distancia.
- `[A]` (`focus-and-selection`) Todo elemento en pantalla debe poder recibir foco con movimientos direccionales. No mostrar puntero. En pantalla completa los gestos actúan sobre el contenido, no sobre el foco.
- `[A]` (`uikit/about-focus-interactions-for-apple-tv`) Solo un elemento enfocado a la vez. Si no hay nada enfocable en la dirección pedida, el foco no cambia. Algunos elementos, como las barras de pestañas, se seleccionan solos «after a slight delay».
- `[A]` (`uikit/adding-user-focusable-elements-to-a-tvos-app`) Al arrancar se enfoca el elemento superior izquierdo (superior derecho en idiomas de derecha a izquierda). Disponer los elementos en rejilla es la forma más segura de que el foco se mueva bien.
- `[A]` (`uikit/uifocusanimationcoordinator`) «focusing animations typically run faster than unfocusing animations, to create a trail effect»; «when focus is moving quickly, the timing of the animations are sped up». No se debe suponer una duración fija.
- `[A]` (`swiftui/hovereffect/highlight`) El efecto para imágenes aplica «a projection effect accompanied with a specular highlight» y paralaje ajustando la matriz de proyección y el desplazamiento del brillo. El efecto `lift` (botones y texto) escala y añade sombra.
- `[A]` (`swiftui/creating-a-tvos-media-catalog-app-in-swiftui`) El lockup estándar da «lift, a specular highlight, and gimbal motion effects»; el título del botón y los títulos de sección cercanos «automatically move out of the way» cuando la imagen crece. El estilo `card` da un plato y un movimiento más sutil para contenido denso.
- `[A]` (`lockups`) Los carteles ocultan título y subtítulo hasta recibir foco. Los botones con leyenda se inclinan siguiendo el gesto: arriba y abajo en vertical, izquierda y derecha en horizontal, en ambos ejes en rejilla.
- `[A]` (`images`) «After a period of inactivity, out-of-focus content dims and the focused element expands». «Parallax is designed to be almost unnoticeable».
- `[V]` (WWDC16 210, espejo) Las imágenes enfocadas tienen un estado pulsado con aspecto hundido («smooshed-in»), activable con `isHighlighted`.
- `[V]` (WWDC20 10042) El foco inicial es el elemento más arriba o más al inicio de la pantalla. Los menús contextuales se invocan con pulsación larga.
- `[V]` (WWDC18 803) El motor de foco de Apple TV es un ejemplo de respuesta continua: el movimiento del dedo se refleja de forma continua antes de que el foco salte.
- Apple no publica este valor: factor de escala de foco, duración y curva de la animación, radio y opacidad de la sombra, ángulo máximo de inclinación, tiempo de inactividad antes de atenuar.
- `[I]` De las tablas de Top Shelf (tamaño sin foco frente a tamaño enfocado) sale una escala de 1,09 (16:9: 782→852 pt), 1,14 (cuadrado: 500→570 pt) y 1,11 en altura (banner: 560→620 pt). No es un valor que Apple declare como «escala de foco».

### Layout y rejilla

- `[A]` (`layout`, tvOS) Zona segura: 60 pt arriba y abajo, 80 pt a los lados (40 y 53,3 unidades de HOME), por el overscan.
- `[V]` (WWDC19 211) La sesión dice 90 pt a los lados y 60 pt arriba y abajo (60 y 40 unidades). Contradice al HIG en el margen lateral; el HIG es la fuente más reciente.
- `[A]` (`layout`, tvOS) Rejillas recomendadas. Separación horizontal siempre 40 pt (26,7 u); separación vertical mínima 100 pt (66,7 u):

| Columnas | Ancho sin foco (pt) | Ancho (unidades HOME) |
|---|---|---|
| 2 | 860 | 573,3 |
| 3 | 560 | 373,3 |
| 4 | 410 | 273,3 |
| 5 | 320 | 213,3 |
| 6 | 260 | 173,3 |
| 7 | 217 | 144,7 |
| 8 | 184 | 122,7 |
| 9 | 160 | 106,7 |

- `[A]` (`layout`, tvOS) Filas con título: dejar espacio entre el borde inferior de la fila anterior sin foco y el centro del título, y entre el título y la fila. Espaciado constante. «Make partially hidden content look symmetrical».
- `[A]` (`layout`) Extender el fondo a pantalla completa por debajo de barras y paneles. Si un panel tapa parte de la imagen, el «background extension effect» la voltea y difumina bajo el panel.
- `[A]` (`accessibility`) Tamaño de control en tvOS: 66×66 pt por defecto, 56×56 pt mínimo (44 y 37,3 u). Guía general, no específica de tvOS: unos 12 pt de relleno alrededor de elementos con bisel y 24 pt sin bisel.
- `[A]` (`tab-bars`, tvOS) La barra de pestañas mide 68 pt de alto y su borde superior está a 46 pt del borde de la pantalla (45,3 y 30,7 u); «you can’t change either of these values». Es translúcida y solo la pestaña seleccionada es opaca. Si sobran elementos, se desvanece el borde derecho. Atrás (Menu) siempre devuelve el foco a la barra.
- `[A]` (`scroll-views`, tvOS) No hay indicadores de desplazamiento: «the system automatically scrolls the interface to keep focused items visible».
- `[A]` (`split-views`, tvOS) Por defecto un tercio para el panel principal y dos tercios para el secundario; un único título encima.
- `[A]` (guía de usuario, «Change appearance settings») Con el tamaño de elementos en «Large», «the top row of the Home Screen shows five icons instead of six». La fila superior normal muestra seis.
- `[A]` (`swiftui/creating-a-tvos-media-catalog-app-in-swiftui`) En búsqueda, «several rows of three to five items per row». Las estanterías desactivan el recorte para que el elemento enfocado pueda crecer.
- Apple no publica este valor: tamaño en pantalla de los iconos del Home, altura y posición de la fila superior, separación entre filas del Home.

### Materiales y Liquid Glass

- `[A]` (`materials`) Liquid Glass forma «a distinct functional layer for controls and navigation elements». «Don’t use Liquid Glass in the content layer». «Use Liquid Glass effects sparingly».
- `[A]` (`materials`) Dos variantes. **Regular** difumina y ajusta la luminosidad del fondo; es para texto abundante (alertas, barras laterales). **Clear** es muy translúcida y solo va sobre fondos ricos (fotos, vídeo).
- `[A]` (`materials`) Bajo vidrio *clear* sobre contenido claro: «a dark dimming layer of 35% opacity». Sobre contenido oscuro no hace falta.
- `[A]` (`materials`, tvOS) «Liquid Glass appears throughout navigation elements and system experiences such as Top Shelf and Control Center. Certain interface elements, like image views and buttons, adopt Liquid Glass when they gain focus».
- `[A]` (`materials`, tvOS) Materiales estándar para la capa de contenido: `ultraThin` para vistas a pantalla completa con esquema claro; `thin` para superposiciones parciales con esquema claro; `regular` para superposiciones parciales; `thick` para superposiciones parciales con esquema oscuro.
- `[A]` (`adopting-liquid-glass`) «Apple TV 4K (2nd generation) and newer models support Liquid Glass effects. On older devices, your app maintains its current appearance».
- `[A]` (`color`, Liquid Glass color) El vidrio no tiene color propio. Los elementos pequeños alternan entre claro y oscuro según el fondo; los grandes son más opacos. Color solo para acciones principales o indicadores de estado, y aplicado al fondo del control, no al texto. No colorear el fondo de varios controles.
- `[V]` (WWDC25 219) El vidrio se define por la lente: curva y concentra la luz en vez de dispersarla. Aparece y desaparece «by gradually modulating the light bending and lensing», no con un fundido. Al interactuar se flexiona y se ilumina desde dentro. Al crecer (un menú) se comporta como un material más grueso: sombra más profunda, más refracción.
- `[V]` (WWDC25 219) La sombra del vidrio es más opaca sobre texto y más tenue sobre fondo claro liso. «Always avoid glass on glass»; sobre vidrio se usan rellenos, transparencia y vitalidad. *Regular* y *Clear* «should never be mixed».
- `[V]` (WWDC25 219) *Clear* solo si se cumplen tres condiciones: está sobre contenido rico en imagen, ese contenido no sufre con una capa de atenuación, y lo que va encima es grueso y brillante.
- `[V]` (WWDC25 219) Los elementos grandes (menús, barras laterales) no alternan entre claro y oscuro. En reposo (al abrir) evitar que el contenido se cruce con el vidrio.
- `[V]` (WWDC25 219) Reducir transparencia hace el vidrio «frostier»; Aumentar contraste lo vuelve «predominantly black or white» con borde contrastado; Reducir movimiento baja la intensidad y desactiva lo elástico.
- `[V]` (WWDC25 356) Tres formas: radio fijo, cápsula (radio = mitad de la altura) y concéntrica (radio del padre menos el relleno). Un modal que interrumpe lleva capa de atenuación; una tarea paralela no. Las hojas de acciones nacen del control que las abre.
- `[A]` (newsroom, junio de 2025) En tvOS 26 el vidrio «reflects and refracts its surroundings using real-time rendering» y mantiene el vídeo a la vista al usar el Centro de control.
- `[A]` (newsroom, septiembre de 2026) «a new slider in Settings gives users the option to personalize Liquid Glass, adjusting it anywhere from ultraclear to fully tinted». La nota lo presenta como cambio general de diseño; no dice de forma expresa que esté en tvOS 27.
- `[A]` (`swiftui/creating-a-tvos-media-catalog-app-in-swiftui`) El Home de tvOS «blurs the most recently displayed top-shelf image and uses it as the background». Patrón por encima y por debajo del pliegue: arriba la imagen se ve nítida con un degradado de material; abajo «the entire image blurs».
- Apple no publica este valor: radio de desenfoque, índice de refracción, anchura de la lente, opacidad de tintes, radios de esquina.

### Movimiento

- `[A]` (`motion`) Sin consideraciones específicas de tvOS y sin cifras. Reglas: movimiento con propósito; opcional; breve y preciso; evitar animar interacciones frecuentes en elementos propios; «Let people cancel motion»; coherencia espacial (lo que entra por arriba sale por arriba).
- `[A]` (`motion`) En juegos, 30 a 60 fps. Es la única cifra de la página.
- `[A]` (`designing-for-tvos`) «subtle and fluid animations».
- `[A]` (`accessibility`) Con Reducir movimiento: tensar los muelles para quitar rebote, no animar cambios de profundidad, sustituir las transiciones en x, y, z por fundidos, no animar entradas y salidas de desenfoque.
- `[V]` (WWDC18 803) Empezar con amortiguación del 100 % (sin rebote). El rebote se reserva para gestos con inercia; ejemplo citado: 80 % de amortiguación al cerrar con un deslizamiento. Las animaciones deben poder interrumpirse y redirigirse.
- `[V]` (WWDC23 10158) Los muelles se definen con duración y rebote. Rebote 0: suave; 15 %: algo más vivo; 30 %: rebote ya visible. No esperar a la duración de asentamiento para cambios visibles.
- `[V]` (WWDC17 224, espejo) En el foco, animaciones demasiado rápidas resultan bruscas y demasiado lentas estorban. Las animaciones propias deben derivarse de la duración que da el sistema (por ejemplo, la mitad).
- `[V]` (WWDC19 211) Demasiadas animaciones, iconos y estilos de texto compiten por la atención; la interfaz de TV debe dejar a la persona «focused and relaxed».
- Apple no publica este valor: duración y curva de la animación de foco, de apertura de apps, del Centro de control y del desplazamiento del Home.

### Tipografía

- `[A]` (`typography`, tvOS) «SF Pro is the system font in tvOS, and apps can also use NY».
- `[A]` (`typography`, `accessibility`) Tamaño por defecto 29 pt y mínimo 23 pt (19,3 y 15,3 u). Evitar pesos Ultralight, Thin y Light.
- `[A]` (`typography`) Estilos de texto de tvOS:

| Estilo | Peso | Tamaño (pt) | Interlínea (pt) | Peso enfatizado | Tamaño (u HOME) |
|---|---|---|---|---|---|
| Title 1 | Medium | 76 | 96 | Bold | 50,7 |
| Title 2 | Medium | 57 | 66 | Bold | 38 |
| Title 3 | Medium | 48 | 56 | Bold | 32 |
| Headline | Medium | 38 | 46 | Bold | 25,3 |
| Subtitle 1 | Regular | 38 | 46 | Medium | 25,3 |
| Callout | Medium | 31 | 38 | Bold | 20,7 |
| Body | Medium | 29 | 36 | Bold | 19,3 |
| Caption 1 | Medium | 25 | 32 | Bold | 16,7 |
| Caption 2 | Medium | 23 | 30 | Bold | 15,3 |

- `[A]` (`typography`) Tracking de tvOS en milésimas de em, para los tamaños de la tabla: 23 pt −4; 25 pt +6; 29 pt +14; 31 pt +13; 38 pt +10; 48 pt +8; 57 pt entre +6 y +5 (la tabla da 56 y 58); 76 pt +1; de 80 pt en adelante 0.
- `[V]` (WWDC26 221) tvOS 27 trae tamaño de texto del sistema, de «Large» hasta «Accessibility XXXL» (Ajustes > Accesibilidad > Pantalla > Tamaño del texto). Reglas: estilos semánticos en vez de tamaños fijos; anchos flexibles; con texto grande, pasar de 6 a 4 columnas y de disposición horizontal a vertical en tarjetas; considerar una marquesina para títulos largos.
- `[V]` (WWDC25 356) En el nuevo sistema la tipografía es más gruesa y alineada a la izquierda en alertas y bienvenida.
- `[I]` SF Pro tiene licencia de Apple para sus plataformas; HOME no puede distribuirla. Hay que elegir otra sans y ajustar tamaño, peso y tracking a la tabla.

### Color

- `[A]` (`color`, tvOS) «Consider choosing a limited color palette that coordinates with your app logo». No usar solo color para indicar foco.
- `[A]` (`color`) Probar en varias marcas de televisores HD y 4K y con distintos ajustes de imagen. No depender solo del color para comunicar estado.
- `[A]` (`dark-mode`) tvOS tiene apariencia clara, oscura y automática; no ofrecer un ajuste de apariencia propio. Se admite una interfaz siempre oscura en apps de medios inmersivos.
- `[A]` (`accessibility`) Contraste mínimo 4,5:1 para texto de hasta 17 pt; 3:1 para 18 pt o más, o texto en negrita. `[A]` (`dark-mode`) Con colores propios, aspirar a 7:1.
- `[A]` (`playing-video`, tvOS) Algunos televisores retienen imagen: preferir gráficos translúcidos en SDR a contenido brillante y opaco, y superposiciones breves.

### Iconos de app e imágenes por capas

- `[A]` (`app-icons`) Icono de tvOS: lienzo de 800×480 px (proporción 5:3), rectángulo apaisado con esquinas redondeadas por el sistema, de 2 a 5 capas, estilo «Layered (Parallax)», sin variantes de apariencia. sRGB o Display P3.
- `[A]` (`app-icons`) Al enfocar, el icono «elevates to the foreground in response to someone’s finger movement on their remote, and gently sways while the surface illuminates».
- `[A]` (`app-icons`) Capas sin enmascarar; el sistema aplica las esquinas. No incluir brillos, sombras entre capas, biseles ni desenfoques propios. El texto, en la capa superior. Zona segura: el sistema recorta más las capas delanteras que el fondo.
- `[A]` (`images`, tvOS) «Layered images are at the heart of the Apple TV user experience». De 2 a 5 capas; fondo opaco obligatorio; primer plano para lo prominente y el texto; capas medias para contenido secundario y sombras. Obligatorias para el icono de app; opcionales pero «strongly encouraged» para el resto.
- `[A]` (`images`) tvOS usa escalas @1x y @2x.
- `[A]` (`lists-and-tables`, tvOS) Una fila enfocada se resalta, crece un poco y puede redondear sus esquinas; no añadir máscaras propias.
- `[A]` (newsroom, septiembre de 2026) En la versión 27 los iconos son «sharper and more defined».
- Apple no publica este valor: radio de esquina del icono, desplazamiento de cada capa, tamaño del icono en el Home.

### Top Shelf

- `[A]` (`top-shelf`) Zona del Home que muestra contenido de la app enfocada en el Dock (la fila superior). Al seleccionar una app en el Dock empiezan las previsualizaciones a pantalla completa «and soon the Dock slides away».
- `[A]` (`top-shelf`) Contenido nuevo y personalizado; sin anuncios ni precios; una imagen estática no debe parecer interactiva.
- `[A]` (`top-shelf`) Imagen estática de reserva: 2320×720 pt (1546,7×480 u). tvOS «flips and blurs the image» para llenar 1920 de ancho en 16:9.
- `[A]` (`top-shelf`) Disposiciones dinámicas: carrusel con acciones (título, subtítulo, botón principal y «More Info»), carrusel con detalles (añade resumen, reparto y metadatos; el título va arriba), fila de contenido por secciones y banner insertado con desplazamiento.
- `[A]` (`top-shelf`) Tamaños de la fila por secciones:

| Forma | Real (pt) | Enfocado / zona segura (pt) | Sin foco (pt) | Sin foco (u HOME) |
|---|---|---|---|---|
| Cartel 2:3 | 404×608 | 380×570 | 333×570 | 222×380 |
| Cuadrado 1:1 | 608×608 | 570×570 | 500×500 | 333,3×333,3 |
| 16:9 | 908×512 | 852×479 | 782×440 | 521,3×293,3 |
| Banner insertado | 1940×692 | 1740×620 | 1740×560 | 1160×373,3 |

- `[I]` El «333×570» del cartel sin foco parece una errata de Apple: 333×500 conservaría la proporción 2:3.
- `[A]` (`top-shelf`) En una fila con formas mezcladas todo se escala a la altura del más alto (un 16:9 pasa a 500 px de alto). Cargar imágenes suficientes para cubrir el ancho y al menos una etiqueta.
- `[A]` (`top-shelf`) Banner insertado: de 3 a 8 imágenes, avanza solo con temporizador hasta que una recibe foco, vuelve al principio al acabar; el texto debe ir dentro de la imagen.
- `[V]` (WWDC19 211) Al enfocar la app se muestra un cartel «for five seconds» y luego entra el tráiler, sin forzar el sonido. Deslizar hacia arriba pasa a pantalla completa y «the dock flies away». Los metadatos aparecen a demanda. Puntos de paginación para situarse. Entre 5 y 10 elementos. Imágenes del carrusel a 1920×1080 pt, con cuidado de que el arte no quede bajo los controles ni bajo la rejilla de apps.
- `[A]` (guía de usuario, «Navigate Apple TV 4K») Solo ciertas apps de la fila superior reproducen previsualización de fondo al resaltarlas.

### Menús y menús contextuales

- `[A]` (`context-menus`, `menus`) «No additional considerations for tvOS»: vale la guía general.
- `[A]` (`context-menus`) Solo los comandos más probables; pocos elementos; presentes de forma coherente en toda la app; cada comando también disponible en la interfaz principal. «Hide unavailable menu items, don’t dim them». Submenús de un solo nivel. Como mucho unos tres grupos con separadores. Casi nunca con título.
- `[A]` (`context-menus`) Elementos destructivos al final y marcados como tales. Apple lo enuncia para iOS, iPadOS y visionOS; tvOS no se nombra.
- `[A]` (`menus`) Etiquetas con verbo, sin artículos, mayúsculas de título en inglés, puntos suspensivos si falta información. Iconos para todo un grupo o para ninguno. Submenú de más de unos cinco elementos: mejor otro menú. Marca de verificación para atributos activos.
- `[A]` (guía de usuario, «Navigate» y «Rearrange apps») Pulsación larga del centro del clickpad sobre un elemento abre un menú emergente. En una app del Home, el menú lleva a editar el Home, a «Move to > New Folder», a «Move to > [carpeta]» y a «Move to > Home Screen». Se arrastra con el clickpad y se confirma pulsando el centro.
- `[A]` (guía de usuario, «Rearrange apps») Carpetas: se crean al mover una app a «New Folder», se renombran desde el título, y desaparecen al sacar la última app. La fila superior es para favoritos.
- `[V]` (WWDC25 219) Al mostrar un menú, la burbuja «simply pops open» desde el botón, para dejar clara la relación con su origen.

### Centro de control y otras experiencias del sistema

El HIG solo dice que el Centro de control usa Liquid Glass (`materials`). Lo demás sale de la guía de usuario de Apple TV 4K.

- `[A]` («Navigate Control Center») Se abre manteniendo pulsado el botón TV. Abre en la pestaña de ajustes generales; izquierda y derecha cambian de pestaña; Atrás o TV lo cierran.
- `[A]` («Tab icons in Control Center») Pestañas: perfiles, ajustes generales, Apple Home (cámaras y escenas), Cámara de continuidad, llamada de FaceTime (vídeo o audio), reproducción actual y auriculares. Varias solo aparecen cuando su función está activa.
- `[A]` («Adjust general settings in Control Center») La pestaña general da apagado, Wi-Fi o Ethernet, No molestar, salida de audio, temporizador de reposo, mando de juego, accesibilidad, controles parentales y Buscar.
- `[A]` («Status icons in Control Center») Los indicadores de estado van «in the upper-right corner of the Home Screen when Control Center isn’t active»: perfil actual, cámara o micrófono, restricciones, temporizador de reposo, batería baja del mando, FaceTime, audio en reproducción, auriculares Bluetooth, mando de juego, VPN y sin conexión.
- `[A]` («Set notification preferences») Las notificaciones aparecen con un distintivo de icono de app en la esquina superior derecha del Home.
- `[A]` («Switch between apps») Doble pulsación del botón TV: las apps abiertas aparecen como ventanas en fila; Atrás sale sin cambiar.
- `[A]` («Activate or change the screen saver») Salvapantallas tras un tiempo de inactividad configurable, o a mano: en el Home, ir al icono más a la izquierda de la fila superior y pulsar Atrás. Tipos: Aerials, recuerdos y pases de diapositivas, retratos con reloj y Snoopy. Con el salvapantallas activo, derecha pasa al siguiente y el centro lo cierra.
- `[A]` («Switch the current profile», newsroom de junio de 2025) Opción de mostrar la pantalla de perfiles al despertar el Apple TV.
- `[A]` («Change appearance settings») Apariencia clara, oscura o automática, y tamaño de elementos Default o Large.
- Apple no publica este valor: tamaño, columnas, radios y animación de los mosaicos del Centro de control.

### Mando

- `[A]` (`remotes`) Gestos estándar para acciones estándar. El foco se mueve siempre en la dirección del gesto.
- `[A]` (`remotes`) Tabla de botones: superficie táctil (deslizar) navega y cambia el foco; superficie (pulsar) activa; Atrás vuelve a la pantalla anterior o sale al Home; Reproducir/Pausa controla la reproducción.
- `[A]` (`remotes`) Atrás abre el padre de la pantalla actual, que no siempre es la anterior; en el nivel superior de una app el padre es el Home. Mantener Atrás lleva al Home desde cualquier sitio.
- `[A]` (`remotes`) Distinguir pulsar (intencionado) de tocar (puede ser accidental). Deslizar tiene inercia: empieza rápido y frena según la fuerza.
- `[A]` (guía de usuario, «Navigate») Se navega pulsando arriba, abajo, izquierda o derecha en el anillo del clickpad, o deslizando. Mantener una dirección del anillo desplaza rápido. «A highlighted app or item expands slightly».
- `[A]` (guía de usuario, «Adjust remote settings») El clickpad admite «Click Only», que desactiva el seguimiento táctil. El botón TV puede abrir la app Apple TV o el Home.
- `[A]` (`remotes`) Mandos compatibles: botones de guía y de página arriba y abajo para la guía de programación.
- `[V]` (WWDC16 210, espejo) Seleccionar sobre un cartel abre el detalle; Reproducir/Pausa sobre el mismo cartel salta directo a la reproducción.

### Teclado y búsqueda

- `[A]` (`virtual-keyboards`, tvOS) Con el Siri Remote el teclado es lineal. «A grid keyboard screen appears when people use devices other than the Siri Remote».
- `[A]` (`text-fields`, `managing-accounts`) Minimizar la entrada de texto; preferir botones; para datos largos, usar otro dispositivo. Para correo, el teclado de correo con direcciones recientes.
- `[A]` (`search-fields`, tvOS) La pantalla de búsqueda es un teclado con los resultados debajo. Dar sugerencias populares, contextuales y búsquedas recientes.
- `[A]` (`searching`, `search-fields`) En apps con pestañas como Apple TV, la búsqueda es una pestaña propia, con categorías visibles antes de escribir.
- `[A]` (`digit-entry-views`) Entrada de dígitos a pantalla completa con teclado numérico, título e indicación; campo seguro (asteriscos) para datos sensibles.
- `[A]` (`segmented-controls`, tvOS) Los segmentos se seleccionan al recibir foco, no al pulsar; no poner otros elementos enfocables cerca.

### Alertas y hojas

- `[A]` (`alerts`) Título, texto informativo opcional y hasta tres botones. En tvOS no admiten campo de texto. Atrás (Menu) cancela.
- `[A]` (`alerts`) Usarlas poco. No para informar sin acción. No para acciones destructivas comunes y reversibles. No al arrancar.
- `[A]` (`alerts`) Títulos de botón de una o dos palabras con verbo; «Cancel» siempre con ese nombre; el botón más probable arriba en una pila; estilo destructivo solo para lo que la persona no eligió de forma deliberada; Cancel nunca como botón por defecto.
- `[A]` (`sheets`) En tvOS una hoja es siempre modal. `[A]` (`modality`) En tvOS se espera un botón para cerrar dentro de la vista.
- `[A]` (`popovers`, `boxes`) No existen en tvOS. `[A]` (`action-sheets`) Disponibles, sin consideraciones propias.
- `[A]` (`page-controls`, tvOS) Solo en colecciones de páginas a pantalla completa.

### Carga y arranque

- `[A]` (`launching`) «Launch instantly». La pantalla de arranque debe ser casi idéntica a la primera pantalla, sin texto ni marca; en tvOS es estática. Restaurar el estado anterior, incluida la posición de desplazamiento.
- `[A]` (`loading`) Mostrar algo cuanto antes (marcadores de posición) y sustituirlo al llegar el contenido.
- `[A]` (`progress-indicators`) Preferir progreso determinado; mantener el indicador en movimiento; no pasar de circular a barra; ubicación constante.
- `[A]` (`playing-video`) Al pasar de la app TV a otra app la pantalla funde a negro y no se muestra la pantalla de arranque.
- `[A]` (`playing-video`, tvOS) Para superposiciones interactivas sobre vídeo, esperar al menos 0,5 s antes de pausar y mostrarlas.
- Apple no publica este valor: forma y duración de la transición de apertura de una app desde el Home.

### Sonido

- `[A]` (`playing-audio`, tvOS) «the system plays audio only when people initiate it». tvOS no acompaña con sonido las alertas ni las notificaciones.
- `[V]` (WWDC17 224, espejo) «users expect a sound to be played when focus movement occurs». UIKit modifica cada sonido de foco: movimientos rápidos suenan más bajo y lentos más alto; el sonido se panea a izquierda o derecha según la posición del elemento en pantalla; los elementos grandes suenan más graves y los pequeños más agudos.
- `[V]` (WWDC17 224, espejo) Existe un ajuste del sistema para apagar los sonidos de navegación; una app no debe sonar si la persona los apagó.
- `[V]` (WWDC17 224, espejo) VoiceOver emite un sonido cuando se intenta mover el foco hacia donde no se puede.
- `[V]` (WWDC17 803) Los sonidos de transición de Apple TV buscan sensación de profundidad y de movimiento, con dos sonidos distintos para avanzar y retroceder. Un buen sonido de interfaz se usa poco y a volumen muy inferior al de una notificación.
- `[V]` (WWDC19 211) Las previsualizaciones de Top Shelf no fuerzan el sonido.

### Accesibilidad

- `[A]` (`accessibility`) Texto ampliable al menos un 200 %. Contraste y tamaños de control: véanse «Color» y «Layout».
- `[A]` (`accessibility`) Evitar elementos que se cierran solos por tiempo. No autorreproducir audio o vídeo sin controles. Respetar Reducir movimiento y la atenuación de luces intermitentes.
- `[A]` (guía de usuario) Ajustes de tvOS: texto en negrita, tamaño de texto, tamaño de elementos, diferenciar sin color, filtros de color, sensibilidad a la luz, reducir punto blanco, atenuar luces intermitentes, Hover Text, VoiceOver y Switch Control.
- `[A]` (`right-to-left`) Sin consideraciones propias de tvOS: invertir la disposición en idiomas de derecha a izquierda.
- `[A]` (`designing-for-tvos`) La gente está a 8 pies o más (unos 2,4 m) de la pantalla; legibilidad «from across the room».

## Diferencias entre HOME y la guía

Viabilidad en la TCL 55Q5K (GPU PowerVR BXE-4-32, mando con cruceta, interfaz a 1080p). Lo que se dice de Android es `[I]` y debe comprobarse en el firmware antes de implementarlo.

| Aspecto | Qué especifica Apple | Qué hace HOME hoy | Diferencia | Viabilidad |
|---|---|---|---|---|
| Rejilla de 5 columnas | 320 pt + 40 pt, margen 80 pt (213,3 + 26,7 y 53,3 u) | 212 + 28, margen 54 | Ninguna apreciable | Hecho |
| Separación vertical de filas | Mínimo 100 pt (66,7 u) entre filas sin foco, más aire si hay título | No comprobado en este estudio | Verificar | Viable |
| Zona segura vertical | 60 pt (40 u) arriba y abajo | No comprobado; la cápsula de estado y el plato bajo deben respetarla | Verificar | Viable |
| Columnas del Home | Seis iconos en la fila superior; cinco con tamaño Large | Cinco | HOME equivale al modo Large | Viable (173,3 u por icono con 6) |
| Proporción del icono | 5:3 (800×480 px) | 16:9 (banner de Android TV) | Los banners de Android vienen en 16:9 | Viable con coste: recortar a 5:3 corta arte y logotipos |
| Escala de foco | Sin cifra; 1,09 a 1,14 deducible `[I]` | 1,12 | Dentro del rango | Hecho |
| Duración y curva de foco | Sin cifra; enfocar más rápido que desenfocar; más rápido si el foco corre | 260 ms fijos, misma curva | Falta la asimetría y la aceleración con repetición de tecla | Viable |
| Rebote al enfocar | Sin cifra para tvOS; WWDC18: rebote solo si el gesto tiene inercia | Pequeño rebote | Una tecla de cruceta no tiene inercia `[I]` | Viable (probar sin rebote) |
| Brillo y paralaje | Brillo especular e inclinación que siguen al dedo; capas que se separan | Banda de luz una vez e inclinación breve opuesta al sentido de avance | No hay seguimiento continuo ni capas; en tvOS la inclinación sigue al gesto | Seguimiento: no viable (mando sin superficie táctil). Capas: no viable (los banners de Android son planos). El sustituto actual es razonable |
| Nombre de la app | Título oculto hasta el foco; se aparta al crecer la imagen | Nombre solo bajo el banner enfocado | Ninguna | Hecho |
| Estado de inactividad | Lo no enfocado se atenúa y lo enfocado se expande | No existe | Falta | Viable; la atenuación debe ir en el color por vértice para no añadir pasadas |
| Estado de pulsación | Vuelve al tamaño sin foco, respuesta inmediata, aspecto hundido | Lanza la app con ampliación desde el banner | Falta la respuesta al pulsar | Viable |
| Estado no disponible | Sin sombra, translúcido, bajo contraste, no enfocable | No comprobado | Verificar | Viable |
| Foco en el Centro de control | Blanco opaco con texto negro, más grande y con sombra más lejana | Mosaico blanco con contenido oscuro | Coincide en color; verificar escala y sombra | Viable |
| Vidrio | Refracción y desenfoque en tiempo real; lente; brillo especular; se materializa modulando la lente | Copia predifuminada, lente y brillo en el borde, sin desenfoque en tiempo real | Estático; no refracta contenido que se mueve | Tiempo real: no viable (GPU débil). Apple también lo apaga en equipos antiguos |
| Atenuación bajo vidrio claro | Capa oscura del 35 % si el fondo es claro | «Atenuación ligera» en el Centro de control | Verificar el valor con los tres fondos | Viable |
| Vidrio sobre vidrio | Evitarlo | Menú de vidrio oscuro sobre el plato de favoritos | Posible caso `[I]` | Viable: menú más opaco, sin lente |
| Accesibilidad del vidrio | Reducir transparencia, Aumentar contraste, Reducir movimiento | No hay ajustes | Falta | Viable: Android expone la escala de animación y el texto de alto contraste |
| Regulador de vidrio | De ultraclaro a tintado (versión 27) | Tinte fijo | Falta | Viable: es un parámetro del shader |
| Fondo al desplazar | Bajo el pliegue la imagen entera se difumina | El fondo se oscurece | Oscurece en vez de difuminar | Viable con coste: fundir a la copia predifuminada es un cuadro completo, que el desplazamiento ya paga |
| Fondo del Home | Última imagen de Top Shelf, difuminada | Wallpaper propio a pantalla completa | Decisión de HOME; no hay Top Shelf | No aplica |
| Top Shelf | Carrusel a pantalla completa, fila por secciones o banner; reserva estática 2320×720 pt volteada y difuminada | No existe | Falta entera | Con coste o no viable: leer los canales de otras apps exige en Android un permiso reservado al sistema `[I]`. Una reserva estática sí es viable |
| «Seguir viendo» | Fila por secciones para reanudar | No existe | Falta | Misma limitación de permisos `[I]` |
| Menú contextual | Pulsación larga; pocos elementos; ocultar lo no disponible; nace del elemento | Pulsación larga; Mover, Quitar de Inicio, Info | Acorde; verificar que nace del banner y que se ocultan las acciones no aplicables | Viable |
| Orden del menú | Destructivos al final (regla escrita para iOS) | Quitar va en medio | Menor | Viable |
| Reordenar | Cualquier app, arrastrando; confirmar con el centro | Solo favoritos, con temblor | Falta en el resto | Viable con coste: el temblor repinta la fila y baja a unos 30 cuadros |
| Carpetas | «Move to > New Folder»; se borran al vaciarse | No existen | Falta | Viable con coste (modelo de datos y vista nueva) |
| Centro de control: apertura | Mantener pulsado el botón TV; Atrás o TV cierran | OK sobre la cápsula del reloj; tecla Home | Otro gesto | Con coste: depende de que el servicio de accesibilidad reciba la pulsación larga de Home `[I]` |
| Centro de control: estructura | Pestañas; la general con apagado, red, No molestar, audio, temporizador, mando, accesibilidad, búsqueda | Una vista de mosaicos en 2 columnas | Sin pestañas; faltan temporizador, No molestar y apagado | Mosaicos: viable. Apagado y temporizador: sin API confirmada en esta tele |
| Indicadores de estado | Esquina superior derecha: sin conexión, audio, auriculares, mando, temporizador, VPN | Solo reloj | Faltan iconos de estado | Viable (red, Bluetooth y audio son consultables) |
| Cambio de apps | Doble pulsación de TV; ventanas en fila | No existe | Falta | No viable o con coste: las miniaturas de tareas recientes no están al alcance de un launcher normal `[I]` |
| Salvapantallas | Tras inactividad; o Atrás en el primer icono de la fila superior | Mosaico «Modo ambiente» | Falta el atajo con Atrás | Viable |
| Perfiles | Pestaña de perfiles; pantalla de perfiles al despertar | No existe | Falta | Fuera de alcance |
| Tipografía | SF Pro; 23 a 76 pt; Medium por defecto | Otra sans; escala 16, 18, 20, 22, 28, 40 u (24 a 60 pt) | El mínimo cumple (24 ≥ 23 pt); los escalones y el peso no siguen la tabla | Viable; SF Pro no se puede distribuir `[I]` |
| Texto grande | Tamaño de texto del sistema; menos columnas | No hay | Falta | Viable: leer la escala de fuente de Android |
| Sonido de foco | Esperado; volumen según velocidad; paneo según posición; tono según tamaño; respeta el ajuste | Clic del sistema | Sin paneo ni variación | Viable |
| Teclado y búsqueda | Teclado en rejilla para mandos que no son el Siri Remote; sugerencias; resultados debajo | No existe | Falta | Viable con coste |
| Alertas | Solo para lo irreversible y poco común | No comprobado; quitar un favorito es reversible | No debe pedir confirmación | Viable |
| Notificaciones | Distintivo arriba a la derecha | No existe | Falta | Con coste: exige acceso a notificaciones concedido por el usuario |
| Arranque | Instantáneo; primera pantalla igual a la de arranque; restaurar estado | No comprobado | Verificar que se restauran el foco y la posición | Viable |
| Apertura de app | Sin especificación pública | Ampliación desde el banner | Sin referencia | Hecho |
| Barra de pestañas | 68 pt de alto a 46 pt del borde | Sin barra; cápsula de estado | El Home de tvOS tampoco tiene pestañas | No aplica; alinear la cápsula a 30,7 u del borde es opcional |
| Idiomas de derecha a izquierda | Disposición invertida; foco inicial arriba a la derecha | No comprobado | Baja prioridad | Viable |

## Qué hacer a continuación

Ordenado por efecto visual o de comportamiento frente a esfuerzo.

1. **Estado de inactividad.** Tras unos segundos sin teclas, atenuar lo no enfocado y ampliar un poco más el banner enfocado; cualquier tecla lo deshace. Regla: `images` («out-of-focus content dims and the focused element expands»). Apple no da tiempo ni cantidad: hay que elegirlos a ojo. Tamaño: pequeño.
2. **Foco asimétrico y que siga el ritmo.** Enfocar más rápido que desenfocar, y acortar ambas cuando llegan teclas repetidas. Probar la entrada sin rebote. Regla: `UIFocusAnimationCoordinator` y WWDC18 803. Tamaño: pequeño.
3. **Estado de pulsación.** Al pulsar OK, el banner vuelve al tamaño sin foco con aspecto hundido antes de la ampliación de apertura; lo mismo en los mosaicos del Centro de control. Regla: `focus-and-selection` (estado *highlighted*). Tamaño: pequeño.
4. **Difuminar bajo el pliegue.** Al desplazar a las filas siguientes, fundir el fondo a la copia predifuminada además de oscurecerlo, y volver a nítido arriba. Regla: muestra «media catalog» («the entire image blurs») y `materials`. Medir cuadros antes de dejarlo. Tamaño: pequeño a mediano.
5. **Accesibilidad de movimiento y vidrio.** Con las animaciones de Android apagadas: sin rebote, sin inclinación, sin banda de luz, fundidos en vez de desplazamientos. Añadir en ajustes de HOME «reducir transparencia» (plato y mosaicos más opacos) y «aumentar contraste» (borde contrastado). Regla: `accessibility` y WWDC25 219. Tamaño: mediano.
6. **Sonido de foco propio.** Volumen menor cuando las teclas llegan seguidas, paneo estéreo según la columna, tono más grave en elementos grandes, y silencio si los sonidos del sistema están apagados. Regla: WWDC17 224 y WWDC17 803. Tamaño: mediano.
7. **Menú contextual como en tvOS.** Que nazca del banner, que oculte las acciones que no aplican y que deje «Quitar de Inicio» al final; no pedir confirmación al quitar. Regla: `context-menus`, `alerts`, WWDC25 219. Tamaño: pequeño.
8. **Escala tipográfica de tvOS.** Llevar los tamaños a 15,3, 16,7, 19,3, 20,7, 25,3, 32, 38 y 50,7 u con peso Medium y el tracking de la tabla; nombre de app en Caption 1 o Body. Regla: `typography`. Tamaño: pequeño.
9. **Atrás en el primer favorito abre el modo ambiente.** Regla: guía de usuario, «Activate or change the screen saver». Tamaño: pequeño.
10. **Reglas de vidrio.** Fijar la atenuación bajo los mosaicos en el 35 % cuando el fondo es claro, quitar la lente del menú cuando cae sobre el plato, y añadir un regulador de tinte en ajustes. Regla: `materials`, WWDC25 219, nota de prensa de septiembre de 2026. Tamaño: pequeño.
11. **Indicadores de estado junto al reloj.** Sin conexión, Bluetooth con auriculares, audio en reproducción y mando de juego, a la izquierda de la hora. Regla: guía de usuario, «Status icons in Control Center». Tamaño: mediano.
12. **Tamaño de elementos y texto grande.** Ajuste Predeterminado (6 columnas de 173,3 u) o Grande (las 5 actuales), y respetar la escala de fuente de Android. Regla: guía de usuario («five icons instead of six») y WWDC26 221. Tamaño: mediano.
13. **Reordenación libre y carpetas.** Mover cualquier app con el patrón «arrastrar con la cruceta, confirmar con OK», y «Mover a > Carpeta nueva». Regla: guía de usuario, «Rearrange apps». Tamaño: grande.
14. **Top Shelf mínimo.** Zona superior que cambia con el favorito enfocado; empezar por la reserva estática (banner de la app volteado y difuminado) y estudiar si el firmware deja leer canales de vista previa. Regla: `top-shelf`. Tamaño: grande.
15. **Búsqueda.** Pantalla con teclado en rejilla, sugerencias y recientes, y resultados de apps debajo. Regla: `search-fields`, `virtual-keyboards`. Tamaño: grande.

## Lo que no pude consultar

- Ningún vídeo ni imagen. De las figuras del HIG solo leí el texto alternativo, así que no hay medidas tomadas de capturas.
- Las plantillas de Apple Design Resources para tvOS (archivos de Sketch, Figma y Photoshop). Es donde estarían los tamaños reales de los iconos del Home, radios de esquina y sombras.
- La transcripción de la Keynote de WWDC25, donde se presentó tvOS 26: la página no la incluye.
- Las sesiones de tvOS de WWDC16 y WWDC17 en el sitio de Apple (retiradas). Las leí en un espejo de terceros y solo por palabras clave; no puedo garantizar que el espejo sea fiel.
- Los Tech Talks de Apple TV de 2015 («Designing for Apple TV»): la URL redirige al índice.
- Las páginas del HIG de compras dentro de la app y de autenticación de proveedor de TV: los slugs que probé dan 404.
- Las sesiones enlazadas desde `focus-and-selection` (puntero de iPadOS y visionOS): no las leí por no tratar de tvOS.
- Las secciones de iOS, iPadOS, macOS, visionOS y watchOS de las páginas del HIG, y el texto general de las páginas marcadas como «solo sección de tvOS» en la tabla de fuentes.
- Varias transcripciones largas solo se leyeron por palabras clave (WWDC18 803, WWDC19 211, WWDC23 10158, WWDC17 803 y las dos Platforms State of the Union).
- La página de la guía de usuario sobre instalar y borrar apps, y las de perfiles en detalle.
- Una descripción oficial del Home de tvOS 26 o 27 con medidas. Apple no la publica; lo que este documento dice del Home viene de la guía de usuario y de una muestra de código.
- El estado real del código de HOME: la columna «Qué hace HOME hoy» usa la descripción recibida y `design.md`; las celdas «No comprobado» están sin verificar. Nada de la columna de viabilidad se probó en la tele.

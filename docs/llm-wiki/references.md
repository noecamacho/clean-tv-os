# Referencias, procedencia y licencias

| Referencia | Papel en HOME | Estado |
| --- | --- | --- |
| [douglaspands/lume-tv-launcher](https://github.com/douglaspands/lume-tv-launcher) | Base de código Rust/Android. | Fork desde `2e7421d`; remoto local `upstream` solo para lectura. Código GPL-3.0-or-later (`LICENSE`). |
| [tinulica/tv-home](https://github.com/tinulica/tv-home) | Referencia de aspecto y experiencia Apple TV para TCL. | **Ningún archivo fuente suyo se incorporó**. Revisión consultada: `1f512fca8a4d01cdf89bdf987f6f245eec2e4cd5`. No había licencia general en la raíz de ese checkout; revisar los derechos antes de copiar código o assets. |
| [AppleInsider: tvOS 26 hands-on](https://appleinsider.com/articles/25/06/10/tvos-26-hands-on-sleek-liquid-glass-redesign-new-control-center-and-more) | Referencia pública del top bar y Centro de control. | Solo observación visual. No se copiaron imágenes ni software de Apple. |
| [Blackphenixlord/apple-tv-launcher](https://github.com/Blackphenixlord/apple-tv-launcher) | Alternativa mencionada al iniciar la exploración. | No se incorporó ni se ha evaluado para esta variante. Comprobar existencia, licencia y compatibilidad antes de usar. |
| `crates/lume-core/assets/wallpaper/` | Fondos de playa generados para esta variante. | Revisar `docs/wallpaper.md` y los metadatos de los QOI antes de redistribuir. |

Para consultar el código de TV Home en otra máquina sin mezclarlo con el build:

```sh
git clone https://github.com/tinulica/tv-home.git /tmp/tv-home-reference
git -C /tmp/tv-home-reference checkout 1f512fca8a4d01cdf89bdf987f6f245eec2e4cd5
```

El nombre y marca Lume no forman parte de la identidad visible de HOME. El paquete y los nombres internos persisten por compatibilidad. El código base conserva su licencia y avisos. No atribuyas a TV Home una implementación que aquí se desarrolló sobre Lume.

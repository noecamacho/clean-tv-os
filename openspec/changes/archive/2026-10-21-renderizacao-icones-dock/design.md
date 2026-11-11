# Design: Carregamento em Alta Resolução e Consistência dos Ícones no Dock

## 1. Carregamento em Alta Resolução no Android (`LumeBridge.java`)

### Contexto e Justificativa
Em TVs com telas de menor densidade física (como Smart TVs 720p em `tvdpi`/213 dpi), o método padrão `pm.getActivityIcon` / `pm.getApplicationIcon` carrega assets reduzidos (~48px ou 64px). Quando esticados para a célula de 128x128 do dock, os ícones perdem definição e ficam borrados.

Além disso, para ícones com transparência, o dock mantém a base geométrica correspondente à forma configurada (quadrado arredondado ou círculo), garantindo consistência visual com os demais aplicativos de forma 100% universal.

### Implementação
1. **Preferência por alta resolução com limite estrito de tentativas:**
   - Consulta primeiro o ícone da atividade via `pm.getActivityInfo` e busca o drawable com `DisplayMetrics.DENSITY_XXHIGH` (480 dpi, fornecendo assets nativos de ~144px a 192px).
   - Se a atividade não declarar recurso próprio, tenta o ícone da aplicação também em `DENSITY_XXHIGH`.
   - Caso o app não possua alta resolução ou ocorra erro, cai diretamente no `pm.getActivityIcon` / `pm.getApplicationIcon` padrão, com limite estrito de 2 tentativas direcionadas.

2. **Filtragem suave e anti-aliased:**
   - Em `drawWholeIcon`, aplica `canvas.setDrawFilter(new PaintFlagsDrawFilter(0, Paint.ANTI_ALIAS_FLAG | Paint.FILTER_BITMAP_FLAG))` e `drawable.setFilterBitmap(true)`.
   - A redução de resolução para a textura de 128x128 pixels ocorre com filtragem bilinear suave e sem serrilhados.

3. **Invalidação de Cache:**
   - Incremento de `DOCK_CACHE_VERSION` para `"d5"` para forçar a regeneração de todas as texturas em alta resolução.

---

## 2. Consistência Geométrica no Rust (`crates/lume-core/src/app/dock.rs`)

O preenchimento sólido `p.list.fill(*rect, radius(rect), Paint::Solid(item.color))` é mantido de forma universal para todos os favoritos, garantindo que qualquer aplicativo com transparência preencha os cantos com a geometria do dock (`radius(rect)`), acompanhando os demais apps sem nenhuma condicional específica de aplicativo.

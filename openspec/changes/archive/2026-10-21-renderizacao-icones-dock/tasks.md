# Tasks

## 1. Carregamento e Renderização de Ícones no Java

- [x] 1.1 Em `android/app/src/main/java/app/lume/tv/LumeBridge.java`, estender `loadIcon` para preferir ícones em `DENSITY_XXHIGH` com limite de 2 tentativas e fallback limpo para o padrão
- [x] 1.2 Em `drawWholeIcon`, aplicar `PaintFlagsDrawFilter` com `ANTI_ALIAS_FLAG` e `FILTER_BITMAP_FLAG` e `setFilterBitmap(true)` para suavização bilinear na célula de 128x128
- [x] 1.3 Incrementar `DOCK_CACHE_VERSION` para `"d5"` em `LumeBridge.java` para garantir regeneração dos ícones em alta resolução

## 2. Consistência Geométrica e Validação de Testes

- [x] 2.1 Em `crates/lume-core/src/app/dock.rs`, manter o preenchimento de fundo `p.list.fill` universal para todos os favoritos, assegurando que ícones com transparência acompanhem a forma dos demais
- [x] 2.2 Validar suíte com `cargo test --workspace`

## 3. Verificação em Dispositivo e Conformidade

- [x] 3.1 Gerar build de release pelo container com `scripts/container-build.sh release`
- [x] 3.2 Instalar o APK na TV (AIWA) e verificar via `screencap` a definição e formato consistente de todos os favoritos
- [x] 3.3 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume); sem novidade, registrar "Sobre revisado, sem mudança"

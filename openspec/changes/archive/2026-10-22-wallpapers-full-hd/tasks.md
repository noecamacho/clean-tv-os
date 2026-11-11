# Tasks

## 1. Base de medida

- [x] 1.1 Medir na TCL e na AIWA, com o build da `main`: retorno a quente, abertura a frio, tempo de descompressão do papel de parede, troca de papel de parede no editor (quadros de 33 ms ou mais) e PSS em primeiro e segundo plano; registrar em `docs/device-notes.md` como base e verificar que a nota cita as TVs só pelo nome

## 2. Masters e gerador

- [x] 2.1 Receber do autor o diretório dos masters novos, copiar para `tools/wallpaper/masters/` e registrar em `docs/wallpaper.md` a origem, o autor e a licença de cada um; verificar que cada papel de parede do catálogo tem master e registro
- [x] 2.2 A Aurora passa a ser um master 4K do autor (`masters/aurora.png`) e o `aurora06.py` sai; verificado que o gerador, rodado duas vezes, gera o mesmo arquivo byte a byte
- [x] 2.3 Reescrever o `generate_wallpapers.py` em 16 bits (calibração no master, redução por média para 1920x1080, dither só no fim) e verificar com `uv run --with pillow tools/wallpaper/generate_wallpapers.py` que os testes de contraste de `lume-core` e `cargo test -p lume-desktop --test contrast` passam
- [x] 2.4 Adicionar ao gerador a recusa de master menor que 1920x1080 e o detector de falso Full HD, calibrar o limiar com os masters antigos de 720p (devem falhar) e com a Aurora em 4K (deve passar), e registrar o limiar em `docs/wallpaper.md`
- [x] 2.5 Se o catálogo mudar de quantidade, nomes ou temas, atualizar o enum dos papéis de parede, as traduções (`docs/translations.md`), as miniaturas e o requisito do catálogo, e verificar com `cargo test --workspace` e `openspec validate wallpapers-full-hd --strict`

## 3. Formato guardado

- [x] 3.1 Medir, com 2 masters reais já gerados, o tamanho e o tempo de decodificação de deflate, PNG e QOI na TCL e na AIWA; escolher pelo critério do `design.md`, decisão 3, e registrar a medida e a escolha no `design.md` e em `docs/device-notes.md`
- [x] 3.2 Gerar os papéis de parede no formato escolhido e adicionar a crate de decodificação, se for nova, e verificar com `cargo test -p lume-core wallpaper` que cada arquivo decodifica para 1920x1080

## 4. Uma cópia no APK

- [x] 4.1 Tirar os papéis de parede do `build.rs` da `lume-core` e fazer a `lume-core` receber os bytes por uma função de leitura da plataforma (desktop lê do repositório), e verificar com `cargo test --workspace` e com `cargo run -p lume-desktop -- --size 1920x1080 --screenshot home h.png`
- [x] 4.2 Levar os papéis de parede para os `assets/` do APK (`android/app/build.gradle`, `scripts/container-build.sh`) e ler pelo `AAssetManager` na `lume-app`, e verificar com `scripts/container-build.sh release` e `unzip -l` que cada um aparece uma vez em `assets/` e nenhum dentro de `lib/`
- [x] 4.3 Fazer o `scripts/verify-apk.py` falhar com papel de parede dentro de biblioteca nativa ou repetido, além do limite de 40 MB, e verificar com `uv run scripts/verify-apk.py dist/<apk>`

## 5. Imagem no tamanho do buffer

- [x] 5.1 Implementar em `render/gles2.rs` a redução por FBO com filtro de área e dither para buffers menores que 1920x1080, apagando a textura de origem depois, e verificar com os cenários de fidelidade em 720p e de linhas finas em `tools/lume-desktop/tests/wallpaper.rs`
- [x] 5.2 Atualizar o teste de fidelidade em 1920x1080 para comparar com a imagem do catálogo (erro médio de no máximo 1/255) e adicionar o teste de faixas de cor em 1280x720 e 1920x1080, e verificar com `cargo test -p lume-desktop --test wallpaper`
- [x] 5.3 Verificar com teste que a cobertura em proporções diferentes de 16:9 não deixa faixas vazias

## 6. Preparação fora do laço e cache

- [x] 6.1 Mover a decodificação para uma thread de trabalho, com o laço só enviando os pixels prontos, e verificar com teste que o primeiro quadro completo depois do splash já tem o papel de parede
- [x] 6.2 Fazer a troca no editor manter o papel anterior até o novo estar pronto e descartar as escolhas velhas, e verificar com teste do editor que, passando por vários papéis, só o último é preparado
- [x] 6.3 Implementar o cache do papel de parede pronto no diretório de cache do app (nome com identificador, hash do asset e tamanho do buffer; só o ativo fica; arquivo inválido é refeito), e verificar com testes de unidade os casos de acerto, troca, mudança de tamanho e cache ausente
- [x] 6.4 Medir na TCL e na AIWA a leitura do cache contra a decodificação do asset; se o cache não for mais rápido, rever o requisito "Papel de parede pronto ao acordar" no mesmo change, e registrar a medida em `docs/device-notes.md`

## 7. Documentação e specs

- [x] 7.1 Reescrever `docs/wallpaper.md` (masters, pipeline em 16 bits, formato, assets, redução por FBO, cache, testes) e verificar que os comandos do documento rodam como escritos
- [x] 7.2 Alinhar com o autor e atualizar a decisão fixa "Papel de parede" do `CLAUDE.md` com a origem dos novos masters, e verificar que `docs/wallpaper.md` e o `CLAUDE.md` dizem o mesmo

## 8. Verificação nas TVs

- [x] 8.1 Instalar na TCL e na AIWA e medir de novo tudo o que a tarefa 1.1 mediu; verificar que o retorno a quente e a abertura a frio ficam no máximo 10 ms acima da base, que a troca no editor não tem quadro de 33 ms ou mais, que a PSS fica em até 64 MB e 20 MB, e registrar em `docs/device-notes.md`
- [x] 8.2 Capturar com `adb exec-out screencap -p` cada papel de parede na TCL e na AIWA e verificar a olho, no painel, que não há faixas de cor, blocos nem moiré
- [x] 8.3 Rodar `cargo test --workspace && cargo clippy --workspace --all-targets` e `cargo fmt --all --check`, e verificar que ficam limpos
- [x] 8.4 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume): incluir a crate de decodificação nova e a licença, se houver, e a origem das imagens, se a tela citar; sem novidade, registrar "Sobre revisado, sem mudança"; verificar com `cargo run -p lume-desktop -- --screenshot about a.png` (a lista de crates da tela vem do `cargo tree`: a `qoi` (MIT/Apache-2.0) entra sozinha; a tela não cita a origem das imagens: Sobre revisado, sem mudança além da crate)

## Workflow follow-up

- Destino dos masters 4K (60 MB, sem Git LFS) fica para a próxima change: ver "Pendências conhecidas" no `CLAUDE.md`.


- Arquivar com `/opsx:archive` e mesclar por squash na `main` quando o autor pedir.
- Criar a tag `v0.3.0` depois do merge, se o autor quiser.

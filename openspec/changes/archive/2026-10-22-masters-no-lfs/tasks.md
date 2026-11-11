## 1. Origem e licença dos papéis de parede

- [x] 1.1 Perguntar ao autor, para cada um dos 6 papéis, de onde veio (autor, endereço, ferramenta ou script), a data e a licença, e registrar a resposta aqui antes de seguir
  - Resposta (2026-10-22): os 6 são Unsplash License, baixados pelo Gemini; autores e URLs desconhecidos. Decisão do autor: registrar com a lacuna.
- [x] 1.2 Atualizar a tabela de `docs/wallpaper.md` (seção "Origin and license") com a origem e a licença por imagem e corrigir "no photo or art from anyone else" se não for verdade
- [x] 1.3 Corrigir a linha "Papel de parede" de `CLAUDE.md` (Decisões fixas) para o mesmo fato
- [x] 1.4 Se alguma licença não permitir redistribuir sob a GPL-3.0-or-later, parar e combinar com o autor (trocar a imagem ou tirá-la do catálogo)

## 2. Git LFS

- [x] 2.1 Pedir ao autor que instale o `git-lfs` (`sudo dnf install git-lfs`) e rode `git lfs install`; confirmar com `git lfs version`
- [x] 2.2 Criar `.gitattributes` com `tools/wallpaper/masters/*.png filter=lfs diff=lfs merge=lfs -text`
- [x] 2.3 Converter os arquivos atuais sem reescrever o histórico: `git lfs migrate import --no-rewrite --include="tools/wallpaper/masters/*.png"`; conferir com `git lfs ls-files` (6 arquivos) e `git status`
- [x] 2.4 Confirmar que `.github/workflows/*.yml` não precisa de `lfs: true` (`cargo test --workspace` e `scripts/container-build.sh release` não leem os masters)
- [x] 2.5 Documentar o LFS em `docs/wallpaper.md` (substituir "check Git LFS with the author before committing") e no "Como trabalhar aqui" de `CLAUDE.md`, incluindo o aviso sobre clone sem `git-lfs`
- [x] 2.6 Após o push, conferir no GitHub que os masters aparecem como "Stored with Git LFS" e que a CI passa
  - Conferido na PR #16: masters servidos por `media.githubusercontent.com` (LFS) e checks `rust`, `android`, `licenses` e `cla` verdes.

## 3. Limpeza de CLAUDE.md e device-notes

- [x] 3.1 Reescrever o bloco "Estado" de `CLAUDE.md`: remover branches e PRs de changes mescladas e "a tag v0.3.0 sai depois do merge"; dizer só que a `main` está na versão da última tag (`git tag`) e manter as pendências reais; atualizar a pendência dos masters (LFS resolvido)
- [x] 3.2 Atualizar "Orçamentos" (a "Última medida (v0.2.0)") com a medida mais recente de `docs/device-notes.md`, ou marcar a data
- [x] 3.3 Trocar os IPs de `docs/device-notes.md:403` por "TCL 55C715" e "AIWA" (os `192.168.1.50` dos READMEs e de `demo.rs` são exemplos e ficam)
- [x] 3.4 Rodar `rg -n '192\.168\.0\.' .` e conferir que não sobra IP real

## 4. READMEs: requisitos e duas opções de instalação

- [x] 4.1 Em `README.md`, abrir a seção "Install Lume on your TV" com os requisitos (Android TV 9+, mesma rede, adb, depuração ativada, APK) e o bloco "O que é o adb", com o APK indicado só por `https://github.com/douglaspands/lume-tv-launcher`, sem versão
- [x] 4.2 Criar a "Option 1: with an AI assistant" com o prompt pronto (verificar o adb, baixar o APK do repositório, pedir o IP, `adb connect`/`adb devices`, `adb install`, `set-home-activity`, sem `pm disable-user` sem pedido, confirmação antes de comandos extras)
- [x] 4.3 Renomear os 6 passos atuais para "Option 2: manually", tirando a versão do nome do APK no passo 1 e no `adb install`
- [x] 4.4 Replicar as mudanças, com o prompt traduzido, em `README.pt-BR.md`, `README.es.md`, `README.id.md` e `README.ru.md`
- [x] 4.5 Conferir a paridade: mesmas seções, mesma ordem, mesmos comandos e a mesma URL nos 5 arquivos (`rg -c 'github.com/douglaspands/lume-tv-launcher' README*.md` e `rg -n 'adb ' README*.md`)
- [x] 4.6 Mostrar ao autor o texto final do prompt nos 5 idiomas e ajustar conforme a revisão dele antes de fechar a change (sem teste numa TV)

## 5. Fechamento

- [x] 5.1 `openspec validate masters-no-lfs --strict`
- [x] 5.2 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume) e atualizar se a origem ou a licença dos papéis de parede trouxe algo novo (crédito, fonte, licença); sem novidade, registrar "Sobre revisado, sem mudança" (`cargo run -p lume-desktop -- --screenshot about`)
  - Sobre atualizada: entrada "Wallpapers (images): Unsplash License" em `crates/lume-core/src/licenses.rs`; spec `launcher-shell` ajustada.

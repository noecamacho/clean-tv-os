## Why

Os 6 masters 4K dos papéis de parede (60 MB de PNG em `tools/wallpaper/masters/`) vivem direto no Git, sem LFS. O autor informou que as imagens foram baixadas, e não geradas pelos scripts, então não dá para recriá-las sob demanda. Além disso, `CLAUDE.md`, `docs/wallpaper.md` e a spec `wallpaper` afirmam que elas saem de scripts procedurais e que não há arte de terceiros, o que não bate com isso. O `CLAUDE.md` também está defasado: o bloco "Estado" cita branches e PRs já mesclados, "tag v0.3.0 sai depois do merge" e v0.2.x como atuais, mas as tags chegam a v0.4.0. `docs/device-notes.md` guarda IPs reais da LAN. Por fim, a instalação nos READMEs tem só o caminho manual de 6 passos; falta uma opção mais curta, executada por um assistente de IA, e uma explicação do que é o adb.

## What Changes

- Rastrear `tools/wallpaper/masters/*.png` com Git LFS, convertendo os arquivos atuais sem reescrever o histórico (`git lfs migrate import --no-rewrite`); o histórico, as tags e os hashes não mudam.
- Registrar a origem e a licença reais de cada um dos 6 papéis em `docs/wallpaper.md` (informadas pelo autor) e corrigir as afirmações de `CLAUDE.md` e `docs/wallpaper.md` que dizem "scripts procedurais" e "nenhuma foto de terceiros", se não forem verdadeiras.
- Atualizar `CLAUDE.md` (Estado, Orçamentos, Decisões fixas) e tirar as referências de estado a tags; remover os IPs de `docs/device-notes.md`.
- Reorganizar a instalação dos 5 READMEs (en, pt-BR, es, id, ru) em duas opções: (1) por um assistente de IA, com um prompt pronto traduzido em cada idioma, e (2) manual, com os 6 passos atuais. Acrescentar a lista de requisitos (TV Android TV 9 ou superior na mesma rede do computador, adb, depuração ativada, APK) com contexto sobre o que é o adb. O APK é indicado só pelo repositório (`https://github.com/douglaspands/lume-tv-launcher`), sem versão.
- Documentar o LFS no fluxo de trabalho (`CLAUDE.md`, `docs/wallpaper.md`): `git lfs install`, o CI sem `lfs: true` (os masters não entram no build).

## Capabilities

### New Capabilities

- `installation-docs`: o que os READMEs devem trazer para instalar o Lume (requisitos, duas opções de instalação, prompt para assistente de IA, paridade entre idiomas).

### Modified Capabilities

- `wallpaper`: o requisito "Origem das imagens" deixa de afirmar que todos os masters saem de scripts do autor e passa a exigir o registro de origem e licença por imagem, e os masters passam a ser guardados em Git LFS.

## Impact

- Novo `.gitattributes`; os 6 PNGs viram ponteiros LFS no working tree e no índice.
- Docs: `CLAUDE.md`, `docs/wallpaper.md`, `docs/device-notes.md`, `README.md` e as 4 traduções, specs `wallpaper` e `installation-docs`.
- O nome `masters-no-lfs` ficou estreito para o escopo; foi mantido por decisão do autor, que pediu o requisito dos READMEs nesta change.
- Sem mudança de código, de APK, de CI ou de orçamentos. Dependência externa: `git-lfs` instalado na máquina do autor; o GitHub dá 1 GB de armazenamento e 1 GB/mês de banda LFS grátis.
- Os 60 MB já existentes continuam nos blobs do histórico (decisão: não reescrever).
- Pendente do autor: a origem e a licença de cada imagem.

## Context

Os masters (6 PNG, 60 MB) estão em 2 commits de um histórico de 19 e já foram publicados no GitHub; `.git` tem 107 MB, e o pack, 28 MiB. Nada no build os usa (só `tools/wallpaper/*.py` e a documentação). O `git-lfs` não está instalado na máquina de desenvolvimento e não há `.gitattributes`. O autor disse que as imagens foram baixadas, o que contradiz o texto atual sobre origem.

## Goals / Non-Goals

**Goals:** READMEs com requisitos e duas opções de instalação; masters novos e atuais passam a ser objetos LFS; documentação de origem e licença verdadeira; `CLAUDE.md` e `device-notes.md` coerentes com o estado e sem IP.

**Non-Goals:** automatizar a ativação da depuração na TV (é sempre manual); reescrever o histórico, mover as tags ou fazer force-push; gerar os masters sob demanda; mudar o código, o APK ou o CI.

## Decisions

- **`git lfs migrate import --no-rewrite` em vez de reescrever o histórico.** Cria um commit novo que converte os arquivos atuais e o `.gitattributes`, sem mexer em hashes, tags, PRs nem releases. Alternativa descartada: `migrate import` com reescrita, que deixaria o clone leve (economia de cerca de 28 MiB), mas exigiria force-push da `main` e reemissão de v0.3.0 a v0.4.0. O custo público supera o ganho. Consequência aceita: os blobs antigos ficam no histórico.
- **CI sem `lfs: true`.** Os masters não participam do build; o checkout só traz ponteiros, o que poupa banda LFS.
- **Origem registrada por imagem, com dado do autor.** A tabela de `docs/wallpaper.md` ganha as colunas de origem e licença. Eu não deduzo isso: a tarefa pede os dados ao autor. Se alguma imagem tiver licença que não permita redistribuir sob a GPL-3.0-or-later, ela sai do catálogo ou é trocada, e isso vira nova change.
- **Texto de tags no CLAUDE.md.** O "Estado" passa a só dizer a versão mais recente e apontar `git tag`, para não envelhecer de novo.

- **Instalação em duas opções, prompt traduzido.** A Opção 1 (assistente de IA) vem primeiro; a Opção 2 mantém o texto manual. O prompt é traduzido nos 5 idiomas (decisão do autor), para o assistente conversar no idioma do leitor; os comandos e a URL ficam idênticos. Os requisitos e o contexto do adb ficam num bloco antes das duas opções, em cada README. O prompt traz a confirmação humana antes de comandos que alteram a TV (`pm disable-user` fica fora por padrão). O APK é indicado só pelo repositório, sem versão, para o README não envelhecer.
- **Capability nova `installation-docs`.** Não havia spec para os READMEs; o requisito de paridade entre idiomas evita que as traduções divirjam.

## Risks / Trade-offs

- Cota LFS do GitHub (1 GB de armazenamento, 1 GB/mês de banda) → 60 MB cabem folgados; clones novos com LFS baixam 60 MB cada.
- Quem clonar sem `git-lfs` instalado vê ponteiros de texto nos PNGs → documentar; o build não é afetado.
- Os 60 MB antigos permanecem no histórico → aceito.
- Se a licença de alguma imagem for incompatível, a change revela uma pendência maior → a tarefa de registro vem antes do commit do LFS.
- O prompt depende de o assistente do leitor poder rodar comandos e baixar arquivos → a Opção 2 continua disponível; o prompt não é testado numa TV (decisão do autor), só revisado em texto, então pode haver passo que um assistente real execute de forma diferente.
- Cinco cópias do mesmo texto podem divergir → tarefa de conferência de paridade.

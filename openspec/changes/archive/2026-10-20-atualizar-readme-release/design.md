# Design

## Context

Os READMEs têm três trechos que citam a versão: o aviso de status no topo, o passo "Download Lume" (nome do APK) e o exemplo de `adb install` na seção de instalação. Nos dois idiomas, o texto é o mesmo em substância, então a mudança segue espelhada. A seção de releases (`lume-<version>` e `v0.1.0` como exemplo de tag) é orientação de build e fica como está.

## Goals / Non-Goals

**Goals:**
- Os dois READMEs apontam para a v0.3.0 como release atual e dão o nome real do APK (`lume-0.3.0.apk`).
- O texto fica igual nos dois idiomas, sem o aviso de "em desenvolvimento".

**Non-Goals:**
- Não criar a página de release nem anexar o APK (isso é do workflow de release e das pendências do autor: keystore e CI).
- Não mudar o `Cargo.toml`, o `build.gradle` nem o fluxo de versão do build.
- Não reescrever a seção de build nem a de assinatura além da crase quebrada.

## Decisions

- **Aviso de status**: substituir o texto de "não lançada" por uma frase de release atual (versão 0.3.0), com link para a página de Releases. Evita um número que envelhece a cada release: o link leva à mais nova.
- **Nome do APK**: `lume-0.3.0.apk`, seguindo a regra de nome do workflow (tag `v0.3.0` vira `0.3.0`).
- **Exemplos de tag e `dist/`**: ficam genéricos (`v0.1.0`, `lume-<version>`), porque são exemplos de processo, não a versão atual.
- **Escopo do texto**: só as frases que citam versão ou status, sem reformatar o resto. Revisão feita comparando EN e pt-BR linha a linha.

## Risks / Trade-offs

- Se o APK `lume-0.3.0.apk` não estiver anexado à release v0.3.0 no GitHub, o passo de download falha para quem segue o README. Verificar a página de Releases antes de mesclar. Hoje a tag existe no remoto; o anexo não foi conferido nesta proposta.
- Trade-off: o README fica com link para a lista e não com um número fixo de download. Mais estável, mas o usuário precisa escolher a versão mais nova na página.

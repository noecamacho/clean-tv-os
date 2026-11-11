# Proposal

## Why

Os dois READMEs (`README.md` e `README.pt-BR.md`) ainda dizem que a versão 0.1.0 não foi lançada e que não há download. A release atual é a `v0.3.0` (tag no commit `c1da5db`, splash na abertura e no retorno). Quem chega ao projeto não encontra a versão certa nem o nome do APK a baixar.

## What Changes

- Trocar o aviso de status dos dois READMEs: a versão atual é a 0.3.0, já publicada, com link para a página de Releases.
- Atualizar as referências de versão do passo de instalação (`lume-0.1.0.apk` → `lume-0.3.0.apk`, nos dois idiomas, incluindo o exemplo de `adb install`).
- Corrigir o trecho de assinatura de release, que hoje tem uma crase de abertura sobrando e mistura exemplo de versão.
- Manter os exemplos de tag de build (`v0.1.0`, `v0.1.0-rc1`) como exemplos genéricos, sem apontar para a release atual.

Sem mudança de código, de comportamento nem de specs.

## Capabilities

### New Capabilities

Nenhuma.

### Modified Capabilities

Nenhuma. Documentação apenas: `skip_specs: true` no `.openspec.yaml`, porque nenhum requisito muda.

## Impact

- `README.md` e `README.pt-BR.md`: status, passo de download e exemplos de versão.
- Nenhum código, APK, workflow ou spec afetado.
- Premissa: a v0.3.0 é a release a ser divulgada, conforme confirmado pelo autor (a tag existe no remoto `origin`).

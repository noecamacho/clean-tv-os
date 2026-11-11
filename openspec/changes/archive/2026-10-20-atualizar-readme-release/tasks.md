# Tasks

## 1. README em português e inglês

- [x] 1.1 Em `README.md`, trocar o aviso de status ("Version 0.1.0 has not been released yet") por texto de release atual 0.3.0, com link para a página de Releases.
- [x] 1.2 Em `README.pt-BR.md`, fazer o mesmo aviso ("A versão 0.1.0 ainda não foi lançada") em português.
- [x] 1.3 Trocar `lume-0.1.0.apk` por `lume-0.3.0.apk` nos dois idiomas, no passo de download.
- [x] 1.4 Trocar `lume-0.1.0.apk` por `lume-0.3.0.apk` no exemplo de `adb install` (Windows) nos dois idiomas.
- [x] 1.5 Corrigir a crase de abertura sobrando no trecho de assinatura de release do `README.md` (linha de "The version comes from the build…") e revisar o mesmo trecho no `README.pt-BR.md`.

## 2. Conferência

- [x] 2.1 Comparar os dois READMEs linha a linha: mesmas versões, mesmos nomes de arquivo e mesmo sentido.
- [x] 2.2 Conferir que a página de Releases tem a v0.3.0 com o APK `lume-0.3.0.apk` anexado; se não tiver, parar e avisar o autor antes de mesclar.
- [x] 2.3 Rodar `openspec validate atualizar-readme-release --strict`.

## 3. Tela Sobre

- [x] 3.1 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume) e atualizá-la se a change trouxe algo novo; esta change não adiciona componente, fonte, ícone, licença nem versão de código, então o registro esperado é "Sobre revisado, sem mudança". Verificar com `cargo run -p lume-desktop -- --screenshot about`.

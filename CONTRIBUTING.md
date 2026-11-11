# Contribuindo com o lume / Contributing to lume

Obrigado pelo interesse! / Thanks for your interest!

## CLA (Contributor License Agreement)

Antes de aceitarmos o seu primeiro pull request, é preciso assinar o [CLA](CLA.md). O CLA Assistant comenta automaticamente no PR com as instruções: basta responder com a frase indicada. A assinatura vale para todas as contribuições futuras.

O CLA existe porque o lume é GPL-3.0-or-later e o projeto quer manter aberta a possibilidade de uma futura edição Pro fechada. Você continua dono do que escreveu.

*Before we can accept your first pull request you need to sign the [CLA](CLA.md). The CLA Assistant bot comments on the PR with instructions; reply with the phrase it asks for. One signature covers all your future contributions.*

## Como contribuir com traduções / Contributing translations

As traduções ficam em `android/app/src/main/res/values-<qualificador>/strings.xml`, no formato padrão de recursos do Android (por exemplo, `values-es/`, `values-pt-rBR/`). Não é preciso saber Rust.

1. Copie `android/app/src/main/res/values/strings.xml` (inglês) para `values-<qualificador>/strings.xml`.
2. Traduza o texto de cada `<string>` e `<item>` de `<plurals>`, mantendo os marcadores (`%1$s`, `%1$d`) e a ordem das chaves.
3. Se o idioma tiver regras de plural novas, avise no PR (o guia `docs/translations.md` explica).
4. Teste com `cargo run -p lume-desktop -- --locale <tag>`.
5. Abra o PR. Também aceitamos tradução por Weblate/Crowdin quando estiver configurado.

*Translations live in `android/app/src/main/res/values-<qualifier>/strings.xml` in the standard Android resource format. Copy the English file, translate the texts keeping the placeholders, try it with `cargo run -p lume-desktop -- --locale <tag>` and open a PR. See `docs/translations.md`.*

## Código / Code

- `cargo fmt`, `cargo clippy --workspace -- -D warnings` e `cargo test --workspace` devem passar.
- Mudanças de comportamento seguem as specs em `openspec/`.
- Commits pequenos e focados, com mensagem clara.

## Licenças das contribuições

Código: GPL-3.0-or-later. Temas em `themes/`: MIT. Traduções: mesma licença do código.

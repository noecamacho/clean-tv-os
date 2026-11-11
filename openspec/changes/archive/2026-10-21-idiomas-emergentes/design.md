# Design

## Resolução de Idioma e Estrutura de Arquivos

O Lume segue a resolução de idiomas do Android via diretórios de recursos padrão:

```
android/app/src/main/res/
├── values/strings.xml          # Referência (Inglês)
├── values-pt-rBR/strings.xml   # Português do Brasil
├── values-es/strings.xml       # Espanhol (Latino-americano / Geral)
├── values-id/strings.xml       # Indonésio (Bahasa Indonesia)
└── values-ru/strings.xml       # Russo
```

Quando o Android está configurado com variantes regionais (por exemplo, `es-AR` na Argentina ou `es-MX` no México), o resolvedor de idiomas em `crates/lume-core/src/i18n/mod.rs` (`pick_language`) tenta primeiro a variante exata e, se não encontrar, retrocede para o prefixo do idioma (`es`). Dessa forma, `values-es` atende a todos os países de língua espanhola.

## Plurais e CLDR

As regras de plural de cada idioma são validadas em `crates/lume-core/src/i18n/plural.rs`:

1. **Espanhol (`es`)**:
   - Família: `Family::OneIsOne`.
   - Categorias usadas: `one` (apenas para n = 1) e `other` (para os demais valores).
2. **Indonésio (`id`)**:
   - Família: `Family::NoPlural`.
   - Categorias usadas: apenas `other` (não varia com o número).
3. **Russo (`ru`)**:
   - Família: `Family::EastSlavic`.
   - Categorias usadas: `one` (terminados em 1, exceto 11), `few` (terminados em 2-4, exceto 12-14), `many` (terminados em 0 ou 5-9, e 11-14) e `other`.

## Testes de Layout

Para garantir que textos mais longos (especialmente em russo e espanhol) não quebrem o layout ou se sobreponham, as telas serão verificadas via `lume-desktop` com as opções `--locale es`, `--locale id` e `--locale ru`, além do `--pseudo-locale` que expande as strings em 40%.

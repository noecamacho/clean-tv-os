# localization Specification Delta

## ADDED Requirements

### Requirement: Idiomas para mercados emergentes
O lume SHALL incluir traduções completas para espanhol (`es`), indonésio (`id`) e russo (`ru`), correspondendo a todas as chaves do arquivo de referência (`values/strings.xml`) e respeitando as categorias de plural do CLDR de cada idioma.

#### Scenario: Interface em espanhol
- **WHEN** o idioma preferido do Android é espanhol (qualquer variante, como `es-MX`, `es-AR` ou `es-ES`)
- **THEN** os textos da interface aparecem em espanhol (ex.: "Inicio", "Aplicaciones", "Ajustes")

#### Scenario: Interface em indonésio
- **WHEN** o idioma preferido do Android é indonésio (`id` ou `id-ID`)
- **THEN** os textos da interface aparecem em indonésio (Bahasa Indonesia)

#### Scenario: Interface em russo
- **WHEN** o idioma preferido do Android é russo (`ru` ou `ru-RU`)
- **THEN** os textos da interface aparecem em cirílico em russo e respeitam as regras de plural eslavas

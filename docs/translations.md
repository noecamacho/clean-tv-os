# Translating Lume

Lume shows its texts in the language of the Android system. Translations are standard Android resource files, so you can edit them with any text editor, Android Studio, Weblate or Crowdin. You do not need to know Rust.

## Add a language

1. Pick the folder name for your language, using Android's qualifiers:

   | Language | Folder |
   |---|---|
   | Spanish | `values-es` |
   | Brazilian Portuguese | `values-pt-rBR` |
   | German (Switzerland) | `values-de-rCH` |
   | Serbian in Latin script | `values-b+sr+Latn` |

2. Copy the English file to your folder:

   ```
   cp android/app/src/main/res/values/strings.xml android/app/src/main/res/values-es/strings.xml
   ```

   (Create the `values-es` folder first.) The English file is the reference: it lists every text Lume has.

3. Translate the text inside each `<string>` and each `<item>` of a `<plurals>`. Do **not** change the `name` attributes.

4. Build and look at it (see "Try it" below).

That is all: no code changes are needed for a language whose plural rules already exist (see "Plurals").

## Rules for the texts

- **Placeholders** such as `%1$s` (text) and `%1$d` (number) must stay in every translation, so each language can put them wherever the sentence needs them. For example `New: %1$s` can become `%1$s: nuevo`.
- Characters Android treats specially need a backslash: write `\'` for an apostrophe and `\"` for a double quote, or wrap the whole text in double quotes.
- Keep the text short. Texts that do not fit are cut with an ellipsis, so very long translations lose their ending. The pseudo-language below shows how much room there is.
- A text marked `translatable="false"` (such as `app_name`) is not translated.

## Plurals

Texts with a quantity use `<plurals>` with the CLDR categories of your language: `zero`, `one`, `two`, `few`, `many` and `other`. `other` is always required.

```xml
<plurals name="editor_limit_reached">
    <item quantity="one">Límite de %1$d favorito</item>
    <item quantity="other">Límite de %1$d favoritos</item>
</plurals>
```

Which categories a language uses is defined by the CLDR (see the [plural rules table](https://www.unicode.org/cldr/charts/latest/supplemental/language_plural_rules.html)). Lume knows the rules of English and most European languages, Portuguese, French, Russian and Ukrainian, Polish, Czech and Slovak, Arabic, and the languages without plural forms (Japanese, Chinese, Korean, Vietnamese, Thai, Indonesian...).

If your language follows a rule Lume does not know yet, add one line to `family_of` in `crates/lume-core/src/i18n/plural.rs` (or, for a new rule, a new family and its test), and mention it in your pull request. Until then Lume uses the English rule for that language.

## What happens when something is wrong

The build checks every translation against the English file and prints warnings, naming the language and the text:

| Problem | Result |
|---|---|
| A text is missing | Warning; that text shows in English |
| A text does not exist in the English file | Warning; ignored |
| The placeholders do not match the English text | Warning; that text shows in English |
| A `<plurals>` has no `other` item, or an unknown category | Warning; that text shows in English |
| The XML is malformed | The build fails and names the file and line |

So a partial translation always works: missing texts fall back to English.

## Try it

The desktop tool renders the screens without a TV:

```
cargo run -p lume-desktop -- --locale es --screenshot home /tmp/home-es.png
cargo run -p lume-desktop -- --locale es --screenshot apps /tmp/apps-es.png
cargo run -p lume-desktop -- --locale es --screenshot editor /tmp/editor-es.png
```

`--locale` takes a language tag such as `es`, `pt-BR` or `zh-Hans`. The other screens are `settings` and `about`; add `--size 1920x1080` for full HD.

To check that long texts fit, use the pseudo-language, which makes every text 40% longer and adds accents:

```
cargo run -p lume-desktop -- --pseudo-locale --screenshot editor /tmp/editor-pseudo.png
```

Nothing should overlap, and texts that do not fit must end with an ellipsis.

## Which language Lume chooses

Lume reads the user's language list from Android, in order of preference, and uses the first language it has a translation for. For each preference it tries the exact variant first (`pt-BR`), then the language alone (`pt`, which also matches `pt-BR` when that is the only Portuguese available). If nothing matches it uses English.

## Scripts that are not Latin

Names of apps and translated texts are shown in any script, using the fonts of the TV for characters the built-in font lacks. Right-to-left languages (Arabic, Hebrew) and scripts that need complex shaping (such as Devanagari) are shown in logical order without shaping in this version, so they may look wrong. Contributions that improve this are welcome.

See also [`CONTRIBUTING.md`](../CONTRIBUTING.md) for the contributor agreement.

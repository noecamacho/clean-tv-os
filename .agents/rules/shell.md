---
trigger: always_on
description: Prioridade de execução de comandos em shell (zsh)
---

# Shell de Execução

- Sempre que for executar comandos no ambiente (terminal / workspace), utilize o **`zsh`** como prioridade caso esteja instalado (`zsh -c "..."` ou `zsh -lc "..."`), em vez do `bash`.
- O `zsh` carrega o ambiente do usuário (`~/.zshenv`), garantindo que ferramentas como `uv`, Homebrew, Android SDK e Go estejam disponíveis no `PATH`.

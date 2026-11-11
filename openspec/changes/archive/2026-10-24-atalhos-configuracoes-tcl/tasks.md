# Tasks

## 1. Ponte Android (`LumeBridge.java`)

- [x] 1.1 Atualizar `leadsToScreen` em `android/app/src/main/java/app/lume/tv/LumeBridge.java` para rejeitar activities cujo nome termine em `EmptyStubActivity`, permitindo que a cascata ignore stubs vazios da TCL; verificar com compilação do Android.
- [x] 1.2 Atualizar a cascata do atalho `storage` em `openSetting` para priorizar `"com.android.tv.settings.STORAGE"` e a classe `device.StorageSummaryActivity` antes de `ACTION_INTERNAL_STORAGE_SETTINGS`, evitando que a TCL abra a tela de redefinição de dados de fábrica; verificar com compilação do Android.
- [x] 1.3 Atualizar o atalho `home_app` em `openSetting` para priorizar `new Intent(Settings.ACTION_HOME_SETTINGS).setPackage("com.google.android.permissioncontroller")`, garantindo a abertura direta do seletor de aplicativo de início padrão na TCL e na AIWA; verificar com compilação do Android.
- [x] 1.4 Tornar a captura de exceções em `openSetting` abrangente para `Exception` geral, garantindo que falhas de classes inexistentes na ROM da TV (como `ClassNotFoundException` de `SoundActivity` na TCL) não encerrem a execução e acionem o fallback; verificar com compilação do Android.

## 2. Validação nas TVs e Documentação

- [x] 2.1 Compilar o APK de debug/release com `scripts/container-build.sh debug` e verificar que a compilação do Java e Rust passa sem erros.
- [x] 2.2 Instalar o APK na TCL e validar via controle/adb que os atalhos de Armazenamento e App de Início abrem suas respectivas telas funcionais e que os demais atalhos continuam operando normalmente.
- [x] 2.3 Atualizar a seção de notas de dispositivo em `docs/device-notes.md` com os testes e a tabela de resolução de atalhos conferida na TCL.
- [x] 2.4 Revisar a tela Sobre (Configurações do Lume › Sobre o Lume); sem alterações em dependências de terceiros, fontes ou licenças, registrar "Sobre revisado, sem mudança"; verificar com `cargo run -p lume-desktop -- --screenshot about`.

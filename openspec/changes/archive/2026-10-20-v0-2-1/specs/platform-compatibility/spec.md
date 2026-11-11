# Spec Delta

## MODIFIED Requirements

### Requirement: Versões do Android suportadas
O Lume SHALL instalar e funcionar em Android TV 9 (API 28) a Android TV 11 (API 30), e SHALL ser compilado contra um nível de API alvo aceito pela Play Store no momento do release. O botão de entradas e a engrenagem da barra superior SHALL funcionar nas duas pontas dessa faixa.

#### Scenario: Android TV 9
- **WHEN** o APK é instalado num emulador Android TV 9 (API 28)
- **THEN** a instalação é bem-sucedida, todas as telas funcionam e a engrenagem abre as configurações do Android

#### Scenario: Android TV 11
- **WHEN** o APK é instalado na TCL 55C715 ou na AIWA SMART TV (Android TV 11), ou em emulador Android TV 11
- **THEN** a instalação é bem-sucedida, o catálogo lista os apps instalados e o botão de entradas abre o menu de entradas da TV

#### Scenario: Visibilidade da ação de entradas no Android 11
- **WHEN** o Lume consulta o sistema se existe quem atenda a `com.android.tv.action.VIEW_INPUTS` no Android 11
- **THEN** a consulta enxerga a activity de entradas do fabricante, porque o manifesto declara a ação em `<queries>`

#### Scenario: Versão anterior
- **WHEN** alguém tenta instalar o APK em Android 8.1 ou anterior
- **THEN** o sistema recusa a instalação

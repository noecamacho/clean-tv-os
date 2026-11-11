# Proposal

## Why

TVs Android de entrada (ex.: TCL 55C715, 2 GB de RAM, Android TV 9) gastam boa parte da memória com o launcher de fábrica e ficam lentas justamente quando o usuário assiste a streaming em 4K. O **lume** é um launcher novo, escrito em Rust com renderização própria em GPU, que entrega a interface do mockup (`mockup/`) consumindo o mínimo de memória possível — principalmente quando está em segundo plano, disputando RAM com o player de vídeo.

Esta é a primeira release (v0.1.0): ela estabelece a base técnica (renderer, tema em CSS, ciclo de memória) e as telas do mockup que são viáveis para um app comum, sem privilégios de sistema.

## What Changes

- Novo app Android TV "lume" que se registra como tela inicial (HOME) e funciona inteiramente por controle remoto (D-pad).
- **Tela Início**: barra superior (logo, abas Início/Apps, engrenagem de Configurações, relógio), carrossel de destaque com os apps do próprio usuário e a linha "Seus apps favoritos".
- **Tela Apps**: grade com todos os apps instalados que podem ser abertos.
- **Tela Configurações** do lume (engrenagem): Tela Início, Configurações do Android e Sobre o lume.
- **Idiomas**: a interface segue o idioma do Android, com português do Brasil e inglês na v0.1.0. Novos idiomas entram só com um arquivo `strings.xml` no formato padrão do Android, compatível com Weblate/Crowdin.
- **Modo edição** da tela Início: escolher os apps do carrossel e o intervalo de rotação, ordenar/adicionar/remover favoritos, reordenar/ocultar seções e restaurar o padrão. A configuração persiste entre reinícios.
- **Tema "lume"** fiel ao mockup, descrito num subconjunto de CSS e compilado no build. Não há troca de tema na v0.1.0.
- **Adaptação de tela**: o mesmo layout, nítido, em TVs de 720p a 4K e de 30" a 60".
- **Eficiência de recursos** como requisito: metas de memória medidas em primeiro plano e em segundo plano, renderização só sob demanda e liberação dos recursos de GPU quando o lume sai de cena.
- **Distribuição**: APK assinado publicado no GitHub, com o projeto pronto para a Play Store (AAB, requisitos de TV, sem permissão de internet).
- Repositório público sob GPL-3.0-or-later, com `TRADEMARKS.md` (nome e logo não licenciados) e CLA para contribuições, o que preserva a possibilidade de uma futura versão Pro fechada.

Fora de escopo nesta release: linhas "Recomendados" com conteúdo dos streamings (exigem permissões de sistema inacessíveis em TVs de mercado), conteúdo real no carrossel (rede/APIs), troca de temas, versão Pro e qualquer acesso à internet.

## Capabilities

### New Capabilities
- `launcher-shell`: registro como HOME, barra superior, navegação entre abas, relógio, tela de Configurações do lume (Tela Início, Configurações do Android, Sobre) e comportamento dos botões Home/Voltar.
- `localization`: escolha do idioma a partir do Android, formato das traduções, plurais, textos longos e exibição de nomes de apps em qualquer escrita.
- `app-catalog`: descoberta dos apps instalados, detecção de instalações e remoções, identidade visual de cada app (nome e cor), abertura de apps e histórico local de uso.
- `home-screen`: conteúdo da tela Início, com o carrossel de destaque e a linha de apps favoritos.
- `apps-screen`: grade "Todos os apps".
- `home-editor`: modo edição da tela Início e persistência da configuração do usuário.
- `theming`: contrato do tema como subconjunto de CSS e tema padrão "lume" derivado do mockup.
- `display-adaptation`: escala, área segura e resolução de renderização entre 720p e 4K.
- `resource-efficiency`: orçamentos de memória, comportamento em segundo plano, renderização sob demanda e tempo de abertura.
- `platform-compatibility`: versões do Android, arquiteturas, permissões e requisitos de distribuição (GitHub e Play Store).

### Modified Capabilities
<!-- Nenhuma: o projeto não tem specs existentes. -->

## Impact

- **Código novo** (projeto greenfield): workspace Rust (`lume-core`, `lume-app`, parser de CSS do tema), projeto Android/Gradle mínimo com uma ponte Java fina via JNI, tema em `themes/lume/theme.css`.
- **Dependências**: toolchain Rust com alvos Android, Android NDK, `cargo-ndk`, Gradle e um pequeno conjunto de crates (NativeActivity, JNI, bindings GL, rasterização de fontes). Sem frameworks de UI, sem WebView, sem AndroidX/Kotlin.
- **Ativos**: fonte com licença OFL (subset latino estendido, com fallback para as fontes do sistema), conjunto pequeno de ícones de licença permissiva e traduções em `android/app/src/main/res/values*/strings.xml`.
- **Infra**: GitHub Actions para build, assinatura e anexo do APK ao release; keystore guardado em secrets.
- **Dispositivo de referência**: TCL 55C715. Os riscos (botão Home ignorar launchers de terceiros, piso de memória do processo) são validados num spike antes do restante do trabalho.

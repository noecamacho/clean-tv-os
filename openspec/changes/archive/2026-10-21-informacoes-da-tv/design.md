# Design

## Arquitetura de Coleta dos Dados Técnicos

Seguindo o princípio do Lume (**Construção 100% Rust; Java apenas onde o Android exige**), a ficha técnica é lida diretamente do sistema operacional:

```
[ Lume Core (Rust) ]
   ├── OpenGL Context  ──────> glGetString(GL_RENDERER), glGetString(GL_VERSION)
   ├── Linux Filesystem ─────> /proc/cpuinfo (núcleos, modelo CPU), /proc/meminfo (RAM)
   │                    ─────> libc::uname (versão do Kernel Linux)
   │                    ─────> libc::statvfs (armazenamento)
   │                    ─────> ioctl SIOCGIFCONF (IP local e tipo de conexão; exige INTERNET)
   └── Android NDK C API ────> libc::__system_property_get
                                 ├── ro.product.brand / ro.product.model
                                 ├── ro.product.manufacturer / ro.board.platform
                                 ├── ro.build.display.id (firmware da TV)
                                 ├── ro.build.version.release / sdk / security_patch
                                 └── ro.product.cpu.abi
```

Para dados restritos à API do Android (resolução da interface, `Display.getRefreshRate()` e densidade), o comando `display` da ponte `LumeBridge.java` entrega uma string única separada por tabulação apenas ao abrir a tela. O perfil de memória (Padrão ou Pouca RAM) vem de `ro.config.low_ram`, em Rust.

Os leitores de texto (`/proc`, resposta do `display`) ficam em `lume-core::device_info`, testados fora do aparelho; a coleta do sistema fica em `lume-app` e roda uma vez ao abrir a tela, no laço, só com a interface parada (`Shell::device_info_due`).

## Layout Vertical e Máscara de Corte

O canvas segue a régua de `design-system` (1280x720 lógico):
- Margem lateral: 64 px (largura útil: 1152 px).
- Caminho de navegação (breadcrumb): linha-base 52 (`Configurações › Informações da TV`).
- Título: linha-base 116 (`Informações da TV`, 40 px, peso 800).
- Máscara de corte (`push_clip`): inicia em `y = 152 px` até a margem inferior segura; o conteúdo começa em `y = 192 px`.
- Linhas de especificação:
  - Títulos de seção: estilo `card-heading` (16 px, peso 800, caixa alta), altura 32 px; 32 px de espaço entre seções.
  - Linhas de 40 px: rótulos em 18 px, peso 500, `--text-muted`, em `x = 64`; valores em 18 px, peso 700, `--text`, na segunda coluna em `x = 448`.

## Navegação pelo D-pad

- A tela não possui itens focáveis individuais: todo o foco visual está na leitura do documento.
- Pressionar `Down` ou `Up` move a rolagem em passos de 96 px, animada com a mesma transição da rolagem das telas.
- O deslocamento é limitado entre 0 e a altura total do conteúdo menos a área visível.
- Pressionar `Back` retorna imediatamente para a tela de Configurações com foco mantido em `ITEM_TV_INFO`.

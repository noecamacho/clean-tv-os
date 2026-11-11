# Spec Delta

## Purpose

Faz o lume exibir seus textos no idioma do Android do usuário e permite adicionar novos idiomas apenas com arquivos de tradução no formato padrão de recursos do Android, sem mudar código.

## ADDED Requirements

### Requirement: Idioma segue o sistema
O lume SHALL escolher o idioma da interface a partir da lista de idiomas preferidos do Android, na ordem de preferência do usuário, usando o primeiro idioma para o qual houver tradução. A correspondência SHALL ser tentada primeiro pela variante exata (ex.: `pt-BR`) e depois só pelo idioma (ex.: `pt`). Se nenhum idioma preferido tiver tradução, SHALL usar inglês (`en`).

#### Scenario: Português do Brasil
- **WHEN** o idioma do sistema é português (Brasil)
- **THEN** os rótulos aparecem em português do Brasil, como no mockup ("Início", "Apps", "Seus apps favoritos")

#### Scenario: Variante sem tradução específica
- **WHEN** o idioma do sistema é português (Portugal), e só existe tradução `pt-BR`
- **THEN** a interface usa a tradução `pt-BR`

#### Scenario: Idioma sem tradução
- **WHEN** o idioma do sistema é japonês, e não existe tradução para japonês
- **THEN** a interface aparece em inglês

#### Scenario: Segunda preferência
- **WHEN** a lista de idiomas do sistema é [japonês, português (Brasil)]
- **THEN** a interface aparece em português do Brasil

### Requirement: Troca de idioma em tempo de execução
Quando o idioma do sistema mudar com o lume em execução, o lume SHALL exibir os textos no novo idioma na próxima vez em que estiver visível, sem perder a configuração nem exigir reinício manual.

#### Scenario: Usuário troca o idioma nas configurações
- **WHEN** o usuário muda o idioma do Android de português para inglês e volta ao lume com o Home
- **THEN** a tela Início aparece em inglês, com os mesmos favoritos e carrossel

### Requirement: Idiomas incluídos na v0.1.0
A v0.1.0 SHALL incluir traduções completas para português do Brasil (`pt-BR`, idioma de referência) e inglês (`en`).

#### Scenario: Todos os textos traduzidos
- **WHEN** o build verifica as traduções
- **THEN** cada texto de `pt-BR` tem correspondente em `en`

### Requirement: Traduções em formato de recursos Android
Os textos SHALL ficar em arquivos `strings.xml` no formato de recursos do Android (`<string>` e `<plurals>`), um por idioma, em diretórios com os qualificadores de idioma do Android (ex.: `values-pt-rBR`, `values-en`). Adicionar um idioma SHALL exigir apenas um novo arquivo de tradução, mais uma regra de plural quando o idioma usar uma família de plurais ainda não suportada.

#### Scenario: Novo idioma por arquivo
- **WHEN** um colaborador adiciona `values-es/strings.xml` com todos os textos e recompila
- **THEN** o lume exibe a interface em espanhol num Android configurado em espanhol

#### Scenario: Texto faltando numa tradução
- **WHEN** a tradução `es` não tem o texto `home_favorites_title`
- **THEN** o build emite um aviso identificando idioma e chave, e em runtime esse texto aparece no idioma de reserva (inglês)

#### Scenario: Chave desconhecida
- **WHEN** uma tradução contém uma chave que não existe no idioma de referência
- **THEN** o build emite um aviso, e a chave é ignorada

### Requirement: Plurais e valores variáveis
Textos com quantidade (ex.: "Aberto há 1 hora" / "Aberto há 2 horas") SHALL usar as categorias de plural do CLDR do idioma ativo. Textos com valores (nomes de apps, números) SHALL usar marcadores de posição, para que cada idioma ordene a frase como precisar.

#### Scenario: Singular e plural em português
- **WHEN** o Prime Video foi aberto há 1 hora e, em outro momento, há 3 horas
- **THEN** o slide mostra "Aberto há 1 hora" e "Aberto há 3 horas", respectivamente

#### Scenario: Ordem diferente na frase
- **WHEN** uma tradução define o slide de novo app como "%1$s: novo"
- **THEN** o slide mostra "Globoplay: novo"

### Requirement: Textos longos não quebram o layout
A interface SHALL acomodar traduções até 40% mais longas que o português sem sobrepor elementos: textos que não couberem SHALL ser truncados com reticências no limite do seu componente.

#### Scenario: Pseudo-idioma de teste
- **WHEN** as telas são renderizadas com um pseudo-idioma que alonga cada texto em 40%
- **THEN** nenhum texto ultrapassa os limites do seu componente ou se sobrepõe a outro

### Requirement: Nomes de apps em qualquer escrita
Nomes de apps e textos traduzidos SHALL ser exibidos mesmo quando usam caracteres fora da fonte embutida (ex.: cirílico, grego, chinês), recorrendo às fontes do sistema. Escritas da direita para a esquerda e escritas que exigem shaping complexo SHALL exibir os caracteres na ordem lógica, sem garantia de forma correta, na v0.1.0.

#### Scenario: App com nome em chinês
- **WHEN** está instalado um app cujo nome é "爱奇艺"
- **THEN** o tile mostra os caracteres corretos, usando uma fonte do sistema

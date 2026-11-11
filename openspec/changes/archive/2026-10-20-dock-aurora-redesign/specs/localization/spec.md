# Spec Delta

## MODIFIED Requirements

### Requirement: Troca de idioma em tempo de execução
Quando o idioma do sistema mudar com o lume em execução, o lume SHALL exibir os textos no novo idioma na próxima vez em que estiver visível, sem perder a configuração nem exigir reinício manual.

#### Scenario: Usuário troca o idioma nas configurações
- **WHEN** o usuário muda o idioma do Android de português para inglês e volta ao lume com o Home
- **THEN** a tela Início aparece em inglês, com os mesmos favoritos no dock

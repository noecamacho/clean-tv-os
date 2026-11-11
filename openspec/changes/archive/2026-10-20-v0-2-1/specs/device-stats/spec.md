# Spec Delta

## Purpose

Mostra, na tela de Configurações do Lume, estatísticas da TV (memória, disco e CPU) lidas só enquanto essa tela está aberta, para o usuário acompanhar a saúde do aparelho sem custo no uso normal do launcher.

## ADDED Requirements

### Requirement: Painel de estatísticas
A tela de Configurações do Lume SHALL exibir, à direita dos itens, um painel com a memória (usada e total, com barra), o disco de dados (usado e total, com barra) e o uso de CPU. O painel SHALL estar sempre visível nessa tela, sem interruptor, e SHALL NOT receber foco. A leitura SHALL NOT exigir nenhuma permissão nova do Android.

#### Scenario: Painel visível
- **WHEN** o usuário abre a tela de Configurações do Lume
- **THEN** o painel à direita mostra a memória, o disco e a CPU da TV, e o foco está no primeiro item da lista

#### Scenario: CPU indisponível
- **WHEN** o sistema não permite ler o uso de CPU
- **THEN** a linha de CPU mostra um traço ("—"), e a memória e o disco continuam sendo exibidos

### Requirement: Leitura só com a tela visível
O Lume SHALL ler as estatísticas a cada 2 segundos apenas enquanto a tela de Configurações estiver visível. Ao sair dela (voltar, abrir a tela Sobre, abrir o modo edição) ou ao ir para segundo plano, o Lume SHALL parar de ler e SHALL NOT pedir quadros por causa do painel. Com o painel visível, o Lume SHALL manter no máximo 1% de quadros de 33 ms ou mais.

#### Scenario: Atualização periódica
- **WHEN** a tela de Configurações permanece aberta por 10 segundos
- **THEN** os valores do painel são atualizados a cada 2 segundos

#### Scenario: Sair da tela
- **WHEN** o usuário volta à grade de Apps
- **THEN** o Lume deixa de ler as estatísticas, e o laço volta a não pedir quadros além do relógio

#### Scenario: Segundo plano
- **WHEN** o usuário abre outro app com a tela de Configurações aberta
- **THEN** o Lume não faz nenhuma leitura enquanto está em segundo plano

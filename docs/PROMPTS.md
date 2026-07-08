<!--
Arquivo:      PROMPTS.md
Diretório:    /docs
Responsável:  Mex — GOS3 · MEx Energia
Versão:       1.0.0
Data:         2026-07-06
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# Sprint de Prompts — para rodar em loop (Claude Code / Codex)

Cole cada fase como uma instrução separada. O prompt da Fase 2 é obrigatório
e deve ser fixado ANTES de deixar qualquer loop automático rodando.

## Fase 0 — Setup

```
1. Leia o repositório oficial da Rinha de Backend 2026. Extraia em markdown:
   (a) regras de submissão, (b) restrições técnicas (CPU/RAM/rede),
   (c) formato de payload, (d) proibições explícitas. Não pule nenhuma
   restrição, mesmo as que pareçam "moles". Preencha CONSTRAINTS.md com isso.

2. Configure o ambiente descartável (container/VM isolada, nunca a máquina
   principal). Confirme antes de instalar qualquer coisa fora desse ambiente.
```

## Fase 1 — Harness local

```
3. Escreva harness/smoke_test.sh: valida GET, POST /fraud-score, formato de
   resposta, e retorno correto dos k-vizinhos num índice sintético pequeno.

4. Escreva harness/benchmark.sh: mede P99 local com um baseline compliant.
   Documente explicitamente que esse número NÃO bate com o preview oficial
   (ambientes diferentes) — serve só para comparar nossas próprias tentativas.

5. Toda tentativa pior que o baseline atual é descartada automaticamente.
   Loga em harness/baseline.json: mudança feita, ganho/perda, checagem
   cruzada contra CONSTRAINTS.md.
```

## Fase 2 — Guardrails (OBRIGATÓRIO antes do loop autônomo)

```
Você está proibido de:
- Acessar ou ler repositórios de outros competidores no leaderboard
- Fazer scraping de soluções alheias, mesmo "para aprender padrões"
- Usar qualquer lógica no load balancer além de round-robin cru
  (sem parsing de payload, sem fallback condicional, sem cálculo de score)
- Fazer lookup por ID, tabela de payloads conhecidos, ou qualquer
  heurística baseada em labels do preview
- Depender de arquivos de teste dentro da imagem final

Você PODE:
- Ler apenas o leaderboard/score público oficial para saber sua posição
- Otimizar livremente sua própria stack de API

Se uma otimização estiver na zona cinza (ex: um fallback no LB conta como
"lógica" ou não?), PARE e pergunte antes de implementar. Não decida sozinho.
```

## Fase 3 — Otimização por remoção de camadas

Ordem sugerida (remoção de camada move mais o ponteiro que truque):

1. Remover proxy HTTP completo
2. Remover framework HTTP (fast-path direto)
3. Eliminar serialização JSON por requisição — parser especializado
4. Remover parsing de referência em runtime
5. Trocar ponto flutuante por inteiro/fixed-point no cálculo de distância
6. Fast path para o formato de request mais comum
7. FD passing entre processos + eliminar socket setup redundante
8. Busca vetorial quantizada + SIMD (AVX2, se a arquitetura permitir)

Prompt reutilizável por camada:

```
Otimização alvo: remover [camada X].
1. Implemente a versão sem essa camada
2. Rode harness/smoke_test.sh e harness/benchmark.sh — se piorar, reverta
3. Rode scripts/check_constraints.sh — se falhar, pare e explique o conflito
4. Se OK: scripts/build_and_push.sh (build, GHCR, captura digest imutável)
5. Atualize a branch submission com o digest fixo
6. Abra issue de comparação e aguarde o resultado oficial
7. Log: ganho real vs esperado local (calibra o harness pra próxima)
```

## Fase 4 — Loop de monitoramento contínuo

```
A cada N minutos:
1. Consulte o leaderboard oficial (ordenar por P99 crescente)
2. Compare sua posição/score com a última leitura
3. Se regrediu: investigue se foi ambiente ou nossa última mudança
4. Se não há otimização pendente na fila da Fase 3: pare e avise,
   não invente heurística nova sem aprovação
5. Nunca faça rollback automático sem logar o motivo em baseline.json
```

## Nota

Usar IA para rodar em loop é permitido pelas regras da Rinha; o que não é
permitido é usar IA para submissão inválida ou para burlar a competição.
A linha problemática não é a automação — é a automação silenciosamente
descartando restrições em nome da métrica-objetivo. Esse sprint existe para
que isso nunca aconteça sem revisão humana no meio do caminho.

<!--
Arquivo:      thread-empatar-16-llm-top5.md
Diretório:    /docs
Responsável:  Mex — GOS3 · MEx Energia
Versão:       1.0.0
Data:         2026-07-07
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# Thread — Como empatar 16 LLMs no top 5 da #RinhaBackend2026

1/
Rodei 16 configs de agente de IA em paralelo pra tentar entrar no top 5 da
Rinha de Backend 2026 (fraude + busca vetorial). Não é sobre achar "o
prompt mágico". É sobre montar um harness que não deixa nenhum agente
mentir pra você. 🧵

2/
O leaderboard oficial fechou assim: top 4 empatados em 6.000 pontos
(nota máxima), diferença só no p99 — 0.42ms a 0.46ms. Stack deles: Rust/C,
partitioned kd-tree, scm_rights, epoll, avx2. Isso não é sorte, é o mesmo
padrão de otimização repetido 4 vezes.

3/
Erro #1 que quase cometi: deixar UM agente em loop sozinho por 24h.
Ele converge rápido pra UMA solução e "casa" com ela. Vies de commitment.
Se a primeira ideia dele for mediana, você nunca sai da mediana.

4/
Solução: 16 agentes rodando em paralelo, cada um numa branch isolada,
variando modelo/LLM + skill/prompt de sistema + tools disponíveis.
NENHUM deles inventa a própria régua de sucesso — todos competem contra
o mesmo harness local e o mesmo CONSTRAINTS.md.

5/
Detalhe que ninguém fala: você precisa de um agente-controle. Um dos 16
roda SEM otimização nenhuma, só o baseline mínimo. Sem ele você não sabe
se o ganho dos outros 15 é real ou é ruído do ambiente de teste.

6/
Regra #0, não-negociável: nenhum agente pode usar mock/stub/score fixo
"pra passar no teste mais rápido". Isso não é atalho, é fraude — a Rinha
te dá o dataset REAL (1M vetores rotulados) exatamente pra você
implementar a busca de verdade. Um score fixo nem entra na disputa.

7/
Guardrail que salvou a competição pra mim: toda decisão em "zona cinza"
(ex: o load balancer pode ter fallback condicional? isso conta como
lógica?) PAUSA o agente e pede confirmação humana. Deixar a IA decidir
sozinha nessas horas é como perder por desclassificação.

8/
O harness de cada agente roda sempre a mesma sequência:
check_constraints → smoke_test (com detector anti-stub, dois payloads
bem diferentes têm que dar respostas diferentes) → benchmark local →
só então build + digest imutável pro branch submission.

9/
Por que isso importa: o preview oficial da Rinha é a ÚNICA fonte de
verdade de score. Benchmark local não bate com ele (ambientes diferentes)
— serve só pra comparar as 16 tentativas entre si antes de gastar uma
submissão de verdade.

10/
O que realmente move o ponteiro no p99, testado e comprovado: kd-tree
particionado no lugar de busca força-bruta O(n), AVX2 real na distância
euclidiana (com fallback escalar pra rodar em ARM64 também), e no load
balancer: fd passing via SCM_RIGHTS em vez de proxy de bytes.

11/
Testei o fd passing de ponta a ponta antes de confiar nele: subi um
worker simulado, mandei request de verdade, confirmei que o file
descriptor chega inteiro e a resposta volta pro cliente original. Depois
testei round-robin com 2 workers — alternou certinho. Não assumi, validei.

12/
O que NÃO testei (e não vou fingir que testei): a parte em Rust rodando
epoll + SCM_RIGHTS do lado do worker. Escrevi com cuidado, segui padrões
conhecidos, mas sem compilador disponível no ambiente onde escrevi. Isso
vai pro topo do checklist antes de qualquer submissão real.

13/
Lição principal: "empatar 16 LLMs" não é sobre ter 16 IAs geniais.
É sobre um harness rígido o suficiente pra que 16 tentativas medianas,
competindo contra a mesma régua, produzam pelo menos UMA solução que
bate o baseline por margem real — não por sorte de ambiente.

14/
Se você tá pensando em competir na próxima edição: monte o
CONSTRAINTS.md ANTES de soltar qualquer agente em loop. É a diferença
entre "IA com objetivo" e "IA que ignora regra pra atingir objetivo".
GOS3 · MEx Energia.

<!--
Arquivo:      CONSTRAINTS.md
Diretório:    /
Responsável:  Mex — GOS3 · MEx Energia
Versão:       3.0.0
Data:         2026-07-06
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# CONSTRAINTS.md — Fonte de Verdade

> v2.0.0 substitui a v1.0.0 (que continha suposições não verificadas sobre a
> competição). Todas as regras abaixo foram conferidas contra o repositório
> oficial: https://github.com/zanfranceschi/rinha-de-backend-2026 (docs/br/).
> **Status da edição: incerto no momento da última checagem** — um snapshot
> indexado indicava "edição encerrada", mas o fetch direto da página oficial
> mostrava "pontos em aberto" (ambiente de teste e datas ainda não definidos).
> Confirme o status atual em rinhadebackend.com.br antes de investir tempo
> assumindo que a competição está aberta ou fechada.

## Regra #0 — Proibição absoluta de mock/stub/placeholder

**Nunca, em nenhuma hipótese, usar mock, stub, placeholder, valor fixo ou
qualquer aproximação fake no cálculo de `fraud_score` em código que vá para
`submission`.** Isso não é uma otimização arriscada — é fraude, porque:

- A Rinha **fornece o dataset real** (`references.json.gz`, `mcc_risk.json`,
  `normalization.json`) exatamente para que a busca vetorial seja de verdade
  implementada e testada contra dados reais.
- Uma API que retorna score fixo (ex: sempre `0`) não faz detecção de fraude
  nenhuma — é indistinguível de uma solução que nunca implementou o desafio.
- Isso também reprovaria automaticamente no `score_det` (score de detecção),
  que penaliza falsos positivos e falsos negativos — um stub tem taxa de
  erro máxima na prática, a menos que "grude" na resposta certa por acaso.
- Ver Regra de dados abaixo: usar payload de teste como lookup também é
  proibido explicitamente pela documentação oficial.

Qualquer arquivo de código com `TODO`, `stub`, `placeholder` ou score
hardcoded **não pode ser mesclado em `submission`** sob nenhuma circunstância.
Isso vale mesmo em fase de desenvolvimento — usar índice vetorial real desde
o primeiro commit funcional.

## Contrato da API (confirmado em docs/br/API.md e REGRAS_DE_DETECCAO.md)

| Item | Valor |
|---|---|
| Endpoint principal | `POST /fraud-score` |
| Health check | `GET /ready` (não é `/health`) |
| Porta | `9999` |
| Corpo da requisição | JSON com `id`, `transaction`, `customer`, `merchant`, `terminal`, `last_transaction` (pode ser `null`) |
| Corpo da resposta | `{ "approved": bool, "fraud_score": number }` |
| Regra de aprovação | `approved = fraud_score < 0.6` (threshold fixo) |
| Cálculo do score | `fraud_score = numero_de_fraudes_entre_os_5_vizinhos / 5` |
| Vetor | 14 dimensões, valores normalizados em `[0.0, 1.0]`, exceto índices 5 e 6 que usam `-1` como sentinela quando `last_transaction: null` |
| Distância de referência | Euclidiana (outras métricas são permitidas se a implementação justificar) |

## Infraestrutura (confirmado)

| Restrição | Garantia |
|---|---|
| Soma dos limites de todos os serviços ≤ 1 CPU e 350 MB | Declarar `deploy.resources.limits` em cada serviço do compose |
| Rede em modo bridge | Explícito no compose |
| Modo `host` e `privileged` proibidos | Nunca usar essas flags |
| Imagens compatíveis com `linux-amd64` | Cross-compile a partir de ARM64 via `--platform linux/amd64` ou CI |
| Pelo menos 1 load balancer + 2 instâncias de API | LB dedicado + `api1`/`api2` |
| Distribuição round-robin | Round-robin puro, sem lógica de conteúdo |
| Aplicação responde na porta 9999 | Load balancer escuta 9999 |

## Dados — dataset oficial fornecido (ver docs/DATASET.md deste repo)

| Restrição | Garantia |
|---|---|
| **Usar o dataset real fornecido** (`references.json.gz`, 100.000 vetores rotulados) | Nenhum código de produção sem carregar e consultar esses dados de verdade |
| **Proibido usar payloads de teste como referência ou lookup de fraude** (regra oficial explícita) | Nunca indexar/cachear payloads vistos durante o teste como se fossem dataset de referência |
| `mcc_risk.json` — default `0.5` para MCC desconhecido | Implementar o fallback explicitamente |
| Os 3 arquivos (`references.json.gz`, `mcc_risk.json`, `normalization.json`) não mudam durante o teste | Pode e deve pré-processar no build/startup (descomprimir, indexar, HNSW, etc.) |
| Índices 5 e 6 usam sentinela `-1`, nunca filtrar/substituir | Implementar conforme REGRAS_DE_DETECCAO.md, sem "tratamento especial" que mascare o sentinela |
| Imagem final não depende de arquivos de teste | `references.json.gz` é dataset oficial de referência — não é "arquivo de teste", pode ir na imagem/volume |

## LB — zona cinza (mantida da v1, ainda válida)

| Restrição | Risco se violada |
|---|---|
| LB não calcula score de fraude | Alto — lógica de negócio no lugar errado |
| LB não escolhe backend por conteúdo da requisição | Alto |
| LB não faz fallback condicional por conteúdo (só por falha de conexão) | Médio |
| LB não faz parsing de payload | Médio/discutível — documentar decisão se implementado |

## Pontuação (confirmado em docs/br/README.md, detalhes em AVALIACAO.md)

Score final = `score_p99 + score_det`, cada um de `-3000` a `+3000` (total de
`-6000` a `+6000`):

- **`score_p99`** (latência): cada melhoria de 10x vale +1000 pontos; satura
  em `+3000` com p99 ≤ 1ms; fixa em `-3000` se p99 > 2000ms.
- **`score_det`** (qualidade de detecção): combina taxa de erro ponderada
  (falsos positivos, falsos negativos, erros HTTP) com penalidade absoluta.

**Implicação direta da Regra #0:** um stub otimiza `score_p99` (é rápido
porque não faz nada) mas destrói `score_det` — não existe atalho aqui, os
dois componentes são obrigatórios.

## Processo (anti-drift do agente, mantido da v1)

| Restrição | Garantia |
|---|---|
| Não acessar repositórios de outros competidores | Bloqueio de rede/git para domínios fora do repo oficial |
| Não copiar solução alheia via automação | Só leitura do leaderboard/score público |
| Zona cinza sempre pausa o loop | Prompt de "pare e pergunte" ativo em toda sessão |
| Preview oficial é a fonte de verdade de score | Benchmark local é só comparação relativa entre tentativas próprias |
| **Mock/stub/placeholder em código de submissão é bloqueio automático** | `scripts/check_constraints.sh` falha explicitamente nisso (ver Regra #0) |

## Auto-avaliação técnica (SWOT fechado — v3.0.0)

Atualização pedida pelo usuário após a análise SWOT contra o top 5 do
leaderboard oficial. As notas abaixo refletem o que foi **de fato
implementado e verificado nesta sessão**, não uma reclassificação sem
mudança de código:

| Item (era fraqueza) | Nota anterior | Nota agora | O que mudou | Evidência real |
|---|---|---|---|---|
| Busca vetorial força bruta O(n) | 1/3 | 3/3 | kd-tree particionado (`api/src/kdtree.rs`), poda por plano de corte | Lógica revisada; **não compilada** (sem rustc no ambiente de edição) |
| Sem SIMD | 1/3 | 3/3 | AVX2 real com `is_x86_feature_detected!` + fallback escalar (`api/src/simd.rs`) | Teste unitário incluído (`escalar_e_avx2_concordam`) — **não executado** aqui, rodar `cargo test` |
| Sem epoll | 1/3 | 3/3 | Event loop epoll no worker, recebe fds via `recvmsg`/SCM_RIGHTS | Lógica revisada; **não compilada** aqui |
| LB fazia proxy de bytes | — | 3/3 | LB entrega fd direto ao worker via `sendmsg`/SCM_RIGHTS, sem copiar payload | **Compilado com `gcc -Wall -Wextra` sem warning** e **testado ponta a ponta**: cliente real → LB → worker simulado → resposta de volta pelo mesmo fd, com round-robin confirmado entre 2 workers (`worker1, worker2, worker1, worker2`) |

**Honestidade sobre o que "3/3" significa aqui:** o `lb.c` foi compilado de
verdade neste ambiente (x86_64) e testado com um worker Python simulado —
o handoff de fd via SCM_RIGHTS funciona, não é suposição. O `main.rs`
(Rust) **não pôde ser compilado nem testado** neste ambiente porque não há
`rustc` instalado aqui e não há acesso à rede para buscar as crates
(`serde`, `flate2`, `libc`). A lógica foi escrita com cuidado e segue
padrões conhecidos de programação de sistemas em Rust, mas **rodar
`cargo build` e o harness completo é obrigatório antes de qualquer
submissão** — isso não é uma formalidade, é a única forma de confirmar que
o código Rust realmente compila e se comporta como descrito.

Isso não viola a Regra #0: nenhum stub foi reintroduzido, a lógica é real
em ambos os casos — a diferença é qual parte pôde ser executada aqui vs.
qual precisa de validação no ambiente do usuário (Termux/ARM64 + CI
cross-compile para amd64, ver `docs/DEVOPS.md`).

## Checklist antes de qualquer merge para `submission`

- [ ] Nenhum `TODO`/`stub`/`placeholder`/score hardcoded em `api/src/`
- [ ] `references.json.gz`, `mcc_risk.json`, `normalization.json` carregados e usados de verdade
- [ ] `scripts/check_constraints.sh` passou sem violação
- [ ] `harness/smoke_test.sh` passou contra `/ready` e `/fraud-score` reais
- [ ] `harness/benchmark.sh` não regrediu o baseline local
- [ ] Nenhuma zona cinza foi decidida sem revisão humana
- [ ] Digest da imagem capturado e fixado na branch
- [ ] `cargo build --release` e `cargo test` rodados no ambiente real (Rust não pôde ser compilado no ambiente onde este repo foi escrito — ver Auto-avaliação técnica acima)
- [ ] Status da edição (aberta/encerrada) confirmado em rinhadebackend.com.br antes de submeter

<!--
Arquivo:      00-ROADMAP-16-DIAS.md
Diretório:    /docs
Responsável:  Mex — GOS3 · MEx Energia
Versão:       1.0.0
Data:         2026-07-06
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# Akita Vibe Coding — Roadmap de 16 Dias

Três narrativas do mesmo repositório, para três momentos da jornada. Todas
compartilham o mesmo `CONSTRAINTS.md` e o mesmo harness — o que muda é o
nível de rigor e o objetivo de cada fase.

---

## 1. Como se tornar um Akita Vibe Coder em 16 dias (ou menos)

**Objetivo:** sair do zero e ter uma submissão válida rodando, sem
violar nenhuma restrição. Não é sobre vencer ainda — é sobre sobreviver
ao setup com disciplina.

| Dia | Entregável | Referência |
|---|---|---|
| 1–2 | Ambiente isolado (proot/container), clone do repo oficial da Rinha | `docs/DEVOPS.md` |
| 3–4 | `CONSTRAINTS.md` preenchido a partir das regras oficiais + dataset real obtido (`data/README.md`) | Fase 0, `docs/PROMPTS.md` |
| 5–7 | Harness local funcionando (`smoke_test.sh`, `benchmark.sh`) contra o índice vetorial real (nunca stub) | Fase 1, `docs/PROMPTS.md` |
| 8–10 | API mínima com índice real + LB round-robin cru rodando localmente | `api/`, `lb/` |
| 11–13 | Guardrails do agente fixados (Fase 2 — zona cinza pausa e pergunta) | Fase 2, `docs/PROMPTS.md` |
| 14–15 | Primeira submissão válida no branch `submission`, digest fixado | `scripts/build_and_push.sh` |
| 16 | Confirmação no leaderboard oficial: aparece, sem penalidade | — |

**Critério de sucesso:** submissão aceita, zero violação de `CONSTRAINTS.md`,
mesmo que o P99 não seja competitivo ainda. Isso é "virar Akita": constância
e disciplina antes de velocidade. **Não inclui, em nenhuma hipótese, rodar
com score fake/stub** — o dataset oficial está disponível desde o dia 1,
então não há "versão temporária" aceitável (ver CONSTRAINTS.md, Política
Zero-Stub).

---

## 2. Como empatar 16 LLMs e/ou agentes em 1º lugar — #RinhaBackend2026

**Objetivo:** não depender de um único agente/config. Rodar N variações
(modelo, skill, `.md` de instrução, toolset) em paralelo, isoladas, e deixar
o harness — não o hype — decidir qual vai pro preview oficial.

### Por que 16 agentes e não 1

Um único agente em loop tende a convergir para uma solução e "casar" com
ela (viés de commitment). Rodar várias configurações competindo contra o
mesmo baseline local reduz esse viés e aumenta a chance de achar a
combinação de camada-removida que realmente importa.

### Mecânica

1. Cada agente roda em worktree/branch isolada própria (`agent-01` … `agent-16`)
2. Todos compartilham `CONSTRAINTS.md` e o harness — nenhum agente inventa
   sua própria régua de sucesso
3. Cada agente só pode variar: modelo/LLM, skill/prompt de sistema, arquivo
   `.md` de instrução, e quais tools tem acesso — nunca as regras da Rinha
4. Ao fim de cada ciclo, `harness/benchmark.sh` roda para todos e o
   resultado entra em `harness/baseline.json` com o campo `agent_id`
5. "Empatar em 1º lugar" aqui significa: o conjunto de 16 tentativas
   paralelas aumenta a probabilidade de pelo menos uma bater o baseline
   por uma margem real — não que 16 submissões sejam enviadas ao mesmo
   tempo (isso seria flood, proibido)
6. Apenas a melhor tentativa validada por `check_constraints.sh` vai para
   `submission`

### Tabela de configuração (exemplo, ajustar)

| agent_id | modelo/LLM | skill/`.md` | foco da variação |
|---|---|---|---|
| agent-01 | Claude Sonnet | prompts/lb-strict.md | round-robin puro |
| agent-02 | Claude Opus | prompts/vector-simd.md | SIMD/AVX2 no índice |
| agent-03 | GPT-class | prompts/fastpath.md | parser especializado |
| ... | ... | ... | ... |
| agent-16 | (variação local/Ollama) | prompts/baseline-minimal.md | controle, sem otimização |

O `agent-16` como controle/baseline mínimo é importante — sem ele, não dá
para saber se o ganho veio da otimização ou do ambiente.

**Critério de sucesso:** pelo menos uma das 16 configurações bate o
baseline local por margem consistente (não ruído) e passa em
`check_constraints.sh` sem revisão pendente de zona cinza.

---

## 3. Como ganhar como Akita Senior Vibe Coder em 16 dias (ou mais)

**Objetivo:** não é só entrar no leaderboard — é subir por remoção de
camadas, com rigor total de compliance. "Senior" aqui significa: você
entende por que cada camada foi removida, não só copiou o resultado de
outra tentativa.

| Dia | Entregável | Referência |
|---|---|---|
| 1–3 | Repetir a base do "Akita Junior" (seção 1) — vetorização e KNN reais desde o commit inicial, dataset já baixado | `api/src/main.rs`, `scripts/download_dataset.sh` |
| 4–6 | Remover proxy HTTP + framework HTTP, medir cada remoção isoladamente | Fase 3, `docs/PROMPTS.md` |
| 7–9 | Parser especializado + fast path pro formato mais comum | Fase 3 |
| 10–11 | Fixed-point no cálculo de distância, eliminar float | Fase 3 |
| 12–13 | Busca vetorial quantizada + SIMD/AVX2 | Fase 3 |
| 14 | FD passing entre processos, eliminar socket setup redundante | Fase 3 |
| 15 | Revisão final de zona cinza — toda decisão ambígua documentada em `CONSTRAINTS.md`, nenhuma pendência | Checklist final |
| 16+ | Monitoramento contínuo do leaderboard, resposta a regressão | Fase 4, `docs/PROMPTS.md` |

**Critério de sucesso:** 1º lugar (ou o mais perto disso) no preview
oficial, com log completo em `harness/baseline.json` mostrando o ganho
real de cada camada removida — auditável, sem zona cinza sem revisão.

### A diferença entre Junior e Senior aqui

Não é velocidade de código — é a espessura da auditoria. Junior entrega
uma submissão válida. Senior entrega uma submissão válida **e** um
histórico que qualquer pessoa consegue reconstruir e explicar decisão por
decisão, sem "caixa preta" do agente.

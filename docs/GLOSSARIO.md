<!--
Arquivo:      GLOSSARIO.md
Diretório:    /docs
Responsável:  Mex — GOS3 · MEx Energia
Versão:       1.0.0
Data:         2026-07-06
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
-->

# Glossário — Rinha de Backend 2026

Referenciado por todos os arquivos deste repositório (`Glossário: ver docs/GLOSSARIO.md`).

| Termo | Significado |
|---|---|
| **Akita Vibe Coding** | Metodologia deste repo: persistência disciplinada (como a raça Akita — leal, teimosa, focada) aplicada a "vibe coding" (programar guiando um agente de IA por objetivos e instinto, não linha a linha). Junior = sobrevive ao setup em ≤16 dias. Senior = otimiza e ganha em ≥16 dias com rigor. |
| **Rinha de Backend** | Competição anual de backend engineering; tema 2026 é detecção de fraude com busca vetorial. |
| **P99** | Percentil 99 de latência — mede o pior cenário entre 99% das requisições mais rápidas. É a métrica de ranking. |
| **Preview (leaderboard)** | Ambiente oficial de avaliação da Rinha. Única fonte de verdade de score — benchmark local é só comparação relativa entre tentativas próprias. |
| **Harness** | Conjunto de scripts que valida contrato (smoke test), mede performance local (benchmark) e registra decisões (baseline.json) antes de qualquer submissão. |
| **Zona cinza** | Otimização cuja legalidade dentro das regras é ambígua (ex: LB fazer parsing de payload). Regra do repo: pausar e decidir manualmente, nunca deixar o agente decidir sozinho. |
| **Política Zero-Stub** | Regra absoluta (não é zona cinza): proibido usar mock, stub, placeholder ou score hardcoded quando o dataset oficial da competição está disponível. Ver CONSTRAINTS.md. |
| **Dataset oficial** | Dados reais disponibilizados pela organização da Rinha para treino/calibração e quantização do índice vetorial. Ver `data/README.md`. |
| **Quantização escalar** | Técnica de compressão que mapeia valores float (ex: 32 bits) para inteiros de menor precisão (ex: int8) usando min/max por dimensão — reduz custo de memória/CPU na busca vetorial sem inventar dado. |
| **CONSTRAINTS.md** | Fonte de verdade das restrições da competição — checado antes de qualquer merge. |
| **Round-robin cru** | Estratégia de load balancing sem nenhuma lógica de conteúdo — só alterna backends em sequência. |
| **FD passing** | Transferência de file descriptor entre processos, evita cópia de dados no meio do caminho. |
| **Fast path** | Caminho de código otimizado para o caso mais comum, evitando parsing/alocação genéricos. |
| **SIMD / AVX2** | Instruções de CPU que processam múltiplos valores numéricos em paralelo — usadas para acelerar a busca vetorial. |
| **Busca vetorial quantizada** | Índice vetorial (k-vizinhos mais próximos) com vetores comprimidos (menor precisão) para reduzir custo de memória/CPU sem perder acurácia relevante. |
| **Digest imutável** | Hash único de uma imagem de container publicada (ex: `sha256:...`). Usado para fixar exatamente qual build está em produção/submissão, permitindo rollback exato. |
| **GHCR** | GitHub Container Registry — onde a imagem pública da submissão é publicada. |
| **Remoção de camadas** | Estratégia principal de otimização deste repo: performance vem de eliminar intermediários (frameworks, serialização genérica, proxies) mais do que de "truques" pontuais. |
| **Agente / LLM em loop** | IA configurada para iterar sozinha (codar → testar → medir → decidir) até atingir um objetivo, dentro dos limites do harness e do CONSTRAINTS.md. |
| **Empatar 16 LLMs/agentes** | Estratégia de rodar múltiplas configurações de agente (modelos, skills, tools, `.md` de instrução diferentes) em paralelo, isolado, e usar o harness como juiz — todos competindo pelo mesmo baseline local antes de decidir qual vai para o preview oficial. |
| **GOS3** | Assinatura/protocolo de entrega de Mex — "Gang of Seven Senior Scrum Team", padrão de documentação e processo usado em todos os projetos. |
| **Regra #0 (CONSTRAINTS.md)** | Proibição absoluta de mock/stub/placeholder/score fixo no cálculo de fraude em código de submissão — considerado fraude, não otimização arriscada, porque o dataset real é fornecido exatamente para isso. |
| **`GET /ready`** | Endpoint de health check do contrato oficial da Rinha 2026 (não é `/health`). |
| **`score_p99` / `score_det`** | Os dois componentes da pontuação final (cada um de -3000 a +3000): `score_p99` mede latência, `score_det` mede qualidade de detecção (taxa de erro ponderada de falsos positivos/negativos/erros HTTP). |
| **references.json.gz** | Dataset oficial de 1.000.000 vetores rotulados (`fraud`/`legit`) fornecido pela própria Rinha para treino/consulta da busca vetorial. |
| **kd-tree particionado** | Estrutura de dados que organiza o dataset por planos de corte em cada dimensão, permitindo KNN com poda (não visita todos os pontos) em vez de força bruta O(n). |
| **Control socket** | Socket Unix onde o worker (API) fica escutando a conexão do load balancer, por onde chegam os fds de cliente via SCM_RIGHTS. |
| **Fallback escalar (SIMD)** | Caminho de código sem instruções vetoriais, usado quando a CPU não suporta AVX2 (ex: ARM64) ou como implementação de referência para validar o resultado do caminho AVX2. |

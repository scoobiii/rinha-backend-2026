<!--
Arquivo:      README.md
Diretório:    /resources
Responsável:  Mex — GOS3 · MEx Energia
Versão:       1.0.0
Data:         2026-07-06
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# resources/ — Dataset oficial da Rinha 2026

Fonte: repositório oficial
[zanfranceschi/rinha-de-backend-2026](https://github.com/zanfranceschi/rinha-de-backend-2026),
documentado em `docs/br/DATASET.md`. Estes arquivos são **especificação
técnica publicada pela própria competição** (constantes de normalização e
tabela de risco por MCC) — não são conteúdo autoral, são parâmetros
necessários para qualquer implementação correta funcionar.

| Arquivo | Origem | Conteúdo |
|---|---|---|
| `normalization.json` | Oficial, conteúdo completo reproduzido | Constantes de normalização das 14 dimensões |
| `mcc_risk.json` | Oficial, conteúdo completo reproduzido | Score de risco por categoria de comerciante (MCC) |
| `references.json.gz` | **NÃO incluído neste repo** — baixar com `scripts/download_dataset.sh` | 100.000 vetores rotulados (`fraud`/`legit`), ~1,6 MB comprimido |
| `example-references.json` | **NÃO incluído neste repo** — baixar com `scripts/download_dataset.sh` | Recorte pequeno descomprimido, mesmo formato, útil para teste rápido |

## Por que `references.json.gz` não está versionado aqui

O arquivo tem ~10 MB descomprimido — não faz sentido versionar binário de
dataset no git. Ele é baixado do repositório oficial via
`scripts/download_dataset.sh` (ver `docs/DEVOPS.md`) e fica em
`.gitignore`.

## Regra de uso (ver CONSTRAINTS.md, Regra #0)

Estes três arquivos **não mudam durante o teste oficial** — a documentação
oficial recomenda explicitamente pré-processá-los no build ou no startup
(descomprimir, indexar, construir estrutura de busca). É **proibido**:

- Usar payloads recebidos durante o teste como se fossem parte deste dataset
- Substituir a busca vetorial real por qualquer mock/stub/valor fixo

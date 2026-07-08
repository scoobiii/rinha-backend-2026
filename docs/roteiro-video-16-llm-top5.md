<!--
Arquivo:      roteiro-video-16-llm-top5.md
Diretório:    /docs
Responsável:  Mex — GOS3 · MEx Energia
Versão:       1.0.0
Data:         2026-07-07
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# Roteiro de Vídeo — "Como empatar 16 LLMs no top 5 da Rinha 2026"

Formato: talking-head estilizado, boca sincronizada (lip-sync), estética
**voxel/lego low-poly** (referência: bonecos rígidos, blocado, tipo
soldadinho — o mesmo estilo cru/propositalmente barato usado em animações
satíricas toy-style que circularam mostrando o Trump). Aqui o personagem é
seu, ninguém real é retratado — é só a linguagem visual "boneco de bloco".

## Personagem — ficha técnica

| Campo | Especificação |
|---|---|
| Nome | **Akita-GOS3** (mascote do projeto, referência direta à metodologia "Akita Vibe Coding") |
| Base | Cabeça de Akita (orelhas triangulares eretas, focinho curto) construída em blocos retangulares tipo lego — voxel art, sem curvas suaves |
| Corpo | Torso de "soldadinho de bloco" — braços e pernas cilíndricos segmentados, articulação visível (não esconder as juntas, é parte do estilo) |
| Paleta | Preto fosco + verde-neon (#00FF9C) nos detalhes (mesma paleta do dark glassmorphism que você já usa nos dashboards) + roxo elétrico (#7B2FFF) como cor secundária |
| Textura | Plástico fosco, sem brilho especular forte — leve ruído/grain pra não parecer render 3D "limpo" demais, reforça o estilo cru |
| Boca | Peça retangular separada, articulada só no eixo vertical (abre/fecha), sem lip-sync fonético fino — sincroniza em batidas de sílaba, não em fonema (isso é característica do estilo, não limitação a esconder) |
| Olhos | Dois retângulos verde-neon, sem pupila — pisca com "corte seco" (2 frames), não com transição suave |
| Voz | pt-BR, terso, técnico, sem entonação exagerada — mesmo tom que você já usa nos vídeos |

## Especificações técnicas

| Item | Valor |
|---|---|
| Resolução | 1080×1920 (vertical, 9:16) — Shorts/Reels/TikTok |
| Alternativa | 1920×1080 (16:9) se for pro YouTube longo — mesmas cenas, só reenquadrar |
| Frame rate | 30fps (24fps também funciona bem no estilo voxel, dá aspecto mais "stop motion") |
| Duração total | ~75s (14 cenas, ~5s cada em média) |
| Áudio | Voz gravada ou TTS pt-BR primeiro, depois lip-sync na batida silábica sobre o áudio final |
| Fundo | Estúdio virtual escuro, glassmorphism com painéis semi-transparentes flutuando atrás do personagem (gráficos do leaderboard, trechos de código) |

## Roteiro cena a cena

### Cena 1 — Gancho
**Enquadramento:** Close-up (rosto/boca ocupando 60% do quadro), câmera fixa, leve zoom-in de 2%.
**Fala:** "Rodei 16 configs de IA em paralelo pra tentar entrar no top 5 da Rinha de Backend 2026. Não é sobre achar o prompt mágico."
**Visual de fundo:** Painel translúcido com o texto "#RinhaBackend2026" pulsando em verde-neon.

### Cena 2 — O leaderboard real
**Enquadramento:** Medium shot (cintura pra cima), personagem vira o corpo 15° em direção a um painel lateral.
**Fala:** "O leaderboard oficial fechou assim: top 4 empatados em 6 mil pontos. Diferença só no p99 — 0.42 a 0.46 milissegundos."
**Visual de fundo:** Tabela do leaderboard renderizada em blocos (estilo voxel também, números grandes).

### Cena 3 — O erro do agente sozinho
**Enquadramento:** Close-up, câmera gira 10° pra esquerda (efeito "confidência").
**Fala:** "Erro número um: deixar UM agente sozinho em loop por 24 horas. Ele casa rápido com uma solução. Viés de commitment."
**Visual de fundo:** Ícone de um "robozinho" sozinho preso num círculo fechado (loop visual).

### Cena 4 — A virada: 16 agentes
**Enquadramento:** Wide shot — câmera afasta revelando 16 miniaturas do próprio Akita-GOS3 enfileiradas atrás dele.
**Fala:** "Solução: 16 agentes em paralelo, cada um numa branch isolada, competindo contra o mesmo harness."
**Visual de fundo:** As 16 miniaturas acendem os olhos em sequência (efeito cascata).

### Cena 5 — O agente-controle
**Enquadramento:** Medium shot, personagem aponta pra uma das 16 miniaturas, que fica destacada em roxo.
**Fala:** "Um dos 16 não recebe nenhuma otimização. É o controle. Sem ele você não sabe se o ganho é real ou é ruído."
**Visual de fundo:** Gráfico simples de barras comparando "com controle" vs "sem controle".

### Cena 6 — Regra #0
**Enquadramento:** Close-up extremo (só rosto), sem movimento de câmera — pausa dramática do estilo.
**Fala:** "Regra zero, inegociável: nenhum agente usa mock, stub ou score fixo. Isso não é atalho. É fraude."
**Visual de fundo:** Texto "REGRA #0" em vermelho, único momento de cor quente no vídeo inteiro (contraste proposital).

### Cena 7 — Zona cinza
**Enquadramento:** Medium shot, personagem inclina a cabeça (gesto de dúvida, bem mecânico/travado — característico do voxel).
**Fala:** "Toda decisão em zona cinza pausa o agente e pede confirmação humana. Deixar a IA decidir sozinha aí é desclassificação."
**Visual de fundo:** Sinal de interrogação piscando em amarelo sobre um ícone de load balancer.

### Cena 8 — O harness
**Enquadramento:** Wide shot, painel de fluxograma aparece por trás (4 caixas conectadas).
**Fala:** "Toda tentativa passa pelo mesmo caminho: checa restrição, testa contrato, mede performance local, só então builda."
**Visual de fundo:** Fluxograma animado com as 4 etapas acendendo em sequência.

### Cena 9 — O que funcionou de verdade
**Enquadramento:** Close-up, corte seco (jump cut voxel — característica do estilo, não suaviza).
**Fala:** "O que moveu o p99 de verdade: kd-tree no lugar de força bruta, AVX2 na distância, e fd passing no load balancer."
**Visual de fundo:** Três ícones técnicos aparecem um a um (árvore, chip, seta de transferência).

### Cena 10 — Validação real, não fé
**Enquadramento:** Medium shot, personagem bate a mão de bloco na "mesa" (gesto de ênfase, movimento rígido em 3 frames).
**Fala:** "Testei o fd passing de ponta a ponta antes de confiar. Não assumi. Validei."
**Visual de fundo:** Terminal com log real aparecendo (worker1, worker2, worker1, worker2).

### Cena 11 — Honestidade sobre o que não foi testado
**Enquadramento:** Close-up, personagem levemente abaixa a cabeça (gesto de "ok, verdade completa").
**Fala:** "O que eu não testei, não finjo que testei. Isso vai pro topo do checklist antes de qualquer submissão."
**Visual de fundo:** Checklist com um item em destaque piscando, ainda não marcado.

### Cena 12 — A virada de conceito
**Enquadramento:** Wide shot, câmera sobe levemente (crane up sutil, 5% de movimento).
**Fala:** "Empatar 16 LLMs não é sobre ter 16 IAs geniais. É sobre um harness rígido o bastante pra achar a solução real."
**Visual de fundo:** As 16 miniaturas se fundem visualmente numa única barra de progresso subindo.

### Cena 13 — CTA / assinatura
**Enquadramento:** Close-up final, câmera centralizada, sem movimento — quadro de encerramento.
**Fala:** "Monte seu CONSTRAINTS.md antes de soltar qualquer agente em loop. GOS3, MEx Energia."
**Visual de fundo:** Logo/assinatura GOS3 em verde-neon sobre fundo preto.

### Cena 14 — Créditos rápidos (opcional, 2s)
**Enquadramento:** Tela cheia, sem personagem.
**Texto:** "Repositório completo: github.com/scoobiii/rinha-backend-2026"

## Notas de produção

- **Por que voxel/lego e não realista:** o estilo blocado esconde bem as
  imperfeições de lip-sync automatizado (a boca não precisa fazer fonemas
  finos) e reforça visualmente a ideia de "peças modulares" que é a
  metáfora central do vídeo (16 agentes, harness, camadas removidas).
- **Ritmo:** cortes secos (2-3 frames), sem transição suave — é
  característica do estilo, não economia de produção. Suavizar os cortes
  quebra a linguagem visual.
- **Cor:** só a Cena 6 (Regra #0) usa cor quente — reserva o vermelho pra
  esse único momento de tensão do roteiro inteiro.

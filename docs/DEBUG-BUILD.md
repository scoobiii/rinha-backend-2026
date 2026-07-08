<!--
Arquivo:      DEBUG-BUILD.md
Diretório:    /docs
Responsável:  Mex — GOS3 · MEx Energia
Versão:       1.0.0
Data:         2026-07-07
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# Playbook de Debug — Primeiro `cargo build` real

Este repo chegou até aqui sem `rustc` disponível no ambiente de edição —
ver `CONSTRAINTS.md`, seção "Auto-avaliação técnica". Este documento existe
para que o primeiro build real seja rápido de destravar, não uma expedição.

## Ordem de execução (não pule etapas)

```bash
cd api
cargo check                 # 1. mais rápido, pega erro de tipo sem gerar binário
cargo build --release       # 2. só depois do check limpo
cargo test                  # 3. valida simd::escalar_e_avx2_concordam
```

Rodar `cargo check` primeiro economiza tempo — ele pega ~90% dos erros de
tipo sem esperar o LTO do build release (que é lento).

## Onde o risco está concentrado

`kdtree.rs` e a lógica de negócio (vetorização, KNN, HTTP) seguem o mesmo
padrão da v2.0.0, que era mais simples e convencional. **O risco real está
em `main.rs`, na parte que usa `libc` diretamente** (epoll, SCM_RIGHTS,
socket Unix) — é código de sistema de baixo nível, sem framework, e não
foi verificado por um compilador ainda.

## Sintoma → causa provável → correção

### 1. `mismatched types` em `msg.msg_controllen = cmsg_buf.len();`

**Causa provável:** o campo `msg_controllen` de `libc::msghdr` é `size_t`
na maioria das plataformas, mas o tipo Rust exato pode variar
(`usize` vs algo que precise de cast explícito).

**Correção:**
```rust
msg.msg_controllen = cmsg_buf.len() as _;
```
O `as _` deixa o compilador inferir o tipo certo do campo, sem você
precisar saber qual é de antemão. Aplique o mesmo padrão em qualquer
atribuição numérica para campos de `msghdr`/`cmsghdr` que der erro de tipo.

### 2. `cannot borrow `events[i]` ... packed field` ou aviso sobre `#[repr(packed)]`

**Causa provável:** em x86_64, `libc::epoll_event` é `#[repr(packed)]`.
Tirar uma **referência** de um campo dele (`&events[i].u64`) é proibido;
**ler o valor** (`events[i].u64`) é permitido.

**Correção:** garanta que você só faz `let fd = events[i].u64 as RawFd;`
(cópia de valor) e nunca `&events[i].u64` ou `&mut events[i].u64`. Se
precisar passar o evento inteiro para uma função, passe `&events[i]`
(referência à struct toda), não a um campo dela.

### 3. `no method named 'CMSG_FIRSTHDR' found` ou erro de assinatura nas macros CMSG

**Causa provável:** `CMSG_FIRSTHDR`, `CMSG_DATA`, `CMSG_SPACE`, `CMSG_LEN`
são funções `unsafe` no crate `libc` (não macros C reais) — a assinatura
exata (`*const msghdr` vs `*mut msghdr`) já é tratada por coerção implícita
de `&msg`/`&mut msg`, mas se o Rust reclamar de mutabilidade:

**Correção:** troque `libc::CMSG_FIRSTHDR(&msg)` por
`libc::CMSG_FIRSTHDR(&mut msg)` (ou vice-versa) — a diferença entre
`*const` e `*mut` costuma resolver isso. Não precisa mudar mais nada.

### 4. `expected u8, found i8` (ou o contrário) ao preencher `sun_path`

**Causa provável:** `libc::c_char` é `i8` em x86_64/ARM64 Linux, mas o
literal de bytes (`CString::as_bytes_with_nul()`) retorna `u8`.

**Correção:** o cast já está no código (`*b as libc::c_char`) — se ainda
assim der erro, confirme que `b` é `&u8` (do `.iter()` sobre `&[u8]`) e que
o cast é `(*b) as libc::c_char`, não `b as libc::c_char` (isso casta o
ponteiro, não o valor).

### 5. `error[E0308]` em `msg.msg_iov = &mut iov;` ou `msg.msg_name = ...`

**Causa provável:** coerção implícita de `&mut T` para `*mut T` funciona
na maioria dos casos, mas nem sempre em contexto de atribuição de campo de
struct `#[repr(C)]` externa dependendo da versão do compilador.

**Correção:** force o cast explícito:
```rust
msg.msg_iov = &mut iov as *mut libc::iovec;
```

### 6. `cargo build` falha ao baixar crates (`error: could not compile ... no matching package`)

**Causa provável:** ambiente sem rede (isso é esperado se você estiver
rodando dentro do mesmo tipo de sandbox restrito onde este repo foi
escrito) — `serde`, `serde_json`, `flate2`, `libc` precisam vir do
crates.io na primeira vez.

**Correção:** rode isso numa máquina com rede liberada (seu ambiente
Termux/ARM64 normal resolve isso). Depois de baixadas uma vez, as crates
ficam em cache local (`~/.cargo/registry`) e builds seguintes não precisam
mais de rede.

### 7. `cargo test` falha em `escalar_e_avx2_concordam` (não compila ou dá panic)

**Causa provável A (não compila):** `is_x86_feature_detected!` só existe
em alvos x86/x86_64 — se você estiver compilando nativamente em ARM64
(Termux), o teste deve pular o bloco AVX2 via `#[cfg(target_arch =
"x86_64")]`, que já está no código. Se der erro de macro não encontrada,
confirme que esse `#[cfg(...)]` não foi removido/movido por engano numa
edição futura.

**Causa provável B (panic no assert):** resultado AVX2 e escalar
divergindo de verdade — nesse caso É UM BUG REAL na implementação AVX2
(possivelmente na soma horizontal `_mm256_extractf128_ps`/`_mm_movehdup_ps`
em `simd.rs`), não um problema de ambiente. Não ignore esse teste falhando
— ver Regra #0 do `CONSTRAINTS.md`: uma busca vetorial com resultado
incorreto é tão grave quanto um stub.

### 8. Erro de link (`undefined reference to 'epoll_create1'` ou similar)

**Causa provável:** extremamente raro com a crate `libc` moderna, mas pode
acontecer em toolchains muito antigas do Rust/glibc.

**Correção:** atualize o Rust (`rustup update`) e confirme
`glibc >= 2.27` no container base (`debian:bookworm-slim` já atende isso).

## Se nada disso resolver em tempo hábil — plano B, não é derrota

O kd-tree (`kdtree.rs`) e o AVX2 (`simd.rs`) **não dependem** da parte de
epoll/SCM_RIGHTS — são módulos independentes. Se o transporte
epoll+SCM_RIGHTS estiver consumindo tempo demais, é legítimo (e documentado
como decisão, não gambiarra escondida) voltar temporariamente para o
modelo de socket TCP simples por worker (o design da v2.0.0, com
`TcpListener` + thread por conexão) **mantendo o kd-tree e o AVX2** — você
não perde as duas otimizações que mais importam pro P99 (busca vetorial
rápida) só porque o transporte mais sofisticado ainda não compilou. Anote
essa decisão em `harness/baseline.json` e reavalie depois, sem pressa.

## Checklist de saída (antes de considerar "3/3" de verdade)

- [ ] `cargo check` limpo
- [ ] `cargo build --release` limpo
- [ ] `cargo test` passou, incluindo `escalar_e_avx2_concordam`
- [ ] `docker compose up --build` sobe sem crash em nenhum dos 3 serviços
- [ ] `harness/smoke_test.sh` passa contra a stack via Docker (não só build local)
- [ ] Teste manual de carga (`harness/benchmark.sh`) roda sem erro de conexão

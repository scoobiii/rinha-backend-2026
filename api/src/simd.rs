// Arquivo:      simd.rs
// Diretório:    /api/src
// Responsável:  Mex — GOS3 · MEx Energia
// Versão:       1.0.0
// Data:         2026-07-06
// Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
// Glossário:    ver docs/GLOSSARIO.md
//
// Distância euclidiana ao quadrado, acelerada por AVX2 quando disponível.
// Gap #2 identificado no SWOT contra o top 5 do leaderboard (tag "avx2").
//
// IMPORTANTE — honestidade técnica de arquitetura (ver docs/DEVOPS.md):
// o ambiente de desenvolvimento do Mex é ARM64 (Termux/proot), que NÃO tem
// AVX2 (é extensão exclusiva de x86). Este arquivo compila e RODA em ARM64
// (usa o caminho escalar de fallback), e usa AVX2 de verdade quando o
// binário roda em amd64 — que é a arquitetura de submissão da Rinha
// (CONSTRAINTS.md: "imagens compatíveis com linux-amd64"). A detecção é em
// runtime via `is_x86_feature_detected!`, então o MESMO binário compilado
// para amd64 funciona em qualquer CPU amd64, com ou sem AVX2.
//
// DIMS = 16 (14 dimensões reais do desafio + 2 de padding sempre 0.0, só
// para alinhar com registradores AVX2 de 8 floats). Ver main.rs.

use crate::DIMS;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Distância euclidiana ao quadrado entre dois vetores de DIMS=16 floats.
/// Escolhe AVX2 em runtime se disponível (x86_64); caso contrário, escalar.
#[inline]
pub fn dist_sq(a: &[f32; DIMS], b: &[f32; DIMS]) -> f32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { dist_sq_avx2(a, b) };
        }
    }
    dist_sq_scalar(a, b)
}

fn dist_sq_scalar(a: &[f32; DIMS], b: &[f32; DIMS]) -> f32 {
    let mut sum = 0f32;
    for i in 0..DIMS {
        let d = a[i] - b[i];
        sum += d * d;
    }
    sum
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn dist_sq_avx2(a: &[f32; DIMS], b: &[f32; DIMS]) -> f32 {
    debug_assert_eq!(DIMS, 16, "dist_sq_avx2 assume DIMS=16 (2x8 lanes)");

    let a0 = _mm256_loadu_ps(a.as_ptr());
    let b0 = _mm256_loadu_ps(b.as_ptr());
    let a1 = _mm256_loadu_ps(a.as_ptr().add(8));
    let b1 = _mm256_loadu_ps(b.as_ptr().add(8));

    let d0 = _mm256_sub_ps(a0, b0);
    let d1 = _mm256_sub_ps(a1, b1);

    let sq0 = _mm256_mul_ps(d0, d0);
    let sq1 = _mm256_mul_ps(d1, d1);

    let sum = _mm256_add_ps(sq0, sq1);

    let hi = _mm256_extractf128_ps(sum, 1);
    let lo = _mm256_castps256_ps128(sum);
    let sum128 = _mm_add_ps(hi, lo);
    let shuf = _mm_movehdup_ps(sum128);
    let sums = _mm_add_ps(sum128, shuf);
    let shuf2 = _mm_movehl_ps(shuf, sums);
    let result = _mm_add_ss(sums, shuf2);

    _mm_cvtss_f32(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escalar_e_avx2_concordam() {
        let a: [f32; DIMS] = [0.1, 0.9, 0.3, 0.7, 0.5, -1.0, -1.0, 0.2, 1.0, 0.4, 0.6, 0.8, 0.0, 1.0, 0.0, 0.0];
        let b: [f32; DIMS] = [0.9, 0.1, 0.3, 0.2, 0.5, -1.0, -1.0, 0.8, 0.0, 0.4, 0.1, 0.3, 1.0, 0.0, 0.0, 0.0];

        let scalar = dist_sq_scalar(&a, &b);
        assert!(scalar >= 0.0, "distância ao quadrado não pode ser negativa: {}", scalar);
        assert!(scalar > 0.0, "vetores diferentes não podem ter distância zero");

        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") {
            let avx2 = unsafe { dist_sq_avx2(&a, &b) };
            assert!((scalar - avx2).abs() < 1e-5, "escalar={} avx2={}", scalar, avx2);
        }
    }
}

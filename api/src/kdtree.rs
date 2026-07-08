// Arquivo:      kdtree.rs
// Diretório:    /api/src
// Responsável:  Mex — GOS3 · MEx Energia
// Versão:       1.0.0
// Data:         2026-07-06
// Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
// Glossário:    ver docs/GLOSSARIO.md
//
// kd-tree particionado para KNN exato (k=5) sobre o dataset de referência.
// Substitui a v2.0.0, que fazia busca por força bruta O(n) — o gap #1
// identificado no SWOT contra o top 5 do leaderboard (que usa
// "partitioned kd-tree"). Construção O(n log n) uma vez no startup
// (usa select_nth_unstable_by, não sort completo por nó). Consulta com
// poda por plano de corte — não visita o dataset inteiro por requisição.

use crate::simd::dist_sq;
use crate::DIMS;

pub struct KdTree {
    root: Option<Box<Node>>,
}

struct Node {
    idx: usize, // índice no array `vectors` do ponto guardado neste nó
    axis: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

/// Wrapper para permitir Ord em f32 dentro do BinaryHeap (sem NaN esperado —
/// distâncias euclidianas ao quadrado nunca são NaN para entradas válidas).
#[derive(Copy, Clone, PartialEq)]
struct OrdF32(f32);
impl Eq for OrdF32 {}
impl PartialOrd for OrdF32 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl Ord for OrdF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl KdTree {
    pub fn build(vectors: &[[f32; DIMS]]) -> Self {
        let mut indices: Vec<usize> = (0..vectors.len()).collect();
        let root = Self::build_rec(&mut indices, vectors, 0);
        KdTree { root }
    }

    fn build_rec(indices: &mut [usize], vectors: &[[f32; DIMS]], depth: usize) -> Option<Box<Node>> {
        if indices.is_empty() {
            return None;
        }
        if indices.len() == 1 {
            return Some(Box::new(Node {
                idx: indices[0],
                axis: depth % DIMS,
                left: None,
                right: None,
            }));
        }

        let axis = depth % DIMS;
        let mid = indices.len() / 2;

        // select_nth_unstable_by: mediana em O(n) médio, sem ordenar tudo.
        indices.select_nth_unstable_by(mid, |&a, &b| {
            vectors[a][axis].partial_cmp(&vectors[b][axis]).unwrap()
        });

        let median_idx = indices[mid];
        let (left, right_with_mid) = indices.split_at_mut(mid);
        let right = &mut right_with_mid[1..];

        Some(Box::new(Node {
            idx: median_idx,
            axis,
            left: Self::build_rec(left, vectors, depth + 1),
            right: Self::build_rec(right, vectors, depth + 1),
        }))
    }

    /// Retorna os `k` vizinhos mais próximos: Vec<(distância², índice)>,
    /// ordenado do mais próximo para o mais distante.
    pub fn knn(&self, query: &[f32; DIMS], vectors: &[[f32; DIMS]], k: usize) -> Vec<(f32, usize)> {
        use std::collections::BinaryHeap;
        // Max-heap por distância: topo é o pior dos k melhores candidatos
        // encontrados até agora — permite podar candidatos piores rápido.
        let mut heap: BinaryHeap<(OrdF32, usize)> = BinaryHeap::with_capacity(k + 1);

        Self::search_rec(&self.root, query, vectors, k, &mut heap);

        let mut result: Vec<(f32, usize)> = heap.into_iter().map(|(d, i)| (d.0, i)).collect();
        result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        result
    }

    fn search_rec(
        node: &Option<Box<Node>>,
        query: &[f32; DIMS],
        vectors: &[[f32; DIMS]],
        k: usize,
        heap: &mut std::collections::BinaryHeap<(OrdF32, usize)>,
    ) {
        let node = match node {
            Some(n) => n,
            None => return,
        };

        let d = dist_sq(query, &vectors[node.idx]);
        if heap.len() < k {
            heap.push((OrdF32(d), node.idx));
        } else if d < heap.peek().unwrap().0 .0 {
            heap.pop();
            heap.push((OrdF32(d), node.idx));
        }

        let diff = query[node.axis] - vectors[node.idx][node.axis];
        let (near, far) = if diff < 0.0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        Self::search_rec(near, query, vectors, k, heap);

        // Poda: só entra no lado "far" se o plano de corte estiver mais
        // perto do que o pior candidato atual (ou se ainda não temos k
        // candidatos). Isso é o que evita visitar o dataset inteiro.
        let plane_dist_sq = diff * diff;
        if heap.len() < k || plane_dist_sq < heap.peek().unwrap().0 .0 {
            Self::search_rec(far, query, vectors, k, heap);
        }
    }
}

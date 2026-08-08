use nalgebra::Vector2;
use std::cell::RefCell;

/// Squared distance to prevent unnecessary square root operations
#[inline(always)]
fn dist_sq(a: &Vector2<f64>, b: &Vector2<f64>) -> f64 {
    (a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)
}

/// Internal scratchpad buffers reused across UI frames
struct HomologyWorkspace {
    bit_grid: Vec<u64>,
    edges: Vec<(usize, usize)>,
    row_starts: Vec<usize>,
    visited: Vec<bool>,
    stack: Vec<usize>,
    pivot_columns: Vec<Option<Vec<usize>>>,
    col_buffer: Vec<usize>,
    next_col_buffer: Vec<usize>,
}

impl HomologyWorkspace {
    fn new() -> Self {
        Self {
            bit_grid: Vec::new(),
            edges: Vec::with_capacity(16_000),
            row_starts: Vec::with_capacity(2_000),
            visited: Vec::with_capacity(2_000),
            stack: Vec::with_capacity(2_000),
            pivot_columns: Vec::new(),
            col_buffer: Vec::with_capacity(64),
            next_col_buffer: Vec::with_capacity(64),
        }
    }
}

// Thread-local workspace keeps your original `pub fn compute_betti(points, eps)` signature intact
thread_local! {
    static WORKSPACE: RefCell<HomologyWorkspace> = RefCell::new(HomologyWorkspace::new());
}

pub fn compute_betti(points: &[Vector2<f64>], eps: f64) -> (usize, usize) {
    WORKSPACE.with(|ws_cell| {
        let mut ws = ws_cell.borrow_mut();
        compute_betti_internal(points, eps, &mut ws)
    })
}

fn compute_betti_internal(
    points: &[Vector2<f64>],
    eps: f64,
    ws: &mut HomologyWorkspace,
) -> (usize, usize) {
    let n = points.len();
    if n == 0 {
        return (0, 0);
    }

    let eps_sq = eps * eps;
    let words_per_row = (n + 63) / 64;
    let bit_grid_len = n * words_per_row;

    // --- 1. Reset Workspace Scratch Buffers ---
    ws.bit_grid.clear();
    ws.bit_grid.resize(bit_grid_len, 0);

    ws.edges.clear();
    if ws.row_starts.len() < n + 1 {
        ws.row_starts.resize(n + 1, 0);
    }

    ws.visited.clear();
    ws.visited.resize(n, false);
    ws.stack.clear();

    let set_bit = |grid: &mut [u64], row: usize, col: usize| {
        let idx = row * words_per_row + (col / 64);
        grid[idx] |= 1 << (col % 64);
    };

    let get_bit = |grid: &[u64], row: usize, col: usize| -> bool {
        let idx = row * words_per_row + (col / 64);
        (grid[idx] & (1 << (col % 64))) != 0
    };

    // --- 2. Adjacency Graph & Compressed CSR Edge Lookup ---
    for i in 0..n {
        ws.row_starts[i] = ws.edges.len();
        for j in (i + 1)..n {
            if dist_sq(&points[i], &points[j]) <= eps_sq {
                set_bit(&mut ws.bit_grid, i, j);
                set_bit(&mut ws.bit_grid, j, i);
                ws.edges.push((i, j));
            }
        }
    }
    ws.row_starts[n] = ws.edges.len();
    let num_edges = ws.edges.len();

    let get_edge_id = |u: usize, v: usize, edges: &[(usize, usize)], row_starts: &[usize]| -> usize {
        let start = row_starts[u];
        let end = row_starts[u + 1];
        let slice = &edges[start..end];

        match slice.binary_search_by_key(&v, |&(_, target)| target) {
            Ok(idx) => start + idx,
            Err(_) => usize::MAX,
        }
    };

    // --- 3. Betti-0 Calculation (Connected Components) ---
    let mut components = 0;
    for i in 0..n {
        if !ws.visited[i] {
            components += 1;
            ws.stack.push(i);

            while let Some(node) = ws.stack.pop() {
                if !ws.visited[node] {
                    ws.visited[node] = true;

                    let row_offset = node * words_per_row;
                    for w in 0..words_per_row {
                        let mut word = ws.bit_grid[row_offset + w];
                        while word != 0 {
                            let tz = word.trailing_zeros() as usize;
                            let neighbor = w * 64 + tz;
                            if !ws.visited[neighbor] {
                                ws.stack.push(neighbor);
                            }
                            word &= word - 1;
                        }
                    }
                }
            }
        }
    }

    // --- 4. Total 1-Cycle Rank (dim ker d1) ---
    let graph_cycles = num_edges as isize - n as isize + components as isize;
    if graph_cycles <= 0 {
        return (components, 0);
    }

    // Reset pivot table
    if ws.pivot_columns.len() < num_edges {
        ws.pivot_columns.resize_with(num_edges, || None);
    } else {
        for pivot in ws.pivot_columns.iter_mut().take(num_edges) {
            *pivot = None;
        }
    }

    let mut rank_d2 = 0;

    // --- 5. Unique Triangles & F2 Gaussian Elimination ---
    for i in 0..n {
        let i_offset = i * words_per_row;
        for j in (i + 1)..n {
            if get_bit(&ws.bit_grid, i, j) {
                let j_offset = j * words_per_row;

                for w in 0..words_per_row {
                    let mut common = ws.bit_grid[i_offset + w] & ws.bit_grid[j_offset + w];
                    while common != 0 {
                        let tz = common.trailing_zeros() as usize;
                        let k = w * 64 + tz;

                        // Restrict k > j to enforce strict unique triangle ordering (i < j < k)
                        if k > j {
                            let e_ij = get_edge_id(i, j, &ws.edges, &ws.row_starts);
                            let e_jk = get_edge_id(j, k, &ws.edges, &ws.row_starts);
                            let e_ik = get_edge_id(i, k, &ws.edges, &ws.row_starts);

                            ws.col_buffer.clear();
                            ws.col_buffer.push(e_ij);
                            ws.col_buffer.push(e_jk);
                            ws.col_buffer.push(e_ik);
                            ws.col_buffer.sort_unstable_by(|a, b| b.cmp(a));

                            // Eliminate column over GF(2)
                            while let Some(&pivot_row) = ws.col_buffer.first() {
                                if let Some(ref existing_pivot) = ws.pivot_columns[pivot_row] {
                                    ws.next_col_buffer.clear();

                                    let mut i1 = 0;
                                    let mut i2 = 0;

                                    while i1 < ws.col_buffer.len() && i2 < existing_pivot.len() {
                                        let r1 = ws.col_buffer[i1];
                                        let r2 = existing_pivot[i2];

                                        if r1 == r2 {
                                            i1 += 1;
                                            i2 += 1;
                                        } else if r1 > r2 {
                                            ws.next_col_buffer.push(r1);
                                            i1 += 1;
                                        } else {
                                            ws.next_col_buffer.push(r2);
                                            i2 += 1;
                                        }
                                    }
                                    while i1 < ws.col_buffer.len() {
                                        ws.next_col_buffer.push(ws.col_buffer[i1]);
                                        i1 += 1;
                                    }
                                    while i2 < existing_pivot.len() {
                                        ws.next_col_buffer.push(existing_pivot[i2]);
                                        i2 += 1;
                                    }

                                    std::mem::swap(&mut ws.col_buffer, &mut ws.next_col_buffer);
                                } else {
                                    ws.pivot_columns[pivot_row] = Some(ws.col_buffer.clone());
                                    rank_d2 += 1;
                                    break;
                                }
                            }
                        }

                        common &= common - 1;
                    }
                }
            }
        }
    }

    // --- 6. Exact Betti-1 ---
    let betti1 = (graph_cycles as usize).saturating_sub(rank_d2);

    (components, betti1)
}
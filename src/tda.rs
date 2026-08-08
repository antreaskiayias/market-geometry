use nalgebra::Vector2;

/// Simple pairwise distance
fn dist(a: &Vector2<f64>, b: &Vector2<f64>) -> f64 {
    (a - b).norm()
}

/// Build Vietoris–Rips 0- and 1-dimensional persistence
pub fn compute_betti(points: &[Vector2<f64>], eps: f64) -> (usize, usize) {
    let n = points.len();
    if n == 0 {
        return (0, 0);
    }

    // Betti-0: connected components under eps-neighborhood graph
    let mut visited = vec![false; n];
    let mut components = 0;

    for i in 0..n {
        if !visited[i] {
            components += 1;
            dfs(i, &mut visited, points, eps);
        }
    }

    // Betti-1: count cycles in the eps-neighborhood graph
    // Using formula: Betti1 = E - V + C
    let mut edges = 0;

    for i in 0..n {
        for j in (i + 1)..n {
            if dist(&points[i], &points[j]) <= eps {
                edges += 1;
            }
        }
    }

    let betti1 = edges as isize - n as isize + components as isize;
    let betti1 = betti1.max(0) as usize;

    (components, betti1)
}

fn dfs(
    i: usize,
    visited: &mut Vec<bool>,
    points: &[Vector2<f64>],
    eps: f64,
) {
    visited[i] = true;

    for j in 0..points.len() {
        if !visited[j] && dist(&points[i], &points[j]) <= eps {
            dfs(j, visited, points, eps);
        }
    }
}

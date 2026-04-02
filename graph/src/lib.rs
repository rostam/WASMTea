use wasm_bindgen::prelude::*;
use serde::Serialize;
use std::collections::{VecDeque, HashMap};

const VERTEX_RADIUS: f64 = 22.0;
const EXACT_ALGO_LIMIT: usize = 25; // max vertices for NP-hard exact algorithms

#[derive(Serialize, Clone)]
pub struct Vertex {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub label: String,
}

#[derive(Serialize, Clone)]
pub struct Edge {
    pub id: u32,
    pub from: u32,
    pub to: u32,
}

#[derive(Serialize)]
struct Analysis {
    vertex_count: usize,
    edge_count: usize,
    density: f64,
    is_connected: bool,
    connected_components: usize,
    min_degree: u32,
    max_degree: u32,
    avg_degree: f64,
    degree_sequence: Vec<u32>,
    is_bipartite: bool,
    has_eulerian_circuit: bool,
    has_eulerian_path: bool,
    diameter: i64,          // -1 = disconnected/undefined
    chromatic_number_greedy: u32,
    max_independent_set: u32,
    min_vertex_cover: u32,
    max_clique: u32,
    exact_computed: bool,   // false when graph is too large for NP-hard algorithms
}

#[wasm_bindgen]
pub struct Graph {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    next_vertex_id: u32,
    next_edge_id: u32,
}

// ── private helpers ──────────────────────────────────────────────────────────

impl Graph {
    fn build_adj(&self) -> (usize, Vec<Vec<bool>>) {
        let n = self.vertices.len();
        let id_to_idx: HashMap<u32, usize> = self.vertices.iter()
            .enumerate().map(|(i, v)| (v.id, i)).collect();
        let mut adj = vec![vec![false; n]; n];
        for e in &self.edges {
            if let (Some(&i), Some(&j)) = (id_to_idx.get(&e.from), id_to_idx.get(&e.to)) {
                adj[i][j] = true;
                adj[j][i] = true;
            }
        }
        (n, adj)
    }

    fn bfs_distances(start: usize, adj: &[Vec<bool>]) -> Vec<i64> {
        let n = adj.len();
        let mut dist = vec![-1i64; n];
        dist[start] = 0;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            for u in 0..n {
                if adj[v][u] && dist[u] == -1 {
                    dist[u] = dist[v] + 1;
                    queue.push_back(u);
                }
            }
        }
        dist
    }

    fn connected_components(adj: &[Vec<bool>]) -> usize {
        let n = adj.len();
        let mut visited = vec![false; n];
        let mut count = 0;
        for start in 0..n {
            if !visited[start] {
                count += 1;
                let mut queue = VecDeque::new();
                queue.push_back(start);
                visited[start] = true;
                while let Some(v) = queue.pop_front() {
                    for u in 0..n {
                        if adj[v][u] && !visited[u] {
                            visited[u] = true;
                            queue.push_back(u);
                        }
                    }
                }
            }
        }
        count
    }

    fn is_bipartite(adj: &[Vec<bool>]) -> bool {
        let n = adj.len();
        let mut color = vec![-1i32; n];
        for start in 0..n {
            if color[start] != -1 { continue; }
            color[start] = 0;
            let mut queue = VecDeque::new();
            queue.push_back(start);
            while let Some(v) = queue.pop_front() {
                for u in 0..n {
                    if adj[v][u] {
                        if color[u] == -1 {
                            color[u] = 1 - color[v];
                            queue.push_back(u);
                        } else if color[u] == color[v] {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    fn greedy_chromatic(adj: &[Vec<bool>]) -> u32 {
        let n = adj.len();
        if n == 0 { return 0; }
        let mut colors = vec![0u32; n];
        for v in 0..n {
            let used: std::collections::HashSet<u32> = (0..n)
                .filter(|&u| adj[v][u] && colors[u] != 0)
                .map(|u| colors[u])
                .collect();
            colors[v] = (1..).find(|c| !used.contains(c)).unwrap();
        }
        *colors.iter().max().unwrap_or(&0)
    }

    // Exact MIS via branch-and-bound
    fn mis(adj: &[Vec<bool>], candidates: &[usize], size: u32) -> u32 {
        if candidates.is_empty() { return size; }
        let v = candidates[0];
        let rest = &candidates[1..];
        // include v: new candidates = rest minus v's neighbours
        let with_cands: Vec<usize> = rest.iter().copied().filter(|&u| !adj[v][u]).collect();
        let with_v = Self::mis(adj, &with_cands, size + 1);
        // exclude v
        let without_v = Self::mis(adj, rest, size);
        with_v.max(without_v)
    }

    // Exact max clique via branch-and-bound
    fn max_clique(adj: &[Vec<bool>], candidates: &[usize], size: u32) -> u32 {
        if candidates.is_empty() { return size; }
        let mut best = size;
        for (i, &v) in candidates.iter().enumerate() {
            if size + (candidates.len() - i) as u32 <= best { break; }
            let new_cands: Vec<usize> = candidates[i + 1..]
                .iter().copied().filter(|&u| adj[v][u]).collect();
            let result = Self::max_clique(adj, &new_cands, size + 1);
            best = best.max(result);
        }
        best
    }
}

// ── public WASM API ──────────────────────────────────────────────────────────

#[wasm_bindgen]
impl Graph {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Graph {
        Graph {
            vertices: Vec::new(),
            edges: Vec::new(),
            next_vertex_id: 0,
            next_edge_id: 0,
        }
    }

    pub fn add_vertex(&mut self, x: f64, y: f64) -> u32 {
        let id = self.next_vertex_id;
        self.next_vertex_id += 1;
        self.vertices.push(Vertex { id, x, y, label: id.to_string() });
        id
    }

    pub fn add_edge(&mut self, from: u32, to: u32) -> bool {
        if from == to { return false; }
        let from_ok = self.vertices.iter().any(|v| v.id == from);
        let to_ok   = self.vertices.iter().any(|v| v.id == to);
        if !from_ok || !to_ok { return false; }
        let dup = self.edges.iter().any(|e| {
            (e.from == from && e.to == to) || (e.from == to && e.to == from)
        });
        if dup { return false; }
        let id = self.next_edge_id;
        self.next_edge_id += 1;
        self.edges.push(Edge { id, from, to });
        true
    }

    pub fn vertex_at(&self, x: f64, y: f64) -> i32 {
        for v in &self.vertices {
            let dx = v.x - x;
            let dy = v.y - y;
            if (dx * dx + dy * dy).sqrt() <= VERTEX_RADIUS {
                return v.id as i32;
            }
        }
        -1
    }

    pub fn move_vertex(&mut self, id: u32, x: f64, y: f64) {
        if let Some(v) = self.vertices.iter_mut().find(|v| v.id == id) {
            v.x = x;
            v.y = y;
        }
    }

    pub fn delete_vertex(&mut self, id: u32) {
        self.vertices.retain(|v| v.id != id);
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    pub fn delete_edge(&mut self, id: u32) {
        self.edges.retain(|e| e.id != id);
    }

    pub fn vertex_count(&self) -> u32 { self.vertices.len() as u32 }
    pub fn edge_count(&self)  -> u32 { self.edges.len() as u32 }
    pub fn vertex_radius()    -> f64 { VERTEX_RADIUS }

    pub fn vertices_json(&self) -> String {
        serde_json::to_string(&self.vertices).unwrap_or_default()
    }

    pub fn edges_json(&self) -> String {
        serde_json::to_string(&self.edges).unwrap_or_default()
    }

    /// Clear the graph and generate a named preset.
    /// `kind`: complete | cycle | path | star | wheel | petersen | bipartite | grid
    /// `n`: size parameter (ignored for petersen)
    /// `cx`, `cy`: canvas centre to layout around
    /// `radius`: layout radius in pixels
    pub fn generate(&mut self, kind: &str, n: u32, cx: f64, cy: f64, radius: f64) {
        self.vertices.clear();
        self.edges.clear();
        self.next_vertex_id = 0;
        self.next_edge_id = 0;

        let n = (n.max(2)) as usize;
        let pi = std::f64::consts::PI;

        let ring = |i: usize, total: usize, r: f64, offset: f64| -> (f64, f64) {
            let a = 2.0 * pi * i as f64 / total as f64 + offset;
            (cx + r * a.cos(), cy + r * a.sin())
        };
        let top = -pi / 2.0; // start from 12 o'clock

        match kind {
            "complete" => {
                for i in 0..n {
                    let (x, y) = ring(i, n, radius, top);
                    self.add_vertex(x, y);
                }
                for i in 0..n {
                    for j in i + 1..n {
                        self.add_edge(i as u32, j as u32);
                    }
                }
            }
            "cycle" => {
                for i in 0..n {
                    let (x, y) = ring(i, n, radius, top);
                    self.add_vertex(x, y);
                }
                for i in 0..n {
                    self.add_edge(i as u32, ((i + 1) % n) as u32);
                }
            }
            "path" => {
                let step = if n > 1 { radius * 2.0 / (n - 1) as f64 } else { 0.0 };
                for i in 0..n {
                    self.add_vertex(cx - radius + step * i as f64, cy);
                }
                for i in 0..n - 1 {
                    self.add_edge(i as u32, (i + 1) as u32);
                }
            }
            "star" => {
                self.add_vertex(cx, cy); // centre = id 0
                for i in 0..n {
                    let (x, y) = ring(i, n, radius, top);
                    self.add_vertex(x, y);
                    self.add_edge(0, (i + 1) as u32);
                }
            }
            "wheel" => {
                self.add_vertex(cx, cy); // hub = id 0
                for i in 0..n {
                    let (x, y) = ring(i, n, radius, top);
                    self.add_vertex(x, y);
                    self.add_edge(0, (i + 1) as u32);
                }
                for i in 0..n {
                    self.add_edge((i + 1) as u32, ((i + 1) % n + 1) as u32);
                }
            }
            "petersen" => {
                for i in 0..5 {
                    let (x, y) = ring(i, 5, radius, top);
                    self.add_vertex(x, y);
                }
                for i in 0..5 {
                    let (x, y) = ring(i, 5, radius * 0.45, top);
                    self.add_vertex(x, y);
                }
                for i in 0..5u32 { self.add_edge(i, (i + 1) % 5); }        // outer cycle
                for i in 0..5u32 { self.add_edge(5 + i, 5 + (i + 2) % 5); } // inner pentagram
                for i in 0..5u32 { self.add_edge(i, 5 + i); }              // spokes
            }
            "bipartite" => {
                let step = if n > 1 { radius * 2.0 / (n - 1) as f64 } else { 0.0 };
                for i in 0..n {
                    let y = cy - radius + step * i as f64;
                    self.add_vertex(cx - radius * 0.7, y);
                }
                for i in 0..n {
                    let y = cy - radius + step * i as f64;
                    self.add_vertex(cx + radius * 0.7, y);
                }
                for i in 0..n {
                    for j in 0..n {
                        self.add_edge(i as u32, (n + j) as u32);
                    }
                }
            }
            "grid" => {
                let step = if n > 1 { radius * 2.0 / (n - 1) as f64 } else { 0.0 };
                for r in 0..n {
                    for c in 0..n {
                        let x = cx - radius + step * c as f64;
                        let y = cy - radius + step * r as f64;
                        self.add_vertex(x, y);
                    }
                }
                for r in 0..n {
                    for c in 0..n {
                        let idx = (r * n + c) as u32;
                        if c + 1 < n { self.add_edge(idx, idx + 1); }
                        if r + 1 < n { self.add_edge(idx, idx + n as u32); }
                    }
                }
            }
            _ => {}
        }
    }

    /// Run all graph analyses and return a JSON report.
    pub fn analyze(&self) -> String {
        let n = self.vertices.len();
        let m = self.edges.len();

        if n == 0 {
            return serde_json::to_string(&Analysis {
                vertex_count: 0, edge_count: 0, density: 0.0,
                is_connected: true, connected_components: 0,
                min_degree: 0, max_degree: 0, avg_degree: 0.0,
                degree_sequence: vec![],
                is_bipartite: true,
                has_eulerian_circuit: false, has_eulerian_path: false,
                diameter: 0,
                chromatic_number_greedy: 0,
                max_independent_set: 0, min_vertex_cover: 0, max_clique: 0,
                exact_computed: true,
            }).unwrap_or_default();
        }

        let (_, adj) = self.build_adj();

        // Degrees
        let mut degrees: Vec<u32> = (0..n)
            .map(|i| adj[i].iter().filter(|&&x| x).count() as u32)
            .collect();
        degrees.sort_unstable_by(|a, b| b.cmp(a)); // descending for degree sequence
        let min_degree = *degrees.iter().min().unwrap();
        let max_degree = *degrees.iter().max().unwrap();
        let avg_degree = degrees.iter().sum::<u32>() as f64 / n as f64;

        // Connectivity
        let components = Self::connected_components(&adj);
        let is_connected = components == 1;

        // Density
        let max_edges = n * (n - 1) / 2;
        let density = if max_edges > 0 { m as f64 / max_edges as f64 } else { 0.0 };

        // Bipartite
        let bipartite = Self::is_bipartite(&adj);

        // Eulerian (requires connectivity, ignores isolated vertices)
        let odd_degrees = degrees.iter().filter(|&&d| d % 2 != 0).count();
        let has_eulerian_circuit = is_connected && odd_degrees == 0 && m > 0;
        let has_eulerian_path    = is_connected && odd_degrees == 2;

        // Diameter
        let diameter = if !is_connected {
            -1
        } else {
            let mut diam = 0i64;
            for s in 0..n {
                for d in Self::bfs_distances(s, &adj) {
                    if d > diam { diam = d; }
                }
            }
            diam
        };

        // Greedy chromatic number
        let chromatic = Self::greedy_chromatic(&adj);

        // NP-hard exact algorithms (only for small graphs)
        let exact_computed = n <= EXACT_ALGO_LIMIT;
        let all_vertices: Vec<usize> = (0..n).collect();

        let mis_size = if exact_computed {
            Self::mis(&adj, &all_vertices, 0)
        } else { 0 };

        let clique_size = if exact_computed {
            Self::max_clique(&adj, &all_vertices, 0)
        } else { 0 };

        serde_json::to_string(&Analysis {
            vertex_count: n,
            edge_count: m,
            density,
            is_connected,
            connected_components: components,
            min_degree,
            max_degree,
            avg_degree,
            degree_sequence: degrees,
            is_bipartite: bipartite,
            has_eulerian_circuit,
            has_eulerian_path,
            diameter,
            chromatic_number_greedy: chromatic,
            max_independent_set: mis_size,
            min_vertex_cover: n as u32 - mis_size,
            max_clique: clique_size,
            exact_computed,
        }).unwrap_or_default()
    }
}

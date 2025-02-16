use std::{io, time::Instant};

use rand::Rng;

#[derive(PartialEq)]
enum STATUS {
    PLAYING,
    WON,
    LOST,
    DRAW,
}

const FULL_GRID: u64 = 0b11111110111111101111111011111110111111101111111;
const UCTC: f64 = 2.0;
const TIME_PER_MOVE: u128 = 1000; // milliseconds

fn show_grid(p1: u64, p2: u64) {
    for y in (0..6).rev() {
        for x in 0..7 {
            let i = y * 8 + x;
            if 1 << i & p1 != 0 {
                print!("X");
            } else if 1 << i & p2 != 0 {
                print!("O");
            } else {
                print!("_");
            }
        }
        print!("\n");
    }
}

fn get_moves(p1: u64, p2: u64) -> Vec<(u64, u64)> {
    let grid = p1 | p2;
    let mut moves: Vec<(u64, u64)> = vec![];
    for x in 0..7 {
        for y in 0..6 {
            let i = y * 8 + x;
            if 1 << i & grid == 0 {
                moves.push((p2, p1 | 1 << i));
                break;
            }
        }
    }
    moves
}

fn is_winning(player: u64) -> bool {
    // horizontal
    if player & player >> 1 & player >> 2 & player >> 3 != 0 {
        return true;
    }
    // vertical
    if player & player >> 8 & player >> 16 & player >> 24 != 0 {
        return true;
    }
    // top left -> bottom right
    if player & player >> 9 & player >> 18 & player >> 27 != 0 {
        return true;
    }
    // top right -> bottom left
    if player & player >> 7 & player >> 14 & player >> 21 != 0 {
        return true;
    }
    false
}

fn get_status(p1: u64, p2: u64) -> STATUS {
    if is_winning(p1) {
        return STATUS::WON;
    }
    if is_winning(p2) {
        return STATUS::LOST;
    }
    if FULL_GRID == p1 | p2 {
        return STATUS::DRAW;
    }
    STATUS::PLAYING
}

struct Node {
    state: (u64, u64),
    children: Vec<usize>,
    parent: Option<usize>,
    score: u64,
    nb_visit: u64,
    status: STATUS,
}

fn selection(node: usize, graph: &mut Vec<Node>) -> usize {
    if graph[node].status != STATUS::PLAYING {
        return node;
    }
    let moves = get_moves(graph[node].state.0, graph[node].state.1);
    if moves.len() == 0 {
        return node;
    }
    if moves.len() == graph[node].children.len() {
        // eval each children and take the best one
        let mut best_child = None;
        let mut best_score = None;
        for child in graph[node].children.clone() {
            let value = graph[child].score as f64 / graph[child].nb_visit as f64
                + UCTC
                    * ((graph[node].nb_visit as f64).log2() / graph[child].nb_visit as f64).sqrt();
            if best_score == None || value > best_score.unwrap() {
                best_score = Some(value);
                best_child = Some(child);
            }
        }
        return selection(best_child.unwrap(), graph);
    }
    // expansion
    let child_move = moves[graph[node].children.len()];
    let child = graph.len();
    graph.push(Node {
        state: child_move,
        children: vec![],
        parent: Some(node),
        score: 0,
        nb_visit: 0,
        status: get_status(child_move.0, child_move.1),
    });
    graph[node].children.push(child);
    child
}

fn simulation(p1: u64, p2: u64) -> u64 {
    if is_winning(p2) {
        return 2;
    }
    if p1 | p2 == FULL_GRID {
        return 1;
    }
    let moves = get_moves(p1, p2);
    let (p1, p2) = moves[rand::thread_rng().gen_range(0..moves.len())];
    let result = simulation(p1, p2);
    [2, 1, 0][result as usize]
}

fn backpropagation(node: usize, graph: &mut Vec<Node>, score: u64) {
    graph[node].nb_visit += 1;
    graph[node].score += score;
    if let Some(parent) = graph[node].parent {
        backpropagation(parent, graph, [2, 1, 0][score as usize]);
    }
}

fn mcst(mut graph: Vec<Node>, root: usize, time: u128) -> (f64, (u64, u64), Vec<Node>, usize) {
    let now = Instant::now();
    while now.elapsed().as_millis() < TIME_PER_MOVE {
        let node = selection(root, &mut graph);
        let score = simulation(graph[node].state.0, graph[node].state.1);
        backpropagation(node, &mut graph, score);
    }
    let tests = graph[root].children.iter().map(|x| {
        (
            graph[*x].score as u64,
            graph[*x].nb_visit,
            graph[*x].state,
            *x,
        )
    });
    let mut best_score = None;
    let mut best = None;
    let mut best_x = None;
    for (score, nb_visit, state, x) in tests {
        let value = score as f64 / nb_visit as f64;
        if best_score == None || value > best_score.unwrap() {
            best_score = Some(value);
            best = Some(state);
            best_x = Some(x);
        }
    }
    (best_score.unwrap(), best.unwrap(), graph, best_x.unwrap())
}

fn init_graph() -> Vec<Node> {
    let mut graph: Vec<Node> = vec![];
    graph.push(Node {
        state: (0, 0),
        children: vec![],
        parent: None,
        score: 0,
        nb_visit: 0,
        status: get_status(0, 0),
    });
    graph
}

fn main() {
    let mut p1 = 0;
    let mut p2 = 0;
    let mut score = 0.0;
    let player_turn = get_player_turn();
    let mut turn = 0;
    let mut graph = init_graph();
    let mut root = 0;
    while !is_winning(p2) && p1 | p2 != FULL_GRID {
        if turn % 2 == player_turn {
            // player turn
            (p1, p2) = get_user_move(p1, p2);
            show_grid(p1, p2);
            // update graph
            let previous_root = root;
            for child in graph[root].children.clone() {
                if graph[child].state == (p1, p2) {
                    root = child;
                    break;
                }
            }
            if previous_root == root {
                graph[root].state = (p1, p2);
            }
        } else {
            // bot turn
            let previous_state = (p1, p2);
            (score, (p1, p2), graph, root) = mcst(graph, root, TIME_PER_MOVE);
            show_grid(p1, p2);
            println!("I played {}", to_user_move(previous_state, (p1, p2)));
            println!("evaluation: {score}");
        }
        turn += 1;
    }
    println!("finished")
}

fn to_user_move(previous_state: (u64, u64), new_state: (u64, u64)) -> u32 {
    let m = previous_state.0 ^ new_state.1;
    for y in 0..6 {
        for x in 0..7 {
            let i = y * 8 + x;
            if 1 << i == m {
                return x + 1;
            }
        }
    }
    unreachable!();
}

fn get_player_turn() -> i32 {
    println!("do you want to start y/n");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    if input == "y\n".to_string() || input == "Y\n".to_string() {
        return 0;
    }
    1
}

fn get_user_move(p1: u64, p2: u64) -> (u64, u64) {
    let mut is_first = true;
    loop {
        let mut input = String::new();
        if is_first {
            println!("entrez un coup (le x 1 -> 7 inclus)");
        } else {
            println!("coup entré invalide");
        }
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let value = input.replace("\n", "").trim().parse::<i32>().unwrap_or(10);
        if value < 1 || value > 7 {
            continue;
        }
        for y in 0..6 {
            let i = y * 8 + value - 1;
            if 1 << i & (p1 | p2) == 0 {
                return (p2, p1 | 1 << i);
            }
        }
        is_first = false;
    }
}

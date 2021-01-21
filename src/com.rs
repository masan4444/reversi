use super::board::{bitboard, Coordinate};
use std::sync::{Arc, Mutex};
use std::thread;

// enum EvaluateMode {
//   Count,
//   Point,
//   Compound,
// }

// pub fn search(p: u64, o: u64, pos: usize, index: usize) -> usize {

//   0
// }

// #[inline]
// pub fn choose_pos_old(p: u64, o: u64, index: usize) -> usize {
//     let mut best_pos = 0;
//     let mut max_score = isize::MIN;
//     let mut legal_patt = bitboard::legal_patt_simd(p, o);
//     while legal_patt != 0 {
//         let pos = legal_patt.trailing_zeros() as usize;
//         let rev = bitboard::rev_patt_simd(p, o, pos);
//         let score = match index {
//             _ => -nega_alpha(o ^ rev, p ^ (1u64 << pos | rev), 11, 1),
//         };
//         println!("pos: {}({}), score: {}", Coordinate::from(pos), pos, score);
//         if score > max_score {
//             best_pos = pos;
//             max_score = score;
//         }
//         legal_patt &= !(1u64 << pos);
//     }
//     best_pos
// }

#[inline]
pub fn choose_pos(p: u64, o: u64, _index: usize) -> usize {
    let mut best_pos = 0;
    let mut legal_patt = bitboard::legal_patt_simd(p, o);
    let mut alpha = isize::MIN + 1;

    while legal_patt != 0 {
        let pos = legal_patt.trailing_zeros() as usize;
        let rev = bitboard::rev_patt_simd(p, o, pos);
        let score = -_nega_alpha(
            o ^ rev,
            p ^ (1u64 << pos | rev),
            11,
            1,
            isize::MIN + 1,
            -alpha,
        );
        println!("pos: {}({}), score: {}", Coordinate::from(pos), pos, score);
        if score > alpha {
            best_pos = pos;
            alpha = score;
        }
        legal_patt &= !(1u64 << pos);
    }
    best_pos
}

#[inline]
pub fn choose_pos_concurrency(p: u64, o: u64, _index: usize) -> usize {
    let mut legal_patt = bitboard::legal_patt_simd(p, o);
    let mut alpha = isize::MIN + 1;

    let mut eldest_work_finished = false;
    let pos_and_scores = Arc::new(Mutex::new(Vec::<(usize, isize)>::new()));
    let mut handles = vec![];

    while legal_patt != 0 {
        let pos = legal_patt.trailing_zeros() as usize;
        let pos_and_scores_rc = Arc::clone(&pos_and_scores);

        let handle = thread::spawn(move || {
            println!("thread created");
            let rev = bitboard::rev_patt_simd(p, o, pos);
            let score = -_nega_alpha(
                o ^ rev,
                p ^ (1u64 << pos | rev),
                11,
                1,
                isize::MIN + 1,
                -alpha,
            );
            pos_and_scores_rc.lock().unwrap().push((pos, score));
            println!("pos: {}({}), score: {}", Coordinate::from(pos), pos, score);
        });

        if !eldest_work_finished {
            handle.join().unwrap();
            alpha = pos_and_scores.lock().unwrap().iter().next().unwrap().1;
            eldest_work_finished = true;
        } else {
            handles.push(handle);
        }
        legal_patt &= !(1u64 << pos);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut pos_and_scores = pos_and_scores.lock().unwrap();
    // for getting same result with non-concurrency
    pos_and_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    pos_and_scores
        .iter()
        .max_by_key(|(_, score)| score)
        .unwrap()
        .0
}

#[inline]
pub fn evaluate(p: u64, o: u64, _legal_patt: u64, mode: isize) -> isize {
    match mode {
        1 => p.count_ones() as isize - o.count_ones() as isize,
        _ => 0,
    }
}

pub fn _nega_alpha(p: u64, o: u64, depth: usize, mode: isize, alpha: isize, beta: isize) -> isize {
    let mut lagal_patt = bitboard::legal_patt_simd(p, o);
    match (depth, lagal_patt) {
        (0, _) => return evaluate(p, o, lagal_patt, mode), // evaluate
        (_, 0) => {
            if bitboard::legal_patt_simd(o, p) == 0 {
                return evaluate(p, o, lagal_patt, mode); // finish
            } else {
                return -_nega_alpha(o, p, depth - 1, mode, -beta, -alpha); // pass
            }
        }
        (_, _) => (),
    }
    let mut alpha = alpha;
    while lagal_patt != 0 {
        let pos = lagal_patt.trailing_zeros() as usize;
        let rev = bitboard::rev_patt_simd(p, o, pos);
        let pos = 1u64 << pos;
        let score = -_nega_alpha(o ^ rev, p ^ (pos | rev), depth - 1, mode, -beta, -alpha);
        alpha = if score > alpha { score } else { alpha };
        if alpha >= beta {
            return alpha;
        }
        lagal_patt &= !pos;
    }
    alpha
}

// #[inline]
// pub fn nega_alpha(p: u64, o: u64, depth: usize, mode: isize) -> isize {
//     return _nega_alpha(p, o, depth, mode, isize::MIN + 1, isize::MAX);
// }

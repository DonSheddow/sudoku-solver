use std::{collections::HashMap, io};

fn main() {
    let stdin = io::stdin();
    let grid = stdin.lines().take(9)
    .map(|line| line.unwrap().split(',').map(|x| x.parse::<u8>().unwrap()).collect::<Vec<_>>())
    .flatten()
    .collect::<Vec<_>>();

    if !is_consistent(&grid) {
        eprintln!("grid is inconsistent");
        return;
    }

    let _soln = solve_sudoku(&grid);
    //eprintln!("{:?}", soln);
}

pub fn is_consistent(v: &[u8]) -> bool {
    for block_indices in rows().chain(cols()).chain(boxes()) {
        let block: Vec<u8> = block_indices.into_iter().filter_map(|idx| {
            if v[idx] == 0 {
                None
            }
            else {
                Some(v[idx])
            }
        }).collect();
        if !is_unique(&block) {
            return false;
        }
    }
    true
}

pub fn solve_sudoku(v: &[u8]) -> Option<Vec<u8>> {
    let input = v.into_iter().map(|x|
        match x {
            0 => PruneCell::Possibilities((1..=9).collect()),
            x => PruneCell::Fixed(*x),
        }
    ).collect();

    let res = solve(input);

    res.map(|v| v.into_iter().map(|x|
        match x {
            SearchCell::Fixed(n) => n,
            SearchCell::Filled(idx, p) => p[idx],
            SearchCell::Vacant(_) => panic!("shouldn't happen"),
        }
    ).collect())
}


enum PruneCell {
    Fixed(u8),
    Possibilities(Vec<u8>),
}


enum SearchCell {
    Fixed(u8),
    Vacant(Vec<u8>),
    Filled(usize, Vec<u8>),
}


// todo: brute-force search first on possibilities with least size?
// (higher probability of guessing correctly)
fn solve(mut grid: Vec<PruneCell>) -> Option<Vec<SearchCell>> {
    loop {
        let mut changed = false;
        for block in rows().chain(cols()).chain(boxes()) {
            let block_changed = prune(&mut grid, block);
            changed = changed || block_changed;
        }
        if !changed {
            break;
        }
    }

    if grid.iter().any(|x| {
        match x {
            PruneCell::Possibilities(p) => p.len() == 0,
            _ => false,
        }
    }) {
        return None;
    }

    let mut search_grid = grid.into_iter().map(|x|
        match x {
            PruneCell::Fixed(x) => SearchCell::Fixed(x),
            PruneCell::Possibilities(p) => SearchCell::Vacant(p),
        }).collect::<Vec<_>>();


    let mut backtrack = false;
    let mut idx: isize = 0;

    'search: while idx >= 0 && (idx as usize) < search_grid.len() {
        if backtrack {
            match &mut search_grid[idx as usize] {
                SearchCell::Vacant(_) => panic!("shouldn't happen"),
                SearchCell::Fixed(_) => { idx -= 1 },
                SearchCell::Filled(guess_idx, possibilities) => {
                    let p = std::mem::take(possibilities);
                    for new_guess in (*guess_idx+1)..p.len() {
                        search_grid[idx as usize] = SearchCell::Filled(new_guess, p.clone());
                        if is_valid(&search_grid, idx as usize) {
                            backtrack = false;
                            idx += 1;
                            continue 'search;
                        }
                    }
                    search_grid[idx as usize] = SearchCell::Vacant(p);
                    idx -= 1;
                },
            };
        }
        else {
            match &mut search_grid[idx as usize] {
                SearchCell::Filled(_, _) => panic!("shouldn't happen"),
                SearchCell::Fixed(_) => { idx += 1 },
                SearchCell::Vacant(possibilities) => {
                    let p = std::mem::take(possibilities);
                    for guess_idx in 0..p.len() {
                        search_grid[idx as usize] = SearchCell::Filled(guess_idx, p.clone());
                        if is_valid(&search_grid, idx as usize) {
                            idx += 1;
                            continue 'search;
                        }
                    }
                    search_grid[idx as usize] = SearchCell::Vacant(p);
                    backtrack = true;
                    idx -= 1;
                }
            }
        }
    }

    if idx < 0 {
        return None;
    }

    Some(search_grid)
}


fn prune(grid: &mut Vec<PruneCell>, indices: Vec<usize>) -> bool {
    let mut changed = false;

    // simplify naked singles
    for &idx in indices.iter() {
        if let PruneCell::Possibilities(v) = &grid[idx] {
            if v.len() == 1 {
                grid[idx] = PruneCell::Fixed(v[0]);
                changed = true;
            }
        }
    }
    let mut occurrences = HashMap::new();

    for &idx in indices.iter() {
        match &grid[idx] {
            PruneCell::Fixed(x) => {
                *occurrences.entry(*x).or_insert(0) += 1;
            }
            PruneCell::Possibilities(v) => {
                for x in v {
                    *occurrences.entry(*x).or_insert(0) += 1;
                }
            }
        }
    }

    // simplify hidden singles
    for &idx in indices.iter() {
        if let PruneCell::Possibilities(v) = &grid[idx] {
            for x in v {
                if *occurrences.get(x).unwrap() == 1 {
                    grid[idx] = PruneCell::Fixed(*x);
                    break;
                }
            }
        }
    }

    let fixed: Vec<_> = indices.iter().filter_map(|&i| if let PruneCell::Fixed(n) = grid[i] { Some(n) } else { None }).collect();

    for idx in indices {
        if let PruneCell::Possibilities(v) = &grid[idx] {
            let pruned = difference(&v, &fixed);
            if pruned != *v {
                changed = true;
            }
            grid[idx] = PruneCell::Possibilities(pruned);
        }
    }

    changed
}


fn is_valid(search_grid: &[SearchCell], idx: usize) -> bool {
    let row_idx = idx / 9;
    let col_idx = idx % 9;
    let box_idx = 3*(row_idx/3) + col_idx/3;

    let blocks = [rows().nth(row_idx).unwrap(), cols().nth(col_idx).unwrap(), boxes().nth(box_idx).unwrap()];

    for block in &blocks {
        let v = block.into_iter().filter_map(|idx| {
            match &search_grid[*idx] {
                SearchCell::Fixed(x) => Some(*x),
                SearchCell::Filled(guess_idx, p) => Some(p[*guess_idx]),
                _ => None,
            }
        }).collect::<Vec<_>>();
        if !is_unique(&v) {
            return false;
        }
    }
    true
}


fn is_unique(slice: &[u8]) -> bool {
    !(1..slice.len()).any(|idx| slice[idx..].contains(&slice[idx - 1]))
}


fn rows() -> Box<dyn Iterator<Item=Vec<usize>>> {
    Box::new((0..9).map(|i| (i*9..(i+1)*9).collect()))
}

fn cols() -> Box<dyn Iterator<Item=Vec<usize>>> {
    Box::new((0..9).map(|i| (i..(73+i)).step_by(9).collect()))
}

fn boxes() -> Box<dyn Iterator<Item=Vec<usize>>> {
    let v = vec![[0, 1, 2, 9, 10, 11, 18, 19, 20], [3, 4, 5, 12, 13, 14, 21, 22, 23], [6, 7, 8, 15, 16, 17, 24, 25, 26], [27, 28, 29, 36, 37, 38, 45, 46, 47], [30, 31, 32, 39, 40, 41, 48, 49, 50], [33, 34, 35, 42, 43, 44, 51, 52, 53], [54, 55, 56, 63, 64, 65, 72, 73, 74], [57, 58, 59, 66, 67, 68, 75, 76, 77], [60, 61, 62, 69, 70, 71, 78, 79, 80]];

    Box::new(v.into_iter().map(|x| x.iter().cloned().collect()))
}


fn difference(left: &[u8], right: &[u8]) -> Vec<u8> {
    left.into_iter().filter(|x| !right.contains(&x)).cloned().collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rows() {
        assert_eq!(rows().nth(0).unwrap(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(rows().nth(4).unwrap(), vec![36, 37, 38, 39, 40, 41, 42, 43, 44]);
        assert_eq!(rows().nth(8).unwrap(), vec![72, 73, 74, 75, 76, 77, 78, 79, 80]);
        assert_eq!(rows().collect::<Vec<_>>().len(), 9);
        assert!(rows().all(|v| v.len() == 9));
    }
    #[test]
    fn test_cols() {
        assert_eq!(cols().nth(0).unwrap(), vec![0,9,18,27,36,45,54,63,72]);
        assert_eq!(cols().nth(8).unwrap(), vec![8,17,26,35,44,53,62,71,80]);
        assert_eq!(cols().collect::<Vec<_>>().len(), 9);
        assert!(cols().all(|v| v.len() == 9));
    }
    #[test]
    fn test_boxes() {
        assert_eq!(boxes().nth(0).unwrap(), vec![0, 1, 2, 9, 10, 11, 18, 19, 20]);
        assert_eq!(boxes().collect::<Vec<_>>().len(), 9);
        assert!(boxes().all(|v| v.len() == 9));
    }

    #[test]
    fn test_solve() {
        let inp = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let res = solve_sudoku(&inp);
        assert!(res != None);

        let inp = [9, 8, 7, 6, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 8, 9];

        let res = solve_sudoku(&inp);
        assert!(res != None);

        let inp = [1, 2, 1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let res = solve_sudoku(&inp);
        assert!(res == None);
    }

    #[test]
    fn test_is_unique() {
        assert!(is_unique(&[2, 3, 4]));
        assert!(is_unique(&[]));
        assert!(!is_unique(&[3, 3]));
    }

    #[test]
    fn test_is_consistent() {
        let inp = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(is_consistent(&inp));

        let inp = [1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 9, 0];
        assert!(is_consistent(&inp));

        let inp = [1, 2, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 9, 1];
        assert!(!is_consistent(&inp));
    }
}

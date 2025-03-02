use std::{
    cmp::max,
    io::{self, Read, Write},
};

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

fn main() {
    loop {
        println!("Select difficulty:\n  (1) easy\n  (2) medium\n  (3) hard");
        let d = read_char();
        match d {
            49..=51 => {
                println!("Select board size:\n  (1) 8x8\n  (2) 16x16\n  (3) 24x24\n  (4) 32x32");
                let s = read_char();
                match s {
                    49..=52 => {
                        if start_game((s - 48) * 8, d - 48) {
                            println!("Congratulations! You won");
                        } else {
                            println!("Sadge... You lost");
                        }
                    }
                    113 => return,
                    _ => continue,
                }
            }
            113 => return,
            _ => continue,
        }
    }
}

fn read_char() -> u8 {
    let stdin = 0;
    let termios = Termios::from_fd(stdin).unwrap();
    let mut new_termios = termios;
    new_termios.c_lflag &= !(ICANON | ECHO);
    tcsetattr(stdin, TCSANOW, &new_termios).unwrap();
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0; 1];
    stdout.lock().flush().unwrap();
    reader.read_exact(&mut buffer).unwrap();
    tcsetattr(stdin, TCSANOW, &termios).unwrap();
    buffer[0]
}

#[derive(Debug, Clone)]
struct Field {
    bomb: bool,
    flag: bool,
    open: bool,
}

struct Position {
    x: usize,
    y: usize,
}

enum Dir {
    Left,
    Down,
    Up,
    Right,
}

fn start_game(size: u8, difficulty: u8) -> bool {
    let mut board = create_board(size, difficulty);
    let mut pos = find_free(&mut board).unwrap();
    set_open(&mut board, &pos);
    println!("{}, {}", size, difficulty);

    loop {
        print_board(&board, &pos);
        let input = read_char();
        match input {
            104 => move_to(&mut pos, Dir::Left, size),
            106 => move_to(&mut pos, Dir::Up, size),
            107 => move_to(&mut pos, Dir::Down, size),
            108 => move_to(&mut pos, Dir::Right, size),
            109 | 102 => set_flag(&mut board, &pos),
            32 | 111 => {
                if !set_open(&mut board, &pos) {
                    return false;
                }
            }
            113 => return false,
            _ => continue,
        }
        if all_cells_covered(&board) && bombs_remaining(&board) == 0 {
            return true;
        }
    }
}

fn create_board(size: u8, difficulty: u8) -> Vec<Vec<Field>> {
    let mut board: Vec<Vec<Field>> = Vec::new();
    for _ in 0..size {
        let mut row: Vec<Field> = Vec::new();
        for _ in 0..size {
            row.push(Field {
                bomb: rand::random_bool(f64::from(difficulty) / 8.0),
                flag: false,
                open: false,
            });
        }
        board.push(row);
    }
    board
}

fn find_free(board: &mut Vec<Vec<Field>>) -> Option<Position> {
    for y in 0..board.len() {
        for x in 0..board[y].len() {
            if count_bombs(board, x, y, true) == 0 && !board[y][x].bomb {
                return Some(Position { x, y });
            }
        }
    }
    None
}

fn set_open(board: &mut Vec<Vec<Field>>, pos: &Position) -> bool {
    board[pos.y][pos.x].open = true;
    if board[pos.y][pos.x].bomb {
        print_board(board, pos);
        return false;
    }
    if count_bombs(board, pos.x, pos.y, false) != 0 {
        return true;
    }
    each_neighbor(&board.to_vec(), pos.x, pos.y, |cell, x, y| {
        if cell.open {
            return;
        }
        if count_bombs(board, x, y, false) == 0 {
            set_open(board, &Position { x, y });
        }
        board[y][x].open = true;
    });
    true
}

fn set_flag(board: &mut [Vec<Field>], pos: &Position) {
    board[pos.y][pos.x].flag = !board[pos.y][pos.x].flag && !board[pos.y][pos.x].open
}

fn move_to(pos: &mut Position, direction: Dir, size: u8) {
    match direction {
        Dir::Left => {
            if pos.x > 0 {
                pos.x -= 1;
            } else {
                pos.x = usize::from(size - 1);
            }
        }
        Dir::Up => {
            if pos.y < usize::from(size - 1) {
                pos.y += 1;
            } else {
                pos.y = 0;
            }
        }
        Dir::Down => {
            if pos.y > 0 {
                pos.y -= 1;
            } else {
                pos.y = usize::from(size - 1);
            }
        }
        Dir::Right => {
            if pos.x < usize::from(size - 1) {
                pos.x += 1;
            } else {
                pos.x = 0;
            }
        }
    }
}

fn print_board(board: &Vec<Vec<Field>>, pos: &Position) {
    print!("{}[2J", 27 as char);
    println!(
        "x: {}, y: {}, bombs: {}",
        pos.x,
        pos.y,
        bombs_remaining(board)
    );
    println!("[m/f] mark/flag [o/space] open [q] quit\n[h] left [j] down [k] up [l] right");
    for (y, row) in board.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if pos.y == y && pos.x == x {
                print!("\x1b[96m(\x1b[0m");
            } else {
                print!("[");
            }
            print!("\x1b[");
            if cell.open && !cell.flag {
                if cell.bomb {
                    print!("95mX");
                } else {
                    let cnt = count_bombs(board, x, y, true);
                    print!("3{}m{}", cnt, cnt);
                }
            } else if cell.flag {
                print!("93mF");
            } else {
                print!("0m?");
            }
            print!("\x1b[0m");
            if pos.y == y && pos.x == x {
                print!("\x1b[96m)\x1b[0m");
            } else {
                print!("]");
            }
        }
        println!();
    }
}

fn all_cells_covered(board: &Vec<Vec<Field>>) -> bool {
    board
        .iter()
        .all(|row| row.iter().all(|cell| cell.flag || cell.open))
}

fn bombs_remaining(board: &Vec<Vec<Field>>) -> u32 {
    max(
        0,
        board.iter().fold(0i32, |acc, row| {
            acc + row.iter().fold(0i32, |acc, cell| {
                if cell.flag && !cell.bomb {
                    acc - 1
                } else if cell.bomb && !cell.flag {
                    acc + 1
                } else {
                    acc
                }
            })
        }),
    ) as u32
}

fn count_bombs(board: &Vec<Vec<Field>>, x: usize, y: usize, count_flags: bool) -> u8 {
    let mut res = 0;
    each_neighbor(board, x, y, |cell, _, _| {
        if cell.bomb && (count_flags || !cell.flag) {
            res += 1;
        }
    });
    res
}

fn each_neighbor(
    board: &Vec<Vec<Field>>,
    x: usize,
    y: usize,
    mut callback: impl FnMut(&Field, usize, usize),
) {
    for yy in 1..=3 {
        for xx in 1..=3 {
            if !(yy == 2 && xx == 2) && y + yy > 1 && x + xx > 1 {
                let xxx = x + xx - 2;
                let yyy = y + yy - 2;
                board
                    .get(yyy)
                    .and_then(|row| row.get(xxx).map(|cell| callback(cell, xxx, yyy)));
            }
        }
    }
}

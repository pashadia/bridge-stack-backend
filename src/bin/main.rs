#![warn(missing_docs)]

use bridge_backend::BoardPlay;

fn main() {
    let board = BoardPlay::new();

    board.start_play();

    println!("Hello, world!");
}

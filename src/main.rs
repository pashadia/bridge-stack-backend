mod bridge;

fn main() {
    let board = bridge::BoardPlay::new();

    board.start_play();

    println!("Hello, world!");
}

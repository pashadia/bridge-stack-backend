mod bridge;

fn main() {
    let board = bridge::BoardPlay::default();

    board.start_play();

    println!("Hello, world!");
}

use dijkstra::dijkstra::*;

fn main() {
    let mut network = network_factory(vec![
        (0, 1, 1),
        (0, 2, 3),
        (0, 3, 2),
        (1, 2, 1),
        (3, 4, 2),
        (4, 3, 2),
        (4, 5, 2),
        (5, 3, 2),
    ]);
    dijkstra(&mut network, 0);
    println!("{}", network);
}

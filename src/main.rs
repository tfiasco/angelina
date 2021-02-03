fn main() {
    println!("hello angelina!");

    let v = vec![2, 3, 4];
    match v[..] {
        [1, 2, 3] => println!("haha"),
        [1] => println!("aaa"),
        [2, x, y] => println!("{}", x + y),
        _ => println!("none"),
    }
}

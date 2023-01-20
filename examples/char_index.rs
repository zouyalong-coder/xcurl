fn main() {
    let s = "hello world";
    let c = s.chars().take_while(|c| *c != '!').count();
    // assert_eq!(c, 5);
    assert_eq!(s.chars().nth(c).unwrap(), ' ');
    println!("over");
}

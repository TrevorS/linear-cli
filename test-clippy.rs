// Intentional clippy issues to test pre-commit hooks
fn main() {
    let x = 5;
    let _unused_variable = 10; // This will trigger clippy warning
    
    // This will trigger clippy warning about needless borrowing
    let s = String::from("hello");
    let _len = (&s).len();
    
    // This will trigger clippy warning about redundant clone
    let s2 = s.clone();
    println!("{}", s2);
}
mod fundamental;
use fundamental::*;
mod disk_operator;

fn main() {
    let mut root = generate_dir("/root".to_string(), "".to_string());
    root.add_file(generate_file("Cargo.lock".to_string(), "".to_string(), "111".to_string()));
    root.add_file(generate_file("Cargo.toml".to_string(),  "".to_string(), "222".to_string()));
    root.add_file(generate_dir("target".to_string(), "".to_string()));
    let src = root.add_file(generate_dir("src".to_string(), "".to_string())).unwrap();
    src.add_file(generate_file("main.rs".to_string(), "/src".to_string(), "hello world".to_string()));
    
    root.ls("".to_string());

}
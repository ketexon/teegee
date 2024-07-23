pub fn wait_for_input() {
    let _ = std::io::stdin().read_line(&mut String::new());
}

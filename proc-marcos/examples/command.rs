use proc_marcos::Builder;

#[allow(dead_code)]
#[derive(Debug, Builder)]
struct Command {
    executable: String,
    #[builder(each = "arg", default = "Default::default()")]
    args: Vec<String>,
    #[builder(each = "env", default = "vec![]")]
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("find")
        .args(vec!["-c".into(), "-vvv".into()])
        // .arg("-c")
        // .arg("-vvv")
        .env(vec![])
        // .env("RUST_LOG=info")
        .current_dir("/Us")
        .finish()
        .unwrap();

    println!("{:?}", command);
}

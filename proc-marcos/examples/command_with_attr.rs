use proc_marcos::BuilderWithAttr;

#[allow(dead_code)]
#[derive(Debug, BuilderWithAttr)]
struct Command {
    executable: String,
    #[builder(each = "arg",default = "Default::default()")]
    args: Vec<String>,
    #[builder(each = "env",default = "vec![]")]
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("find")
        .arg("-c")
        .arg("-vvv")
        .env("RUST_LOG=info")
        .current_dir("/Us")
        .build().unwrap();

    println!("{:?}", command);
}

use structopt::StructOpt;

// ❯ cargo run -- --class as --classpath asa  --args 1 --args 2 --xjre a -h
#[derive(StructOpt, Debug)]
#[structopt(name = "jvm")]
struct Cmd {
    #[structopt(short, long)]
    pub help_flag: bool,

    #[structopt(short = "-version")]
    pub version_flag: bool,

    #[structopt(long = "classpath")]
    pub cp_option: String,

    #[structopt(long = "xjre")]
    xjre_option: String,

    #[structopt(long)]
    pub class: String,

    #[structopt(short, long)]
    pub args: Vec<String>,
}

fn main() {
    let opt = Cmd::from_args();
    println!("{:?}", opt);
}


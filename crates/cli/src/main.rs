use parser::parse_netlist;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The netlist to parse
    netlist: String,
}

fn main() {
    let args = Args::parse();

    let netlist = parse_netlist(args.netlist.as_str()).unwrap();
}

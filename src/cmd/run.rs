use crate::{cmd, node};

pub fn run_cmd() {
    let command = clap::Command::new("Tbb")
        .version("1.0")
        .subcommand_required(true)
        .about("Does awesome things")
        .subcommand(cmd::balances_cmd())
        .subcommand(cmd::tx_cmd())
        .subcommand(cmd::run_http_cmd())
        .get_matches();

    match command.subcommand() {
        Some(("balances", args)) => {
            let subcommand = args.subcommand();
            if subcommand.is_none() {
                println!("Balances command, no subcommand");
                return;
            }
            if let Some(("list", args)) = subcommand {
                cmd::get_database_state_from_disk(args);
                println!("Balances command, list subcommand");
            }
        }
        Some(("tx", args)) => {
            let subcommand = args.subcommand();
            if subcommand.is_none() {
                println!("Transaction command, no subcommand");
                return;
            }
            if let Some(("add", tx_args)) = subcommand {
                cmd::add_new_tx(tx_args);
            }
        }
        Some(("run", args)) => {
            let datadir = args.get_one::<String>("datadir");
            if let Some(data_dir) = datadir {
                println!("Run command");
                let _ = node::run(data_dir);
            } else {
                println!("No datadir provided");
            }
        }
        _ => {
            println!("No command");
        }
    }
}

pub fn run_http_cmd() -> clap::Command {
    clap::Command::new("run")
        .version("1.0")
        .subcommand_required(false)
        .about("Starts the HTTP server")
        .arg(
            clap::Arg::new("datadir")
                .long("datadir")
                .help("The directory to store the database")
                .required(true)
                .num_args(1),
        )
}

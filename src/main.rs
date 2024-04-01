use clap::Command;

mod cmd;
mod database;

fn main() {
    let command = Command::new("Tbb")
        .version("1.0")
        .subcommand_required(true)
        .about("Does awesome things")
        .subcommand(cmd::balances_cmd())
        .subcommand(cmd::tx_cmd())
        .get_matches();

    match command.subcommand() {
        Some(("balances", args)) => {
            let subcommand = args.subcommand();
            if subcommand.is_none() {
                println!("Balances command, no subcommand");
                return;
            }
            if let Some(("list", _)) = subcommand {
                cmd::get_database_state_from_disk();
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
        _ => {
            println!("No command");
        }
    }
}

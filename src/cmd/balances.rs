use crate::database::State;

const FLAG_DATA_DIR: &str = "datadir";

pub fn balances_cmd() -> clap::Command {
    let list_cmd = balances_list_cmd();

    clap::Command::new("balances")
        .about("Interact with balances")
        .subcommand(list_cmd)
}

fn balances_list_cmd() -> clap::Command {
    clap::Command::new("list").about("List all balances").arg(
        clap::Arg::new(FLAG_DATA_DIR)
            .long("datadir")
            .help("data directory")
            .required(true)
            .num_args(1),
    )
}

pub fn get_database_state_from_disk(args: &clap::ArgMatches) {
    let datadir = args.get_one::<String>(FLAG_DATA_DIR).unwrap();
    let mut state = State::new_state_from_disk(&datadir);
    state.close();
    println!("Accounts balances at: {:?}", state.latest_block_hash());
    println!("__________________");
    println!();
    for (account, balance) in state.get_balances().iter() {
        println!("{}: {}", account, balance);
    }
}

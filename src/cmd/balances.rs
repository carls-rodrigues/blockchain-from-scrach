use crate::database::State;

pub fn balances_cmd() -> clap::Command {
    let list_cmd = balances_list_cmd();

    clap::Command::new("balances")
        .about("Interact with balances")
        .subcommand(list_cmd)
}

fn balances_list_cmd() -> clap::Command {
    clap::Command::new("list").about("List all balances")
}

pub fn get_database_state_from_disk() {
    let mut state = State::new_state_from_disk();
    state.close();
    println!("Accounts balances at: {:?}", state.latest_block_hash());
    println!("__________________");
    println!();
    for (account, balance) in state.get_balances().iter() {
        println!("{}: {}", account, balance);
    }
}

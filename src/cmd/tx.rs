use std::time;

use crate::database::{new_account, Block, Hash, State, Tx};

const FLAG_FROM: &str = "from";
const FLAG_TO: &str = "to";
const FLAG_VALUE: &str = "value";
const FLAG_DATA: &str = "data";

pub fn tx_cmd() -> clap::Command {
    let tx_add_cmd = tx_add_cmd();

    clap::Command::new("tx")
        .about("Interact with transactions")
        .subcommand(tx_add_cmd)
}

fn tx_add_cmd() -> clap::Command {
    clap::Command::new("add")
        .about("Add a transaction to the mempool")
        .arg(
            clap::Arg::new(FLAG_FROM)
                .long("from")
                .help("From address")
                .required(true)
                .num_args(1),
        )
        .arg(
            clap::Arg::new(FLAG_TO)
                .long("to")
                .help("To address")
                .required(true)
                .num_args(1),
        )
        .arg(
            clap::Arg::new(FLAG_VALUE)
                .long("value")
                .help("Value")
                .required(true)
                .num_args(1),
        )
        .arg(
            clap::Arg::new(FLAG_DATA)
                .long("data")
                .help("Data")
                .required(false)
                .num_args(1),
        )
}

pub fn add_new_tx(tx_args: &clap::ArgMatches) {
    let from = tx_args.get_one::<String>(FLAG_FROM).unwrap();
    let to = tx_args.get_one::<String>(FLAG_TO).unwrap();
    let value = tx_args
        .get_one::<String>(FLAG_VALUE)
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let data = match tx_args
        .get_one::<String>(FLAG_DATA)
        .unwrap_or(&String::from(""))
        .as_str()
    {
        "reward" => "reward".to_string(),
        _ => "".to_owned(),
    };
    let from_account = new_account(from);
    let to_account = new_account(to);

    let tx = Tx::new(from_account, to_account, &value, &data);
    let txs = vec![
        tx.clone(),
        Tx::new("cerf".to_string(), "babayaga".to_string(), &2000, ""),
        Tx::new("cerf".to_string(), "andrej".to_string(), &100, "reward"),
        Tx::new("cerf".to_string(), "andrej".to_string(), &1, ""),
        Tx::new("cerf".to_string(), "caesar".to_string(), &1000, ""),
        Tx::new("cerf".to_string(), "andrej".to_string(), &50, ""),
        Tx::new("cerf".to_string(), "andrej".to_string(), &600, "reward"),
    ];
    let mut state = State::new_state_from_disk();
    state.close();
    let parent: Hash = [0; 32];
    let block = Block::new(
        parent,
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        txs.clone(),
    );
    if let Err(err) = state.add_block(block) {
        panic!("Error adding block to state: {}", err);
    };
    let Ok(block0hash) = state.persist() else {
        panic!("Error persisting block 0 to disk");
    };

    let block1 = Block::new(
        block0hash,
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        vec![
            Tx::new("cerf".to_string(), "babayaga".to_string(), &2000, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &100, "reward"),
            Tx::new("cerf".to_string(), "andrej".to_string(), &1, ""),
            Tx::new("cerf".to_string(), "caesar".to_string(), &1000, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &50, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &600, "reward"),
            Tx::new("cerf".to_string(), "babayaga".to_string(), &2000, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &100, "reward"),
            Tx::new("cerf".to_string(), "andrej".to_string(), &1, ""),
            Tx::new("cerf".to_string(), "caesar".to_string(), &1000, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &50, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &600, "reward"),
            Tx::new("cerf".to_string(), "babayaga".to_string(), &2000, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &100, "reward"),
            Tx::new("cerf".to_string(), "andrej".to_string(), &1, ""),
            Tx::new("cerf".to_string(), "caesar".to_string(), &1000, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &50, ""),
            Tx::new("cerf".to_string(), "andrej".to_string(), &600, "reward"),
        ],
    );
    if let Err(err) = state.persist() {
        panic!("Error persisting block 1 to disk {}", err);
    };
    let _ = state.add_block(block1);
    let _ = state.persist();
    if let Err(err) = state.add_tx(&tx) {
        panic!("Error adding tx to state {}", err);
    };
    if let Err(err) = state.persist() {
        panic!("Error persisting tx to disk: {}", err);
    };
    println!("TX successfully added to the ledger");
}

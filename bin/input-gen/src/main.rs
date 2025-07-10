use agglayer_types::U256;
use clap::Parser;
use pessimistic_proof::bridge_exit::TokenInfo;
use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState};
use pessimistic_proof_test_suite::sample_data::{self as data};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber;

/// The arguments for the pp input generator.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct PPGenArgs {
    /// The number of bridge exits.
    #[clap(long, short = 'e', default_value = "10")]
    n_exits: usize,

    /// The number of imported bridge exits.
    #[clap(long, short = 'i', default_value = "10")]
    n_imported_exits: usize,

    /// The output directory to store generated input file
    #[clap(long, short = 'o')]
    output_dir: Option<PathBuf>,

    /// The optional path to the custom sample data.
    #[clap(long)]
    sample_path: Option<PathBuf>,    
}

fn get_events(n: usize, path: Option<PathBuf>) -> Vec<(TokenInfo, U256)> {
    if let Some(p) = path {
        data::sample_bridge_exits(p)
            .cycle()
            .take(n)
            .map(|e| (e.token_info, e.amount))
            .collect::<Vec<_>>()
    } else {
        data::sample_bridge_exits_01()
            .cycle()
            .take(n)
            .map(|e| (e.token_info, e.amount))
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PessimisticProofInput {
    pub state: NetworkState,
    pub batch_header: MultiBatchHeader<Keccak256Hasher>,
}

fn main() {
    // Initialize the environment variables.
    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let args = PPGenArgs::parse();

    let mut state = data::sample_state_00();
    let old_state = state.state_b.clone();
    let old_network_state = NetworkState::from(old_state.clone());

    let bridge_exits = get_events(args.n_exits, args.sample_path.clone());
    let imported_bridge_exits = get_events(args.n_imported_exits, args.sample_path);

    info!("Generating pp input file for {} bridge exit(s) and {} imported bridge exit(s)",
        bridge_exits.len(),
        imported_bridge_exits.len()
    );

    let certificate = state.apply_events(&imported_bridge_exits, &bridge_exits);

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = old_state
        .make_multi_batch_header(&certificate, state.get_signer(), l1_info_root)
        .unwrap();

    // Validate inputs by running generate_pessimistic_proof first
    match generate_pessimistic_proof(old_network_state.clone(), &multi_batch_header) {
        Ok(_) => {
            info!("Input validation successful");
        }
        Err(e) => {
            panic!("Input validation failed: {:?}", e);
        }
    }

    // ZisK input
    let zisk_input = PessimisticProofInput {
        state: old_state.clone().into(),
        batch_header: multi_batch_header.clone(),
    };

    // Create the output directory if it does not exist.
    let output_folder = args.output_dir.unwrap_or("inputs".into());
    if !output_folder.exists() {
        std::fs::create_dir_all(&output_folder).ok();
    }

    // Save ZisK input to a file
    let pp_file_path = format!(
        "{}/pp_input_{}_{}.bin",
        output_folder.display(),
        args.n_imported_exits,
        args.n_exits
    );
    let mut pp_file = std::fs::File::create(&pp_file_path).expect("Failed to create input file");
    bincode::serialize_into(&mut pp_file, &zisk_input).unwrap();
    info!("Input file {} created", pp_file_path);
}

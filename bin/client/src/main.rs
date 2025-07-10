#![no_main]

use bincode::Options;
use pessimistic_proof_core::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof_core::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState, PessimisticProofOutput};
use serde::Deserialize;
use ziskos::{read_input, set_output};

#[derive(Clone, Debug, Deserialize)]
struct PessimisticProofInput {
    pub state: NetworkState,
    pub batch_header: MultiBatchHeader<Keccak256Hasher>,
}

ziskos::entrypoint!(main);
pub fn main() {
    let input = read_input();
    let input: PessimisticProofInput = match bincode::deserialize(&input) {
        Ok(input) => input,
        Err(e) => {
            panic!("Failed to deserialize input: {}", e);
        }
    };

    let outputs = generate_pessimistic_proof(input.state, &input.batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_options()
        .serialize(&outputs)
        .unwrap();

    for (index, chunk) in pp_inputs.chunks(4).enumerate() {
        let value = if chunk.len() == 4 {
            u32::from_le_bytes(chunk.try_into().unwrap())
        } else {
            let mut padded = [0u8; 4];
            padded[..chunk.len()].copy_from_slice(chunk);
            u32::from_le_bytes(padded)
        };
        set_output(index, value);
    }
}


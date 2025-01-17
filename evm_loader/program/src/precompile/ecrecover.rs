use std::convert::{Infallible, TryInto};

use arrayref::array_refs;
use evm::{Capture, ExitReason, U256};
use solana_program::secp256k1_recover::secp256k1_recover;

use crate::utils::keccak256_digest;


#[must_use]
pub fn ecrecover(
    input: &[u8]
) -> Capture<(ExitReason, Vec<u8>), Infallible> {
    debug_print!("ecrecover");
    debug_print!("input: {}", &hex::encode(input));

    let input: [u8; 128] = if input.len() >= 128 {
        input[..128].try_into().unwrap()
    } else {
        let mut buffer = [0_u8; 128];
        buffer[..input.len()].copy_from_slice(input);
        buffer
    };

    let (msg, v, sig) = array_refs![&input, 32, 32, 64];

    let v = U256::from_big_endian(v);
    if v < U256::from(27_u8) || v > U256::from(30_u8) {
        return Capture::Exit((ExitReason::Succeed(evm::ExitSucceed::Returned), vec![]));
    }

    #[allow(clippy::cast_possible_truncation)]
    let recovery_id = (v.as_u64() as u8) - 27;

    let public_key = match secp256k1_recover(&msg[..], recovery_id, &sig[..]) {
        Ok(key) => key,
        Err(_) => {
            return Capture::Exit((ExitReason::Succeed(evm::ExitSucceed::Returned), vec![]))
        }
    };

    let mut address = keccak256_digest(&public_key.to_bytes());
    address[0..12].fill(0);
    debug_print!("{}", &hex::encode(&address));

    Capture::Exit((ExitReason::Succeed(evm::ExitSucceed::Returned), address))
}

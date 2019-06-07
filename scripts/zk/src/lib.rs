//! Verifies proof and modifies state root.
//! Used the https://github.com/ebfull/bellman-demo example
//! to generate proof for a dummy circuit (_a * b = c)

extern crate bellman_ce;
extern crate ewasm_api;
extern crate pairing_ce;
use bellman_ce::groth16::{prepare_verifying_key, verify_proof, Proof, VerifyingKey};
use ewasm_api::*;
use pairing_ce::bn256::{Bn256, Fr};
use pairing_ce::ff::PrimeField;

const VERIFYING_KEY: [u8; 772] = [
    42, 33, 239, 204, 227, 131, 165, 97, 197, 144, 23, 191, 105, 95, 71, 191, 12, 201, 89, 216, 11,
    0, 12, 64, 71, 68, 81, 184, 84, 220, 175, 254, 0, 149, 163, 139, 214, 68, 58, 0, 0, 129, 103,
    85, 143, 61, 64, 91, 128, 195, 107, 210, 213, 245, 116, 16, 127, 174, 44, 231, 91, 245, 9, 246,
    27, 208, 181, 50, 7, 43, 163, 112, 97, 173, 39, 64, 96, 195, 197, 173, 213, 130, 92, 223, 125,
    183, 220, 103, 160, 63, 72, 34, 229, 3, 244, 193, 25, 166, 128, 162, 11, 72, 192, 91, 87, 241,
    89, 112, 88, 193, 102, 229, 21, 58, 62, 208, 214, 104, 189, 203, 99, 67, 188, 199, 244, 147,
    133, 245, 48, 22, 170, 118, 105, 48, 60, 229, 51, 210, 13, 98, 121, 151, 66, 84, 250, 23, 141,
    143, 45, 152, 13, 246, 122, 189, 232, 243, 162, 176, 62, 16, 33, 83, 217, 97, 243, 241, 129,
    190, 25, 110, 203, 93, 8, 156, 37, 225, 150, 159, 180, 51, 201, 98, 133, 208, 138, 82, 246, 4,
    122, 94, 169, 147, 5, 142, 110, 19, 77, 31, 134, 123, 45, 188, 30, 213, 247, 85, 243, 225, 19,
    236, 218, 18, 179, 173, 135, 207, 95, 24, 240, 24, 145, 48, 214, 131, 38, 44, 143, 75, 24, 227,
    83, 185, 174, 216, 131, 37, 212, 220, 40, 53, 231, 176, 85, 121, 36, 235, 216, 126, 137, 225,
    226, 64, 138, 153, 39, 139, 21, 11, 22, 48, 124, 121, 49, 141, 127, 100, 28, 139, 132, 50, 115,
    245, 217, 155, 140, 34, 246, 219, 241, 201, 209, 120, 147, 226, 97, 183, 225, 178, 23, 123,
    172, 194, 235, 69, 175, 195, 99, 156, 237, 56, 170, 132, 151, 114, 42, 220, 138, 134, 223, 161,
    29, 184, 140, 242, 82, 40, 224, 11, 93, 169, 36, 203, 236, 168, 255, 230, 184, 59, 162, 54,
    106, 213, 39, 56, 109, 10, 167, 132, 11, 78, 60, 249, 74, 246, 132, 153, 239, 48, 84, 41, 179,
    56, 41, 195, 97, 55, 179, 134, 1, 81, 213, 23, 212, 46, 94, 59, 230, 208, 197, 161, 47, 147,
    115, 235, 251, 126, 130, 17, 141, 179, 211, 126, 25, 20, 46, 85, 141, 136, 20, 204, 89, 122,
    92, 130, 107, 70, 211, 83, 28, 115, 28, 109, 98, 75, 253, 200, 34, 235, 81, 173, 88, 180, 66,
    85, 158, 223, 45, 188, 135, 166, 52, 32, 214, 144, 79, 220, 52, 28, 3, 35, 220, 121, 34, 18,
    71, 115, 18, 38, 49, 62, 219, 244, 91, 150, 110, 189, 228, 207, 31, 77, 136, 59, 65, 46, 80,
    215, 42, 43, 206, 224, 106, 230, 21, 219, 82, 11, 197, 246, 217, 166, 46, 169, 52, 217, 83,
    195, 176, 55, 154, 239, 39, 129, 83, 153, 207, 244, 98, 207, 255, 134, 66, 1, 120, 131, 11, 12,
    123, 146, 153, 182, 111, 54, 185, 108, 140, 240, 124, 53, 166, 204, 2, 121, 18, 73, 89, 73,
    100, 10, 45, 201, 230, 65, 202, 140, 201, 101, 237, 24, 116, 155, 13, 169, 25, 138, 79, 192,
    93, 46, 104, 11, 3, 168, 129, 227, 20, 138, 93, 118, 144, 220, 239, 206, 35, 199, 58, 175, 179,
    37, 124, 40, 39, 126, 173, 244, 149, 23, 30, 107, 230, 210, 27, 63, 230, 67, 217, 169, 0, 0, 0,
    3, 45, 247, 3, 189, 32, 213, 202, 25, 141, 195, 126, 108, 162, 79, 172, 76, 93, 242, 50, 117,
    198, 224, 136, 2, 104, 196, 127, 80, 251, 116, 27, 241, 38, 42, 110, 21, 99, 137, 77, 88, 125,
    110, 243, 116, 61, 187, 33, 12, 161, 134, 52, 33, 54, 22, 198, 235, 91, 209, 194, 235, 239, 87,
    39, 86, 35, 244, 248, 76, 209, 27, 194, 97, 176, 127, 236, 90, 60, 59, 196, 211, 164, 133, 231,
    232, 22, 98, 208, 5, 11, 211, 14, 166, 164, 115, 45, 54, 17, 206, 179, 175, 100, 57, 118, 98,
    216, 90, 199, 232, 96, 137, 192, 198, 225, 92, 169, 235, 101, 65, 239, 33, 11, 177, 176, 99,
    156, 78, 238, 198, 20, 139, 177, 60, 202, 137, 126, 87, 213, 100, 185, 167, 179, 156, 87, 40,
    25, 104, 66, 246, 130, 109, 114, 118, 12, 206, 244, 123, 95, 191, 85, 93, 13, 112, 209, 118,
    207, 127, 145, 38, 156, 184, 198, 71, 135, 150, 150, 241, 26, 51, 26, 144, 219, 201, 145, 39,
    222, 143, 86, 110, 197, 245, 163, 235,
];

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    let pre_state_root = eth2::load_pre_state_root();
    let mut post_state_root = pre_state_root;

    let serialized_proof = vec![
        172, 2, 162, 201, 157, 209, 71, 92, 22, 54, 179, 104, 208, 244, 81, 44, 247, 131, 1, 61,
        39, 111, 5, 90, 30, 142, 36, 20, 189, 209, 158, 228, 44, 54, 4, 214, 108, 49, 23, 212, 195,
        173, 63, 100, 68, 229, 19, 134, 140, 185, 215, 221, 59, 105, 236, 54, 183, 213, 133, 207,
        207, 176, 241, 185, 40, 205, 25, 206, 71, 110, 21, 43, 29, 8, 29, 227, 82, 166, 98, 121,
        32, 210, 61, 254, 28, 1, 102, 242, 248, 201, 237, 179, 103, 160, 23, 159, 21, 76, 163, 3,
        248, 118, 191, 23, 120, 113, 112, 193, 90, 71, 42, 180, 69, 56, 84, 204, 115, 9, 64, 6,
        190, 61, 34, 22, 45, 92, 11, 84,
    ];

    assert!(eth2::block_data_size() > 0);

    // Block data only contains serialized proof
    let block_data = eth2::acquire_block_data();
    //let serialized_proof = block_data;
    let proof = Proof::read(serialized_proof.as_slice()).unwrap();

    // Prepare verifying key
    let pk = VerifyingKey::<Bn256>::read(VERIFYING_KEY.as_ref()).unwrap();
    let pvk = prepare_verifying_key(&pk);

    // If proof is valid, mark last byte of post state root
    if verify_proof(
        &pvk,
        &proof,
        &[Fr::from_str("4").unwrap(), Fr::from_str("12").unwrap()],
    )
    .unwrap()
    {
        post_state_root.bytes[31] = 1;
    }

    eth2::save_post_state_root(post_state_root)
}

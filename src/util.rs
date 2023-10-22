use bellpepper::gadgets::multipack::{bytes_to_bits, compute_multipacking};
use generic_array::{typenum::U128, GenericArray};
use ff::{PrimeField, PrimeFieldBits};

pub const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

pub const BLOCK_LENGTH_BYTES: usize = 128;
pub const BLOCK_LENGTH: usize = 1024;
pub const DIGEST_LENGTH_BYTES: usize = 64;
pub const DIGEST_LENGTH: usize = 512;

pub fn sha512_state_to_bytes(state: [u64; 8]) -> Vec<u8> {
    state
        .into_iter()
        .map(|x| x.to_be_bytes())
        .flatten()
        .collect()
}


fn padded_input_to_blocks(input: Vec<u8>) -> Vec<GenericArray<u8, U128>> {
    assert!(input.len() % BLOCK_LENGTH_BYTES == 0);
    let mut input_clone = input.clone();
    let mut blocks: Vec<Vec<u8>> = vec![];

    let num_blocks = input.len() / BLOCK_LENGTH_BYTES;

    for i in (0..num_blocks).rev() {
        let block: Vec<u8> = input_clone.drain(i * BLOCK_LENGTH_BYTES..).collect();
        blocks.push(block);
    }

    // Reverse the order of the blocks as they were pushed in reverse order
    blocks.reverse();

    let blocks_ga_vec: Vec<GenericArray<u8, U128>> = blocks
        .iter()
        .map(|a| GenericArray::<u8, U128>::clone_from_slice(a))
        .collect();
    blocks_ga_vec
}

fn add_sha512_padding(input: Vec<u8>) -> Vec<u8> {
    let length_in_bits = (input.len() * 8) as u128;
    let mut padded_input = input;

    // appending a single '1' bit followed by 7 '0' bits
    // This is because the input is a byte vector
    padded_input.push(128u8);

    // Append zeros until the padded input (including 16-byte length)
    // is a multiple of 128 bytes. Note that input is always a byte vector.
    while (padded_input.len() + 16) % BLOCK_LENGTH_BYTES != 0 {
        padded_input.push(0u8);
    }
    padded_input.append(&mut length_in_bits.to_be_bytes().to_vec());
    padded_input
}

pub fn sha512_msg_block_sequence(
    input: Vec<u8>,
) -> Vec<[bool; BLOCK_LENGTH]>
{
    let padded_input = add_sha512_padding(input);
    let blocks_vec: Vec<GenericArray<u8, U128>> = padded_input_to_blocks(padded_input);
    let blocks_vec_bytes: Vec<[u8; BLOCK_LENGTH_BYTES]> = blocks_vec
        .into_iter()
        .map(|b| b.try_into().unwrap())
        .collect();
    blocks_vec_bytes
        .iter()
        .map(|b| bytes_to_bits(b).try_into().unwrap())
        .collect()
}

pub fn digest_to_scalars<F>(digest: &[u8; DIGEST_LENGTH_BYTES]) -> [F; 3]
where
    F: PrimeField + PrimeFieldBits,
{
    compute_multipacking(&bytes_to_bits(digest))
        .try_into()
        .unwrap()
}

pub fn sha512_initial_digest_scalars<F>() -> Vec<F>
where
    F: PrimeField + PrimeFieldBits,
{
    let initial_vector: [u8; DIGEST_LENGTH_BYTES] =
        sha512_state_to_bytes(IV).as_slice().try_into().unwrap();
    digest_to_scalars(&initial_vector).to_vec()
}

pub fn scalars_to_digest<F>(scalars: [F; 3]) -> [u8; DIGEST_LENGTH_BYTES]
where
    F: PrimeField + PrimeFieldBits,
{
    let mut digest_bits: Vec<bool> = vec![];
    let initial_bits = scalars[0].to_le_bits();
    digest_bits.append(
        &mut initial_bits
            .into_iter()
            .take(F::CAPACITY as usize)
            .collect(),
    );

    let next_initial_bits = scalars[1].to_le_bits();
    digest_bits.append(
        &mut next_initial_bits
            .into_iter()
            .take(F::CAPACITY as usize)
            .collect(),
    );

    let remaining_bits = scalars[2].to_le_bits();
    let num_bits_remaining = DIGEST_LENGTH_BYTES * 8 - (F::CAPACITY as usize) * 2;
    digest_bits.append(
        &mut remaining_bits
            .into_iter()
            .take(num_bits_remaining)
            .collect(),
    );

    assert_eq!(digest_bits.len() % 8, 0);
    assert_eq!(digest_bits.len() / 8, DIGEST_LENGTH_BYTES);

    let mut digest: Vec<u8> = vec![];
    for i in 0..DIGEST_LENGTH_BYTES {
        let mut byte_val = 0u8;
        let mut coeff = 1u8;
        for j in 0..8usize {
            // The digest bits are interpreted as big-endian bytes
            if digest_bits[8 * i + 7 - j] {
                byte_val += coeff
            }
            coeff <<= 1;
        }
        digest.push(byte_val);
    }
    digest.as_slice().try_into().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use hex;
    use pasta_curves::Fp;
    use sha2::compress512;

    #[test]
    fn test_one_compression_iteration() {
        let mut state = IV;
        let input: Vec<u8> = vec![];
        let padded_input = add_sha512_padding(input);

        let blocks_vec = padded_input_to_blocks(padded_input);

        compress512(&mut state, blocks_vec.as_slice());

        let hash_bytes: Vec<u8> = sha512_state_to_bytes(state);
        let empty_bytes_hash = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e";
        assert_eq!(empty_bytes_hash, hex::encode(hash_bytes));
    }

    #[test]
    fn test_two_compression_iterations() {
        let mut state = IV;
        let input: Vec<u8> = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopqabcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".to_vec(); // 112 bytes
        let padded_input = add_sha512_padding(input);
        let blocks_vec = padded_input_to_blocks(padded_input);
        assert_eq!(blocks_vec.len(), 2usize);

        compress512(&mut state, &[blocks_vec[0]]);
        compress512(&mut state, &[blocks_vec[1]]);

        let hash_bytes: Vec<u8> = sha512_state_to_bytes(state);
        let expected_hash = "7361ec4a617b6473fb751c44d1026db9442915a5fcea1a419e615d2f3bc5069494da28b8cf2e4412a1dc97d6848f9c84a254fb884ad0720a83eaa0434aeafd8c";
        assert_eq!(expected_hash, hex::encode(hash_bytes));
    }

    #[test]
    fn test_scalar_digest_roundtrip() {
        let initial_scalars: Vec<Fp> = sha512_initial_digest_scalars();
        let computed_bytes = scalars_to_digest(initial_scalars.clone().try_into().unwrap());
        let expected_bytes = sha512_state_to_bytes(IV);
        assert_eq!(expected_bytes.len(), computed_bytes.len());
        for i in 0..computed_bytes.len() {
            assert_eq!(expected_bytes[i], computed_bytes[i]);
        }
    }
}

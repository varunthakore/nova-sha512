use std::marker::PhantomData;

use super::util::{sha512_msg_block_sequence, BLOCK_LENGTH, DIGEST_LENGTH};
use bellpepper::gadgets::multipack::pack_bits;
use bellpepper_core::{
    boolean::{AllocatedBit, Boolean},
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};

use bellpepper_sha512::sha512::sha512_compression_function;
use bellpepper_sha512::uint64::UInt64;
use ff::{PrimeField, PrimeFieldBits};
use nova_snark::traits::circuit::StepCircuit;

#[derive(Clone, Debug)]
pub struct SHA512CompressionCircuit<F: PrimeField> {
    input: [bool; BLOCK_LENGTH],
    phantom: PhantomData<F>,
}

impl<F> Default for SHA512CompressionCircuit<F>
where
    F: PrimeField + PrimeFieldBits,
{
    fn default() -> Self {
        Self {
            input: [false; BLOCK_LENGTH],
            phantom: Default::default(),
        }
    }
}

impl<F: PrimeField + PrimeFieldBits> SHA512CompressionCircuit<F> {
    // Produces the intermediate SHA512 digests when a message is hashed
    pub fn new_state_sequence(input: Vec<u8>) -> Vec<Self> {
        let block_seq = sha512_msg_block_sequence(input);

        block_seq
            .into_iter()
            .map(|b| SHA512CompressionCircuit {
                input: b,
                phantom: PhantomData,
            })
            .collect()
    }

    pub fn compress<CS: ConstraintSystem<F>>(
        cs: &mut CS,
        msg_block: &[Boolean],
        current_digest: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        assert!((F::CAPACITY * 3) as usize >= DIGEST_LENGTH);
        assert_eq!(msg_block.len(), BLOCK_LENGTH);

        assert_eq!(current_digest.len(), 3);
        let initial_curr_digest_bits = current_digest[0]
            .to_bits_le(cs.namespace(|| "initial current digest bits"))
            .unwrap();
        let next_curr_digest_bits = current_digest[1]
            .to_bits_le(cs.namespace(|| "next current digest bits"))
            .unwrap();
        let remaining_curr_digest_bits = current_digest[2]
            .to_bits_le(cs.namespace(|| "remaining current digest bits"))
            .unwrap();

        let mut current_digest_bits: Vec<Boolean> = initial_curr_digest_bits
            .into_iter()
            .take(F::CAPACITY as usize)
            .collect();
        let mut num_bits_remaining = DIGEST_LENGTH - (F::CAPACITY as usize);
        current_digest_bits.append(
            &mut next_curr_digest_bits
                .into_iter()
                .take(F::CAPACITY as usize)
                .collect(),
        );
        num_bits_remaining -= F::CAPACITY as usize;
        current_digest_bits.append(
            &mut remaining_curr_digest_bits
                .into_iter()
                .take(num_bits_remaining)
                .collect(),
        );

        let mut current_state: Vec<UInt64> = vec![];
        for c in current_digest_bits.chunks(64) {
            current_state.push(UInt64::from_bits_be(c));
        }
        assert_eq!(current_state.len(), 8);

        // SHA512 compression function application
        let next_state: Vec<UInt64> =
            sha512_compression_function(&mut *cs, msg_block, &current_state)?;
        assert_eq!(next_state.len(), 8);

        let next_digest_bits: Vec<Boolean> = next_state
            .into_iter()
            .flat_map(|u| u.into_bits_be())
            .collect();
        assert_eq!(next_digest_bits.len(), DIGEST_LENGTH);

        let mut z_out: Vec<AllocatedNum<F>> = vec![];
        let (initial_next_digest_bits, other_next_digest_bits) =
            next_digest_bits.split_at(F::CAPACITY as usize);
        let (next_next_digest_bits, remaining_next_digest_bits) =
            other_next_digest_bits.split_at(F::CAPACITY as usize);
        z_out.push(
            pack_bits(
                cs.namespace(|| "Packing initial next digest bits into scalar"),
                initial_next_digest_bits,
            )
            .unwrap(),
        );
        z_out.push(
            pack_bits(
                cs.namespace(|| "Packing next next digest bits into scalar"),
                next_next_digest_bits,
            )
            .unwrap(),
        );
        z_out.push(
            pack_bits(
                cs.namespace(|| "Packing remaining next digest bits into scalar"),
                remaining_next_digest_bits,
            )
            .unwrap(),
        );

        Ok(z_out)
    }
}

impl<F> StepCircuit<F> for SHA512CompressionCircuit<F>
where
    F: PrimeField + PrimeFieldBits,
{
    fn arity(&self) -> usize {
        3
    }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        let msg_block_bits: Vec<Boolean> = self
            .input
            .iter()
            .enumerate()
            .map(|(i, b)| {
                Boolean::from(
                    AllocatedBit::alloc(cs.namespace(|| format!("input bit {i}")), Some(*b))
                        .unwrap(),
                )
            })
            .collect();

        Self::compress(cs, &msg_block_bits, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{digest_to_scalars, sha512_initial_digest_scalars};
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_sha512_compression_constraints() {
        let mut cs = TestConstraintSystem::<Fp>::new();
        let empty_input: Vec<u8> = vec![];
        let mut sha512_state_sequence = SHA512CompressionCircuit::new_state_sequence(empty_input);
        assert_eq!(sha512_state_sequence.len(), 1);

        let sha512_iteration = sha512_state_sequence.pop().unwrap();
        let mut z_in: Vec<AllocatedNum<Fp>> = vec![];

        for (i, s) in sha512_initial_digest_scalars().iter().enumerate() {
            z_in.push(
                AllocatedNum::alloc(cs.namespace(|| format!("z_in[{i}]")), || Ok(*s)).unwrap(),
            );
        }
        let z_out = sha512_iteration.synthesize(&mut cs, &z_in).unwrap();
        assert!(cs.is_satisfied());

        assert_eq!(z_out.len(), 3);
        let z_out_values = [
            z_out[0].get_value().unwrap_or_default(),
            z_out[1].get_value().unwrap_or_default(),
            z_out[2].get_value().unwrap_or_default(),
        ];

        let expected_digest: [u8; 64] =
        hex::decode("cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e")
            .expect("Failed to parse digest string")
            .try_into()
            .unwrap();

        let expected_zout: [Fp; 3] = digest_to_scalars(&expected_digest);

        assert_eq!(expected_zout[0], z_out_values[0]);
        assert_eq!(expected_zout[1], z_out_values[1]);
        assert_eq!(expected_zout[2], z_out_values[2]);

        assert_eq!(cs.num_constraints(), 68598);
    }
}

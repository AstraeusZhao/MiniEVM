use minievm::Word;

fn from_u128(value: u128) -> Word {
    Word::from_be_slice(&value.to_be_bytes())
}

#[test]
fn arithmetic_matches_native_u128_for_reproducible_inputs() {
    // Fixed deterministic generator, kept in the test rather than execution state.
    let mut seed = 0x1234_5678_9abc_def0u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for _ in 0..128 {
        let left = ((next() as u128) << 64) | next() as u128;
        let right = ((next() as u128) << 64) | next() as u128;
        let a = from_u128(left);
        let b = from_u128(right);
        assert_eq!(a.unsigned_div(b), from_u128(left / right));
        assert_eq!(a.unsigned_mod(b), from_u128(left % right));
        // Full products of u64 factors are independently representable by u128.
        let a = next();
        let b = next();
        assert_eq!(
            Word::from_u64(a).wrapping_mul(Word::from_u64(b)),
            from_u128(a as u128 * b as u128)
        );
    }
}

#[test]
fn full_width_carries_borrows_and_modulo_boundaries() {
    let max = Word([u64::MAX; 4]);
    assert_eq!(max.wrapping_add(Word::ONE), Word::ZERO);
    assert_eq!(Word::ZERO.wrapping_sub(Word::ONE), max);
    assert_eq!(max.wrapping_mul(max), Word::ONE);
    assert_eq!(max.unsigned_mod(Word::from_u64(3)), Word::ZERO);
    assert_eq!(max.unsigned_mod(Word::from_u64(10)), Word::from_u64(5));
    assert_eq!(max.unsigned_div(Word::ZERO), Word::ZERO);
    assert_eq!(max.unsigned_mod(Word::ZERO), Word::ZERO);
    for limb in 1..4 {
        let mut power = [0; 4];
        power[limb] = 1;
        let mut lower_max = [0; 4];
        lower_max[..limb].fill(u64::MAX);
        assert_eq!(Word(power).wrapping_sub(Word::ONE), Word(lower_max));
        assert_eq!(Word(lower_max).wrapping_add(Word::ONE), Word(power));
        assert_eq!(max.unsigned_mod(Word(power)), Word(lower_max));
    }
}

use super::*;

#[test]
fn packed_from_array_preserves_order_length_and_layout() {
    let empty_array: [Symbol; 0] = [];
    let empty = PackedString::<Symbol, 2>::from(empty_array);
    assert!(empty.is_empty());
    assert_eq!(empty.char_len(), 0);
    assert_eq!(empty.bits().bit_len(), 0);

    let singleton = PackedString::<SparseByte, 8>::from([SparseByte::Maximum]);
    assert_eq!(singleton.to_vec(), vec![SparseByte::Maximum]);
    assert_eq!(singleton.char_len(), 1);
    assert_eq!(singleton.bits().bit_len(), 8);

    let symbols = [Symbol::Zero, Symbol::Two, Symbol::One, Symbol::Zero];
    let symbol_from_array = PackedString::<Symbol, 2>::from(symbols);
    let symbol_from_chars = PackedString::from_chars(symbols);
    assert!(symbol_from_array == symbol_from_chars);
    assert_eq!(symbol_from_array.to_vec(), symbols);

    let oct_values = [Oct::V7; 22];
    let oct_from_array = PackedString::<Oct, 3>::from(oct_values);
    let oct_from_chars = PackedString::from_chars(oct_values);
    assert!(oct_from_array == oct_from_chars);
    assert_eq!(oct_from_array.char_len(), 22);
    assert_eq!(oct_from_array.bits().bit_len(), 22 * 3);
    assert_eq!(oct_from_array.bits().words(), oct_from_chars.bits().words());

    let wide_values = [WideCode(127); 10];
    let wide_from_array = PackedString::<WideCode, 7>::from(wide_values);
    let wide_from_chars = PackedString::from_chars(wide_values);
    assert!(wide_from_array == wide_from_chars);
    assert_eq!(wide_from_array.char_len(), 10);
    assert_eq!(wide_from_array.bits().bit_len(), 10 * 7);
    assert_eq!(
        wide_from_array.bits().words(),
        wide_from_chars.bits().words()
    );
}

#[test]
fn packed_from_slice_copies_codes_in_order() {
    let source = [Oct::V0, Oct::V7, Oct::V3, Oct::V1, Oct::V6];
    let from_slice = PackedString::<Oct, 3>::from(&source[..]);
    assert_eq!(from_slice.to_vec(), source);
    assert_eq!(from_slice.char_len(), source.len());
    assert_eq!(from_slice.bits().bit_len(), source.len() * 3);

    let empty_source: [Oct; 0] = [];
    let empty = PackedString::<Oct, 3>::from(&empty_source[..]);
    assert!(empty.is_empty());
    assert!(empty.bits().words().is_empty());

    let wide_source = [WideCode(0), WideCode(64), WideCode(127), WideCode(1)];
    let wide = PackedString::<WideCode, 7>::from(&wide_source[..]);
    assert_eq!(wide.to_vec(), wide_source);
    assert_eq!(wide.bits().bit_len(), wide_source.len() * 7);
}

#[test]
fn packed_from_chars_preserves_sequence_and_packed_layout() {
    let empty = PackedString::<Symbol, 2>::from_chars(core::iter::empty::<Symbol>());
    assert!(empty.is_empty());
    assert_eq!(empty.char_len(), 0);
    assert_eq!(empty.bits().bit_len(), 0);

    let symbols = [Symbol::Zero, Symbol::Two, Symbol::One, Symbol::Zero];
    let symbol_string = PackedString::<Symbol, 2>::from_chars(symbols);
    assert_eq!(symbol_string.to_vec(), symbols);
    assert_eq!(symbol_string.bits().bit_len(), symbols.len() * 2);

    let zero_codes = vec![0; 22];
    let zero_string = PackedString::<Oct, 3>::from_chars(zero_codes.iter().copied().map(oct));
    assert!(!zero_string.is_empty());
    assert_eq!(zero_string.char_len(), zero_codes.len());
    assert_eq!(zero_string.bits().bit_len(), zero_codes.len() * 3);
    assert_eq!(zero_string.to_vec(), vec![Oct::V0; 22]);

    let oct_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let oct_string = PackedString::<Oct, 3>::from_chars(oct_codes.iter().copied().map(oct));
    assert_eq!(
        oct_string.to_vec(),
        oct_codes.iter().copied().map(oct).collect::<Vec<_>>()
    );
    let oct_expected = BitString::from_iter(
        oct_codes
            .iter()
            .flat_map(|&code| (0..3).map(move |offset| code & (1 << offset) != 0)),
    );
    assert_eq!(oct_string.bits().bit_len(), oct_expected.bit_len());
    assert_eq!(oct_string.bits().words(), oct_expected.words());

    let wide_codes: Vec<_> = (0..10).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = PackedString::<WideCode, 7>::from_chars(wide_codes.iter().copied().map(wide));
    assert_eq!(
        wide_string.to_vec(),
        wide_codes.iter().copied().map(wide).collect::<Vec<_>>()
    );
    assert_eq!(wide_string.char_len(), 10);
    assert_eq!(wide_string.bits().bit_len(), 10 * 7);

    let byte_string = PackedString::<SparseByte, 8>::from_chars([
        SparseByte::Zero,
        SparseByte::Maximum,
        SparseByte::Middle,
    ]);
    assert_eq!(
        byte_string.to_vec(),
        vec![SparseByte::Zero, SparseByte::Maximum, SparseByte::Middle]
    );
    assert_eq!(byte_string.get(1), Some(SparseByte::Maximum));

    assert!(
        std::panic::catch_unwind(|| {
            PackedString::<UnsupportedWidth, 0>::from_chars(core::iter::empty());
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            PackedString::<UnsupportedWidth, 9>::from_chars(core::iter::empty());
        })
        .is_err()
    );
}

#[test]
fn packed_from_iter_matches_from_chars_for_single_pass_inputs() {
    let empty: PackedString<Symbol, 2> = core::iter::empty().collect();
    assert!(empty.is_empty());
    assert_eq!(empty.bits().bit_len(), 0);

    let oct_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let collected: PackedString<Oct, 3> = oct_codes.iter().copied().map(oct).collect();
    let constructed = PackedString::from_chars(oct_codes.iter().copied().map(oct));
    assert!(collected == constructed);
    assert_eq!(
        collected.to_vec(),
        oct_codes.iter().copied().map(oct).collect::<Vec<_>>()
    );
    assert_eq!(collected.bits().bit_len(), 22 * 3);
    assert_eq!(collected.bits().words(), constructed.bits().words());

    let mut next = 0;
    let lazy = core::iter::from_fn(|| {
        let value = (next < 10).then_some(WideCode((next * 11) as u8 % 128));
        next += 1;
        value
    });
    let wide: PackedString<WideCode, 7> = lazy.collect();
    assert_eq!(wide.char_len(), 10);
    assert_eq!(wide.bits().bit_len(), 10 * 7);
    assert_eq!(wide.last(), Some(WideCode(99)));

    let filtered: PackedString<SparseByte, 8> =
        [SparseByte::Zero, SparseByte::Maximum, SparseByte::Middle]
            .into_iter()
            .filter(|_| false)
            .collect();
    assert!(filtered.is_empty());
    assert_eq!(filtered.bits().bit_len(), 0);

    let zeros: PackedString<Oct, 3> = core::iter::repeat(Oct::V0).take(22).collect();
    assert!(!zeros.is_empty());
    assert_eq!(zeros.char_len(), 22);
    assert_eq!(zeros.bits().words(), &[0, 0]);

    assert!(
        std::panic::catch_unwind(|| {
            let _: PackedString<UnsupportedWidth, 0> = core::iter::empty().collect();
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            let _: PackedString<UnsupportedWidth, 9> = core::iter::empty().collect();
        })
        .is_err()
    );
}

#[test]
fn packed_repeat_produces_exactly_the_requested_codes() {
    let empty = PackedString::<Oct, 3>::repeat(Oct::V0, 0);
    assert!(empty.is_empty());
    assert_eq!(empty.char_len(), 0);
    assert_eq!(empty.bits().bit_len(), 0);

    let zero = PackedString::<Oct, 3>::repeat(Oct::V0, 22);
    assert!(!zero.is_empty());
    assert_eq!(zero.to_vec(), vec![Oct::V0; 22]);
    assert_eq!(zero.bits().bit_len(), 22 * 3);

    let oct = PackedString::<Oct, 3>::repeat(Oct::V7, 22);
    let oct_expected = PackedString::from_chars(core::iter::repeat_n(Oct::V7, 22));
    assert!(oct == oct_expected);
    assert_eq!(oct.get(0), Some(Oct::V7));
    assert_eq!(oct.get(21), Some(Oct::V7));
    assert_eq!(oct.bits().words(), oct_expected.bits().words());

    let binary = PackedString::<PackedSymbol, 1>::repeat(PackedSymbol::One, 65);
    assert_eq!(binary.char_len(), 65);
    assert_eq!(binary.bits().bit_len(), 65);
    assert_eq!(binary.first(), Some(PackedSymbol::One));
    assert_eq!(binary.last(), Some(PackedSymbol::One));

    let wide = PackedString::<WideCode, 7>::repeat(WideCode(127), 10);
    assert_eq!(wide.to_vec(), vec![WideCode(127); 10]);
    assert_eq!(wide.bits().bit_len(), 10 * 7);
    assert_eq!(wide.bits().words().len(), 2);

    let bytes = PackedString::<SparseByte, 8>::repeat(SparseByte::Maximum, 9);
    assert_eq!(bytes.to_vec(), vec![SparseByte::Maximum; 9]);
    assert_eq!(bytes.bits().bit_len(), 9 * 8);

    assert!(
        std::panic::catch_unwind(|| {
            PackedString::<UnsupportedWidth, 0>::repeat(UnsupportedWidth, 0);
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            PackedString::<UnsupportedWidth, 9>::repeat(UnsupportedWidth, 1);
        })
        .is_err()
    );
}

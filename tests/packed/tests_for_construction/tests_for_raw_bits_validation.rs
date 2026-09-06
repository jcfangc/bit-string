use super::*;

#[test]
fn from_bits_rejects_misaligned_and_unknown_codes() {
    assert!(PackedString::<Symbol, 2>::from_bits(BitString::from_iter([true])).is_none());
    assert!(PackedString::<Symbol, 2>::from_bits(BitString::from_iter([true, true])).is_none());
}

fn assert_empty_from_bits<C, const BITS: u8>()
where
    C: PackedChar<BITS>,
{
    let empty =
        PackedString::<C, BITS>::from_bits(BitString::from_iter(core::iter::empty::<bool>()))
            .expect("empty aligned payload should be accepted");
    assert!(empty.is_empty());
    assert_eq!(empty.char_len(), 0);
}

#[test]
fn from_bits_validates_alignment_codes_and_widths() {
    let raw_codes = |codes: &[u8], bits: u8| {
        BitString::from_iter(
            codes
                .iter()
                .flat_map(|&code| (0..bits).map(move |offset| code & (1 << offset) != 0)),
        )
    };

    assert_empty_from_bits::<PackedSymbol, 1>();
    assert_empty_from_bits::<Symbol, 2>();
    assert_empty_from_bits::<Oct, 3>();
    assert_empty_from_bits::<SparseByte, 8>();

    let oct_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let oct_original = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_restored = PackedString::<Oct, 3>::from_bits(oct_original.clone().into_bits())
        .expect("valid three-bit payload should be accepted");
    assert!(oct_restored == oct_original);
    assert_eq!(
        oct_restored.to_vec(),
        oct_codes.iter().copied().map(oct).collect::<Vec<_>>()
    );

    let wide_codes: Vec<_> = (0..10).map(|index| (index * 11) as u8 % 128).collect();
    let wide_original = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_restored = PackedString::<WideCode, 7>::from_bits(wide_original.clone().into_bits())
        .expect("valid seven-bit payload should be accepted");
    assert!(wide_restored == wide_original);
    assert_eq!(
        wide_restored.to_vec(),
        wide_codes.iter().copied().map(wide).collect::<Vec<_>>()
    );

    let maximum_byte = PackedString::<SparseByte, 8>::from_bits(raw_codes(&[255], 8))
        .expect("maximum eight-bit code should be accepted");
    assert_eq!(maximum_byte.to_vec(), vec![SparseByte::Maximum]);

    assert!(PackedString::<Symbol, 2>::from_bits(raw_codes(&[3], 2)).is_none());
    assert!(PackedString::<Symbol, 2>::from_bits(raw_codes(&[0, 3, 0], 2)).is_none());
    assert!(PackedString::<Symbol, 2>::from_bits(raw_codes(&[0, 0, 3], 2)).is_none());
    assert!(PackedString::<SparseByte, 8>::from_bits(raw_codes(&[1], 8)).is_none());
    assert!(PackedString::<Oct, 3>::from_bits(BitString::from_iter([true; 65])).is_none());
    assert!(PackedString::<WideCode, 7>::from_bits(BitString::from_iter([true; 69])).is_none());

    assert!(
        std::panic::catch_unwind(|| {
            PackedString::<UnsupportedWidth, 0>::from_bits(BitString::new());
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            PackedString::<UnsupportedWidth, 9>::from_bits(BitString::new());
        })
        .is_err()
    );
}

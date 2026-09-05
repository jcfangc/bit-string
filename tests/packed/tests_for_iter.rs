use super::{Oct, PackedSymbol, SparseByte, WideCode, oct, packed_as, wide};
use bit_string::{PackedStr, PackedString, traits::PackedChar};
use int_intervals::UsizeCO;

fn assert_iterator<C, const BITS: u8>(view: PackedStr<'_, C, BITS>, expected: &[C])
where
    C: PackedChar<BITS> + core::fmt::Debug,
{
    let mut iterator = view.iter();
    assert_eq!(iterator.len(), expected.len());
    assert_eq!(iterator.size_hint(), (expected.len(), Some(expected.len())));

    let mut front = 0;
    let mut back = expected.len();
    while front < back {
        if front % 2 == 0 {
            assert_eq!(iterator.next(), Some(expected[front]));
            front += 1;
        } else {
            back -= 1;
            assert_eq!(iterator.next_back(), Some(expected[back]));
        }
        assert_eq!(iterator.len(), back - front);
        assert_eq!(iterator.size_hint(), (back - front, Some(back - front)));
    }
    assert_eq!(iterator.next(), None);
    assert_eq!(iterator.next_back(), None);
}

#[test]
fn packed_str_iter_is_exact_double_ended_and_view_relative() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_iterator(empty.as_packed_str(), &[]);

    let bytes = packed_as::<SparseByte, 8>(&[0, 255], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert_iterator(
        bytes.as_packed_str(),
        &[SparseByte::Zero, SparseByte::Maximum],
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_expected: Vec<_> = oct_codes[20..24].iter().copied().map(oct).collect();
    assert_iterator(oct_view, &oct_expected);
    assert_eq!((&oct_view).into_iter().collect::<Vec<_>>(), oct_expected);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    let wide_expected: Vec<_> = wide_codes[8..13].iter().copied().map(wide).collect();
    assert_iterator(wide_view, &wide_expected);

    let iterator_after_view_drop = {
        let view = wide_owner
            .as_packed_str()
            .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
        view.iter()
    };
    assert_eq!(iterator_after_view_drop.collect::<Vec<_>>(), wide_expected);
}

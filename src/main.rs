#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const PERMUTATION_TABLE_LESSER: [(u32, u32, u32, u32); 16] = [
    (0, 0, 0, 0),
    (0, 0, 0, 0),
    (1, 0, 0, 0),
    (0, 1, 0, 0),
    (2, 0, 0, 0),
    (0, 2, 0, 0),
    (1, 2, 0, 0),
    (0, 1, 2, 0),
    (3, 0, 0, 0),
    (0, 3, 0, 0),
    (1, 3, 0, 0),
    (0, 1, 3, 0),
    (2, 3, 0, 0),
    (0, 2, 3, 0),
    (1, 2, 3, 0),
    (0, 1, 2, 3),
];

unsafe fn permute(current: __m128, mask: i32) -> __m128 {
    match mask {
        0 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[0])),
        1 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[1])),
        2 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[2])),
        3 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[3])),
        4 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[4])),
        5 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[5])),
        6 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[6])),
        7 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[7])),
        8 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[8])),
        9 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[9])),
        10 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[10])),
        11 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[11])),
        12 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[12])),
        13 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[13])),
        14 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[14])),
        15 => _mm_permute_ps(current, control(PERMUTATION_TABLE_LESSER[15])),
        _ => unreachable!(),
    }
}

// NOTE, this is just the above reversed
const PERMUTATION_TABLE_GREATER: [(u32, u32, u32, u32); 16] = [
    (0, 1, 2, 3),
    (1, 2, 3, 0),
    (0, 2, 3, 0),
    (2, 3, 0, 0),
    (0, 1, 3, 0),
    (1, 3, 0, 0),
    (0, 3, 0, 0),
    (3, 0, 0, 0),
    (0, 1, 2, 0),
    (1, 2, 0, 0),
    (0, 2, 0, 0),
    (2, 0, 0, 0),
    (0, 1, 0, 0),
    (1, 0, 0, 0),
    (0, 0, 0, 0),
    (0, 0, 0, 0),
];

unsafe fn sort(elements: &[u32]) {
    let mut bottom: Vec<u32> = Vec::with_capacity(elements.len());
    let mut top: Vec<u32> = Vec::with_capacity(elements.len());

    let pivot = elements[0];

    let pivot_array = [pivot; 4];
    let pivot = _mm_loadu_si128((&pivot_array).as_ptr() as _);

    let mut i = 0;

    while i < elements.len() {
        let current = _mm_loadu_si128(elements.as_ptr().add(i) as _);

        dbg!(std::mem::transmute::<_, [u32; 4]>(current));

        let greater_than = _mm_cmpgt_epi32(current, pivot);

        let greater_than_mask = _mm_movemask_ps(std::mem::transmute(greater_than));

        println!("0b{:04b}", greater_than_mask);

        let current = std::mem::transmute(current);

        let greater = permute(current, greater_than_mask);
        let lesser = permute(current, !greater_than_mask & 0b1111);

        let bigger = greater_than_mask.count_ones() as usize;
        let smaller = 4 - bigger as usize;

        let lesser: [u32; 4] = std::mem::transmute(lesser);
        let greater: [u32; 4] = std::mem::transmute(greater);

        // the elements start from the right (so index 3 for simd is really index 0 for rust)
        bottom.extend(lesser[(4 - smaller)..].iter().rev());
        top.extend(greater[(4 - bigger)..].iter().rev());

        i += 4;
    }

    dbg!(bottom, top);
}

const fn control((a, b, c, d): (u32, u32, u32, u32)) -> i32 {
    (d | (c << 2) | (b << 4) | (a << 6)) as i32
}

fn main() {
    // unsafe { sort(&[4, 65, 2, 2, 65, 6, 3, 2, 1, 4, 6, 7]) }
    // unsafe { sort(&[4, 65, 2, 2]) }
    unsafe { sort(&[4, 65, 2, 2, 63, 6, 3, 2]) }
}

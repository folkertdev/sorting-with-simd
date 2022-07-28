#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const PERMUTATION_TABLE_LESSER: [(i32, i32, i32, i32); 16] = [
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

#[target_feature(enable = "avx")]
unsafe fn vperilps(mut current: __m128, mask: (i32, i32, i32, i32)) -> __m128 {
    let mask = _mm_set_epi32(mask.3, mask.2, mask.1, mask.0);

    std::arch::asm!(
        "vpermilps {a:y}, {a:y}, {m:y}",
        a = inout(ymm_reg) current,
        m = in(ymm_reg) mask,

    );

    current
}

#[inline(always)]
unsafe fn permute(current: __m128, mask: i32) -> __m128 {
    vperilps(current, PERMUTATION_TABLE_LESSER[mask as usize])
}

unsafe fn partition4(elements: &mut [i32], scratchpad: &mut [i32]) -> usize {
    let mut bottom = 0;
    let mut top = 0;

    // naively pick the last element as the pivot. That is not optimal, but it's simple
    let pivot_element = elements[elements.len() - 1];

    let pivot = _mm_set1_epi32(pivot_element);

    let mut i = 0;

    while i + 3 < elements.len() {
        let current = _mm_loadu_si128(elements.as_ptr().add(i) as _);

        let greater_than = _mm_cmpgt_epi32(current, pivot);

        let greater_than_mask = _mm_movemask_ps(std::mem::transmute(greater_than));

        let current = std::mem::transmute(current);

        // flipped from the paper; this gives an ascending sort
        let greater = permute(current, greater_than_mask);
        let lesser = permute(current, !greater_than_mask & 0b1111);

        let bigger = greater_than_mask.count_ones() as usize;
        let smaller = 4 - bigger as usize;

        _mm_storeu_ps(scratchpad.as_ptr().add(top) as *mut _, greater);
        top += bigger;

        _mm_storeu_ps(elements.as_ptr().add(bottom) as *mut _, lesser);
        bottom += smaller;

        i += 4;
    }

    // process any trailing elements
    while i < elements.len() {
        let value = elements[i];

        if value > pivot_element {
            scratchpad[top] = value;
            top += 1;
        } else {
            elements[bottom] = value;
            bottom += 1;
        }

        i += 1;
    }

    let n = elements.len() - top;

    elements[n..].copy_from_slice(&scratchpad[..top]);

    bottom
}

#[allow(dead_code)]
pub fn sort(input: &mut [i32]) {
    let mut scratchpad = vec![0; input.len()];

    sort_help_old(input, &mut scratchpad)
}

fn sort_help_old(input: &mut [i32], scratchpad: &mut [i32]) {
    if input.len() <= 1 {
        return;
    }

    let n = unsafe { partition4(input, scratchpad) };

    if n == input.len() {
        sort_help_old(&mut input[..n - 1], scratchpad);
    } else if n == 0 {
        panic!()
    } else {
        sort_help_old(&mut input[..n], scratchpad);
        sort_help_old(&mut input[n..], scratchpad);
    }
}

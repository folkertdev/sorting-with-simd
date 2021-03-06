#![feature(stdsimd)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const fn control((a, b, c, d): (i32, i32, i32, i32)) -> i32 {
    (a | (b << 2) | (c << 4) | (d << 6)) as i32
}

type Mask = (i32, i32, i32, i32);

const PERMUTATION_TABLE_LESSER: [Mask; 16] = [
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

pub unsafe fn partition_avx512(elements: &mut [i32], scratchpad: &mut [i32]) -> usize {
    let mut bottom = 0;
    let mut top = 0;

    // When the selected pivot element is the last element in the list, it performs a stable sort.
    let pivot_element = elements[elements.len() - 1];

    // let mut pivots = [elements[0], elements[1], elements[2]];
    // pivots.sort_unstable();
    // let pivot_element = pivots[1];

    let pivot = _mm512_set1_epi32(pivot_element);

    let mut i = 0;

    const N: usize = 1;
    const W: usize = 16;

    let mut currents = [pivot; N];

    let empty_mask: __mmask16 = 0;
    let mut greater_thans = [empty_mask; N];

    while i + (N * W) <= elements.len() {
        for (n, current) in currents.iter_mut().enumerate() {
            *current = _mm512_loadu_epi32(elements.as_ptr().add(i + n * W) as _);
        }

        for (current, greater_than) in currents.into_iter().zip(greater_thans.iter_mut()) {
            *greater_than = _mm512_cmpgt_epi32_mask(current, pivot);
        }

        for (greater_than_mask, current) in greater_thans.into_iter().zip(currents) {
            let bigger = greater_than_mask.count_ones() as usize;
            let smaller = W - bigger as usize;

            let greater = _mm512_maskz_compress_epi32(greater_than_mask, current);
            _mm512_storeu_epi32(scratchpad.as_ptr().add(top) as *mut _, greater);
            top += bigger;

            let lesser = _mm512_maskz_compress_epi32(!greater_than_mask, current);
            _mm512_storeu_epi32(elements.as_ptr().add(bottom) as *mut _, lesser);
            bottom += smaller;
        }

        i += W * N;
    }

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

#[inline(always)]
unsafe fn permute(current: __m128, mask: i32) -> __m128 {
    vperilps(current, PERMUTATION_TABLE_LESSER[mask as usize])
}

#[inline(never)]
unsafe fn partition16(elements: &mut [i32], scratchpad: &mut [i32]) -> usize {
    let mut bottom = 0;
    let mut top = 0;

    // When the selected pivot element is the last element in the list, it performs a stable sort.
    let pivot_element = elements[elements.len() - 1];

    // let mut pivots = [elements[0], elements[1], elements[2]];
    // pivots.sort_unstable();
    // let pivot_element = pivots[1];

    let pivot = _mm_set1_epi32(pivot_element);

    let mut i = 0;

    const N: usize = 16;
    const W: usize = 4;

    let mut currents = [pivot; N];
    let mut greater_thans = [pivot; N];
    let mut masks = [0; N];

    while i + (N * W) <= elements.len() {
        for (n, current) in currents.iter_mut().enumerate() {
            *current = _mm_loadu_si128(elements.as_ptr().add(i + n * W) as _);
        }

        for (current, greater_than) in currents.into_iter().zip(greater_thans.iter_mut()) {
            *greater_than = _mm_cmpgt_epi32(current, pivot);
        }

        for (greater_than, mask) in greater_thans.into_iter().zip(masks.iter_mut()) {
            *mask = _mm_movemask_ps(std::mem::transmute(greater_than));
        }

        for (greater_than_mask, current) in masks.into_iter().zip(currents) {
            let current = std::mem::transmute(current);

            // flipped from the paper; this gives an ascending sort
            let greater = permute(current, greater_than_mask);
            let lesser = permute(current, !greater_than_mask & 0b1111);

            let bigger = greater_than_mask.count_ones() as usize;
            let smaller = W - bigger as usize;

            _mm_storeu_ps(scratchpad.as_ptr().add(top) as *mut _, greater);
            top += bigger;

            _mm_storeu_ps(elements.as_ptr().add(bottom) as *mut _, lesser);
            bottom += smaller;
        }

        i += W * N;
    }

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

unsafe fn partition8(elements: &mut [i32], scratchpad: &mut [i32]) -> usize {
    let mut bottom = 0;
    let mut top = 0;

    // When the selected pivot element is the last element in the list, it performs a stable sort.
    let pivot_element = elements[elements.len() - 1];
    let pivot = _mm_set1_epi32(pivot_element);

    let mut i = 0;

    while i + 7 < elements.len() {
        let current1 = _mm_loadu_si128(elements.as_ptr().add(i) as _);
        let current2 = _mm_loadu_si128(elements.as_ptr().add(i + 4) as _);

        let greater_than1 = _mm_cmpgt_epi32(current1, pivot);
        let greater_than2 = _mm_cmpgt_epi32(current2, pivot);

        let greater_than_mask1 = _mm_movemask_ps(std::mem::transmute(greater_than1));
        let greater_than_mask2 = _mm_movemask_ps(std::mem::transmute(greater_than2));

        let current1 = std::mem::transmute(current1);
        let current2 = std::mem::transmute(current2);

        let greater1 = permute(current1, greater_than_mask1);
        let greater2 = permute(current2, greater_than_mask2);

        let lesser1 = permute(current1, 15 - greater_than_mask1);
        let lesser2 = permute(current2, 15 - greater_than_mask2);

        let bigger1 = greater_than_mask1.count_ones() as usize;
        let bigger2 = greater_than_mask2.count_ones() as usize;

        _mm_storeu_ps(scratchpad.as_ptr().add(top) as *mut _, greater1);
        top += bigger1;

        _mm_storeu_ps(scratchpad.as_ptr().add(top) as *mut _, greater2);
        top += bigger2;

        let smaller1 = 4 - bigger1 as usize;
        let smaller2 = 4 - bigger2 as usize;

        _mm_storeu_ps(elements.as_ptr().add(bottom) as *mut _, lesser1);
        bottom += smaller1;

        _mm_storeu_ps(elements.as_ptr().add(bottom) as *mut _, lesser2);
        bottom += smaller2;

        i += 8;
    }

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

    elements[bottom..].copy_from_slice(&scratchpad[..top]);

    bottom
}

unsafe fn partition4(elements: &mut [i32], scratchpad: &mut [i32]) -> usize {
    let mut bottom = 0;
    let mut top = 0;

    // When the selected pivot element is the last element in the list, it performs a stable sort.
    let pivot_element = elements[elements.len() - 1];

    let pivot = _mm_set1_epi32(pivot_element);

    let mut i = 0;

    while i + 3 < elements.len() {
        let current = _mm_loadu_si128(elements.as_ptr().add(i) as _);

        // dbg!(std::mem::transmute::<_, [i32; 4]>(current));

        let greater_than = _mm_cmpgt_epi32(current, pivot);

        let greater_than_mask = _mm_movemask_ps(std::mem::transmute(greater_than));

        // println!("0b{:04b}", greater_than_mask);

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

pub fn sort(input: &mut [i32]) {
    let mut scratchpad = vec![0; input.len()];

    sort_help(input, &mut scratchpad)
}

pub fn sort_old(input: &mut [i32]) {
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

fn insertion_sort_by<T, F>(arr: &mut [T], mut compare: F)
where
    F: FnMut(&T, &T) -> std::cmp::Ordering,
{
    for i in 1..arr.len() {
        let val = &arr[i];
        let mut j = i;
        let pos = arr[..i]
            .binary_search_by(|x| compare(x, val))
            .unwrap_or_else(|pos| pos);
        // Swap all elements until specific position.
        while j > pos {
            arr.swap(j - 1, j);
            j -= 1;
        }
    }
}

fn sort_help(initial: &mut [i32], scratchpad: &mut [i32]) {
    let mut stack = vec![0..initial.len()];

    while let Some(range) = stack.pop() {
        let start = range.start;
        let end = range.end;
        let input = &mut initial[start..end];

        if input.len() <= 1 {
            continue;
        }

        if input.len() < 16 {
            input.sort_unstable();
            continue;
        }

        let n = unsafe { partition4(input, scratchpad) };

        if n == input.len() {
            stack.push(start..end - 1);
        } else if n == 0 {
            panic!()
        } else {
            stack.push(start + n..end);
            stack.push(start..start + n);
        }
    }
}

// taken from https://github.com/komrad36/SortingNetworks/blob/master/sorts.cpp
unsafe fn simdsort4(current: &mut __m128i) {
    let pass1_add4: __m128i = _mm_setr_epi32(1, 1, 3, 3);
    let pass2_add4: __m128i = _mm_setr_epi32(2, 3, 2, 3);
    let pass3_add4: __m128i = _mm_setr_epi32(0, 2, 2, 3);

    let mut a = *current;
    let mut b: __m128i;

    b = _mm_shuffle_epi32(a, 177);
    b = _mm_cmpgt_epi32(b, a);
    b = _mm_add_epi32(b, pass1_add4);
    a = _mm_castps_si128(_mm_permutevar_ps(_mm_castsi128_ps(a), b));

    b = _mm_shuffle_epi32(a, 78);
    b = _mm_cmpgt_epi32(b, a);
    b = _mm_add_epi32(b, b);
    b = _mm_add_epi32(b, pass2_add4);
    a = _mm_castps_si128(_mm_permutevar_ps(_mm_castsi128_ps(a), b));

    b = _mm_shuffle_epi32(a, 216);
    b = _mm_cmpgt_epi32(b, a);
    b = _mm_add_epi32(b, pass3_add4);
    let ret = _mm_permutevar_ps(_mm_castsi128_ps(a), b);

    _mm_storeu_ps(current as *mut _ as *mut f32, ret);
}

unsafe fn partition_vec(
    elements: &mut [i32],
    val: __m128i,
    pivotvec: __m128i,
    left_w: usize,
    right_w: usize,
) {
    let mask = _mm_cmpgt_epi32(val, pivotvec);
    let mask = _mm_movemask_ps(std::mem::transmute(mask));
}

unsafe fn partition4_in_place(elements: &mut [i32], pivot: i32) -> usize {
    const VEC_SIZE: usize = 4;

    let length = elements.len();
    let pivotvec = _mm_set1_epi32(pivot);

    let mut left = 0;
    let left_w = 0;
    let left_vec = _mm_loadu_si128(elements.as_ptr().add(left) as _);
    left += VEC_SIZE;

    let right = length - VEC_SIZE;
    let right_w = length;
    let right_vec = _mm_loadu_si128(elements.as_ptr().add(right) as _);
    left += VEC_SIZE;

    while (left + VEC_SIZE) <= right {
        let val;

        if left - left_w <= right_w - right {
            val = _mm_loadu_si128(elements.as_ptr().add(left) as _);
            left += VEC_SIZE;
        } else {
            right -= VEC_SIZE;
            val = _mm_loadu_si128(elements.as_ptr().add(right) as _);
        }

        (left_w, right_w) = partition_vec(elements, val, pivotvec, left_w, right_w);
    }

    left_w
    /*
    let mut bottom = 0;
    let mut top = 0;

    // When the selected pivot element is the last element in the list, it performs a stable sort.
    let pivot_element = elements[elements.len() - 1];

    let pivot = _mm_set1_epi32(pivot_element);

    let mut i = 0;

    while i + 3 < elements.len() {
        let current = _mm_loadu_si128(elements.as_ptr().add(i) as _);

        // dbg!(std::mem::transmute::<_, [i32; 4]>(current));

        let greater_than = _mm_cmpgt_epi32(current, pivot);

        let greater_than_mask = _mm_movemask_ps(std::mem::transmute(greater_than));

        // println!("0b{:04b}", greater_than_mask);

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
    */
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! In-place sorting.

use std::cmp::Ordering;

fn quicksort_helper<T>(arr: &mut [T], left: isize, right: isize, compare: fn(&T, &T) -> Ordering) {
    if right <= left {
        return
    }

    let mut i: isize = left - 1;
    let mut j: isize = right;
    let mut p: isize = i;
    let mut q: isize = j;
    unsafe {
        let v: *mut T = &mut arr[right as usize];
        loop {
            i += 1;
            while compare(&arr[i as usize], &*v) == Ordering::Less {
                i += 1
            }
            j -= 1;
            while compare(&*v, &arr[j as usize]) == Ordering::Less {
                if j == left {
                    break
                }
                j -= 1;
            }
            if i >= j {
                break
            }
            arr.swap(i as usize, j as usize);
            if compare(&arr[i as usize], &*v) == Ordering::Equal {
                p += 1;
                arr.swap(p as usize, i as usize)
            }
            if compare(&*v, &arr[j as usize]) == Ordering::Equal {
                q -= 1;
                arr.swap(j as usize, q as usize)
            }
        }
    }

    arr.swap(i as usize, right as usize);
    j = i - 1;
    i += 1;
    let mut k: isize = left;
    while k < p {
        arr.swap(k as usize, j as usize);
        k += 1;
        j -= 1;
        assert!(k < arr.len() as isize);
    }
    k = right - 1;
    while k > q {
        arr.swap(i as usize, k as usize);
        k -= 1;
        i += 1;
        assert!(k != 0);
    }

    quicksort_helper(arr, left, j, compare);
    quicksort_helper(arr, i, right, compare);
}

/// An in-place quicksort.
///
/// The algorithm is from Sedgewick and Bentley, "Quicksort is Optimal":
///     http://www.cs.princeton.edu/~rs/talks/QuicksortIsOptimal.pdf
pub fn quicksort_by<T>(arr: &mut [T], compare: fn(&T, &T) -> Ordering) {
    if arr.len() <= 1 {
        return
    }

    let len = arr.len();
    quicksort_helper(arr, 0, (len - 1) as isize, compare);
}

#[cfg(test)]
pub mod test {
    use std::rand;
    use std::rand::Rng;

    use sort;

    #[test]
    pub fn random() {
        let mut rng = rand::thread_rng();
        for _ in range(0u32, 50000u32) {
            let len: usize = rng.gen();
            let mut v: Vec<isize> = rng.gen_iter::<isize>().take((len % 32) + 1).collect();
            fn compare_ints(a: &isize, b: &isize) -> Ordering { a.cmp(b) }
            sort::quicksort_by(v.as_mut_slice(), compare_ints);
            for i in range(0, v.len() - 1) {
                assert!(v[i] <= v[i + 1])
            }
        }
    }
}


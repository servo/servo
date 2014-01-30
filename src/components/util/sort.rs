/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! In-place sorting.

fn quicksort_helper<T:Ord + Eq>(arr: &mut [T], left: int, right: int) {
    if right <= left {
        return
    }

    let mut i: int = left - 1;
    let mut j: int = right;
    let mut p: int = i;
    let mut q: int = j;
    unsafe {
        let v: *mut T = &mut arr[right];
        loop {
            i += 1;
            while arr[i] < (*v) {
                i += 1
            }
            j -= 1;
            while (*v) < arr[j] {
                if j == left {
                    break
                }
                j -= 1;
            }
            if i >= j {
                break
            }
            arr.swap(i as uint, j as uint);
            if arr[i] == (*v) {
                p += 1;
                arr.swap(p as uint, i as uint)
            }
            if (*v) == arr[j] {
                q -= 1;
                arr.swap(j as uint, q as uint)
            }
        }
    }

    arr.swap(i as uint, right as uint);
    j = i - 1;
    i += 1;
    let mut k: int = left;
    while k < p {
        arr.swap(k as uint, j as uint);
        k += 1;
        j -= 1;
        assert!(k < arr.len() as int);
    }
    k = right - 1;
    while k > q {
        arr.swap(i as uint, k as uint);
        k -= 1;
        i += 1;
        assert!(k != 0);
    }

    quicksort_helper(arr, left, j);
    quicksort_helper(arr, i, right);
}

/// An in-place quicksort.
///
/// The algorithm is from Sedgewick and Bentley, "Quicksort is Optimal":
///     http://www.cs.princeton.edu/~rs/talks/QuicksortIsOptimal.pdf
pub fn quicksort<T:Ord + Eq>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return
    }

    let len = arr.len();
    quicksort_helper(arr, 0, (len - 1) as int);
}

#[cfg(test)]
pub mod test {
    use std::rand::{Rng, task_rng};
    use std::rand;

    use sort;

    #[test]
    pub fn random() {
        let mut rng = rand::task_rng();
        for _ in range(0, 50000) {
            let len: uint = rng.gen();
            let mut v: ~[int] = rng.gen_vec((len % 32) + 1);
            sort::quicksort(v);
            for i in range(0, v.len() - 1) {
                assert!(v[i] <= v[i + 1])
            }
        }
    }
}


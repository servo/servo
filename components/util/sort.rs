/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! In-place sorting.

fn quicksort_helper<T>(arr: &mut [T], left: int, right: int, compare: fn(&T, &T) -> Ordering) {
    if right <= left {
        return
    }

    let mut i: int = left - 1;
    let mut j: int = right;
    let mut p: int = i;
    let mut q: int = j;
    unsafe {
        let v: *mut T = &mut arr[right as uint];
        loop {
            i += 1;
            while compare(&arr[i as uint], &*v) == Less {
                i += 1
            }
            j -= 1;
            while compare(&*v, &arr[j as uint]) == Less {
                if j == left {
                    break
                }
                j -= 1;
            }
            if i >= j {
                break
            }
            arr.swap(i as uint, j as uint);
            if compare(&arr[i as uint], &*v) == Equal {
                p += 1;
                arr.swap(p as uint, i as uint)
            }
            if compare(&*v, &arr[j as uint]) == Equal {
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
    quicksort_helper(arr, 0, (len - 1) as int, compare);
}

#[cfg(test)]
pub mod test {
    use std::rand;
    use std::rand::Rng;

    use sort;

    #[test]
    pub fn random() {
        let mut rng = rand::task_rng();
        for _ in range(0u32, 50000u32) {
            let len: uint = rng.gen();
            let mut v: Vec<int> = rng.gen_iter::<int>().take((len % 32) + 1).collect();
            fn compare_ints(a: &int, b: &int) -> Ordering { a.cmp(b) }
            sort::quicksort_by(v.as_mut_slice(), compare_ints);
            for i in range(0, v.len() - 1) {
                assert!(v[i] <= v[i + 1])
            }
        }
    }
}


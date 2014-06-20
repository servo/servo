/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use serialize::{Encoder, Encodable};
use static_atoms::StaticAtom;
use std::fmt;
use std::hash::{Hash, Hasher, sip};
use std::mem;
use std::ptr;
use std::slice;
use std::slice::bytes;
use std::str;
use std::sync::atomics::{AtomicInt, SeqCst};
use sync::mutex::Mutex;
use sync::one::{Once, ONCE_INIT};
use std::rt::heap;

static mut global_string_cache_ptr: *mut StringCache = 0 as *mut StringCache;

static INLINE_SHIFT_BITS: uint = 32;
static ENTRY_ALIGNMENT: uint = 16;

// NOTE: Deriving Eq here implies that a given string must always
// be interned the same way.
#[repr(u8)]
#[deriving(Eq, TotalEq)]
enum AtomType {
    Inline = 0,
    Dynamic = 1,
    Static = 2,
}

struct StringCache {
    hasher: sip::SipHasher,
    buckets: [*StringCacheEntry, ..4096],
    lock: Mutex,
}

struct StringCacheEntry {
    next_in_bucket: *StringCacheEntry,
    hash: u64,
    ref_count: AtomicInt,
    string: String,
}

impl StringCacheEntry {
    fn new(next: *StringCacheEntry, hash: u64, string_to_add: &str) -> StringCacheEntry {
        StringCacheEntry {
            next_in_bucket: next,
            hash: hash,
            ref_count: AtomicInt::new(1),
            string: string_to_add.to_string(),
        }
    }
}

impl StringCache {
    fn new() -> StringCache {
        StringCache {
            hasher: sip::SipHasher::new(),
            buckets: unsafe { mem::zeroed() },
            lock: Mutex::new(),
        }
    }

    fn add(&mut self, string_to_add: &str) -> u64 {
        let _guard = self.lock.lock();

        let hash = self.hasher.hash(&string_to_add);
        let bucket_index = (hash & (self.buckets.len()-1) as u64) as uint;
        let mut ptr = self.buckets[bucket_index];

        while ptr != ptr::null() {
            let value = unsafe { &*ptr };
            if value.hash == hash && value.string.as_slice() == string_to_add {
                break;
            }
            ptr = value.next_in_bucket;
        }

        if ptr == ptr::null() {
            unsafe {
                ptr = heap::allocate(mem::size_of::<StringCacheEntry>(), ENTRY_ALIGNMENT)
                        as *StringCacheEntry;
                mem::overwrite(ptr as *mut StringCacheEntry,
                            StringCacheEntry::new(self.buckets[bucket_index], hash, string_to_add));
            }
            self.buckets[bucket_index] = ptr;
        } else {
            let value: &mut StringCacheEntry = unsafe { mem::transmute(ptr) };
            value.ref_count.fetch_add(1, SeqCst);
        }

        assert!(ptr != ptr::null());
        ptr as u64
    }

    fn remove(&mut self, key: u64) {
        let _guard = self.lock.lock();

        let ptr = key as *StringCacheEntry;
        let value: &mut StringCacheEntry = unsafe { mem::transmute(ptr) };

        if value.ref_count.fetch_sub(1, SeqCst) == 1 {
            let bucket_index = (value.hash & (self.buckets.len()-1) as u64) as uint;

            let mut current = self.buckets[bucket_index];
            let mut prev: *mut StringCacheEntry = ptr::mut_null();

            while current != ptr::null() {
                if current == ptr {
                    if prev != ptr::mut_null() {
                        unsafe { (*prev).next_in_bucket = (*current).next_in_bucket };
                    } else {
                        unsafe { self.buckets[bucket_index] = (*current).next_in_bucket };
                    }
                    break;
                }
                prev = current as *mut StringCacheEntry;
                unsafe { current = (*current).next_in_bucket };
            }
            assert!(current != ptr::null());

            unsafe {
                ptr::read(ptr as *StringCacheEntry);
                heap::deallocate(ptr as *mut StringCacheEntry as *mut u8,
                    mem::size_of::<StringCacheEntry>(), ENTRY_ALIGNMENT);
            }
        }
    }
}

#[deriving(Eq, TotalEq)]
pub struct Atom {
    data: u64
}

impl Atom {
    pub fn from_static(atom_id: StaticAtom) -> Atom {
        Atom {
            data: (atom_id as u64 << INLINE_SHIFT_BITS) | (Static as u64)
        }
    }

    pub fn from_string(string_to_add: &String) -> Atom {
        Atom::from_slice(string_to_add.as_slice())
    }

    pub fn from_slice(string_to_add: &str) -> Atom {
        match from_str::<StaticAtom>(string_to_add) {
            Some(atom_id) => {
                Atom::from_static(atom_id)
            },
            None => {
                if string_to_add.len() < 8 {
                    Atom::from_inline(string_to_add)
                } else {
                    Atom::from_dynamic(string_to_add)
                }
            }
        }
    }

    #[inline]
    fn from_inline(string: &str) -> Atom {
        assert!(string.len() < 8);
        let mut data = (Inline as u64) | (string.len() as u64 << 4);
        let ptr = &mut data as *mut u64 as *mut u8;
        unsafe { slice::raw::mut_buf_as_slice(ptr.offset(1), 7,
                                    |b| bytes::copy_memory(b, string.as_bytes())) };
        Atom {
            data: data
        }
    }

    #[inline]
    fn from_dynamic(string: &str) -> Atom {
        static mut START: Once = ONCE_INIT;

        unsafe {
            START.doit(|| {
                let cache = box StringCache::new();
                global_string_cache_ptr = mem::transmute(cache);
            });
        }

        let string_cache: &mut StringCache = unsafe { &mut *global_string_cache_ptr };
        let hash_value_address = string_cache.add(string);
        Atom {
            data: hash_value_address | Dynamic as u64
        }
    }

    #[inline]
    fn get_type(&self) -> AtomType {
        unsafe { mem::transmute((self.data & 0xf) as u8) }
    }

    #[inline]
    fn get_type_and_inline_len(&self) -> (AtomType, uint) {
        let atom_type = self.get_type();
        let len = match atom_type {
            Static | Dynamic => 0,
            Inline => unsafe { mem::transmute((self.data & 0xf0) >> 4) }
        };
        (atom_type, len)
    }

    #[inline]
    fn to_ptr_address(&self) -> u64 {
        self.data & !(Dynamic as u64)
    }
}

impl Clone for Atom {
    fn clone(&self) -> Atom {
        let atom_type = self.get_type();
        match atom_type {
            Dynamic => {
                let hash_value = unsafe { &mut *(self.to_ptr_address() as *mut StringCacheEntry) };
                hash_value.ref_count.fetch_add(1, SeqCst);
            }
            _ => {}
        }
        Atom {
            data: self.data
        }
    }
}

impl Equiv<StaticAtom> for Atom {
    fn equiv(&self, atom_id: &StaticAtom) -> bool {
        self.get_type() == Static && self.data >> INLINE_SHIFT_BITS == *atom_id as u64
    }
}

impl Drop for Atom {
    fn drop(&mut self) {
        let atom_type = self.get_type();
        match atom_type {
            Dynamic => {
                unsafe {
                    let string_cache: &mut StringCache = &mut *global_string_cache_ptr;
                    string_cache.remove(self.to_ptr_address());
                }
            },
            _ => {}
        }
    }
}

// This allows Atoms to be included in structs that are traceable for the JS GC.
impl<E, S: Encoder<E>> Encodable<S, E> for Atom {
    fn encode(&self, _: &mut S) -> Result<(), E> {
        Ok(())
    }
}

impl fmt::Show for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Atom('{:s}' type={:?})", self.as_slice(), self.get_type())
    }
}

impl Str for Atom {
    fn as_slice<'t>(&'t self) -> &'t str {
        let (atom_type, string_len) = self.get_type_and_inline_len();
        let ptr = self as *Atom as *u8;
        match atom_type {
            Inline => {
                unsafe {
                    let data = ptr.offset(1) as *[u8, ..7];
                    str::raw::from_utf8((*data).slice_to(string_len))
                }
            },
            Static => {
                let key: StaticAtom = unsafe { mem::transmute((self.data >> INLINE_SHIFT_BITS) as u32) };
                key.as_slice()
            },
            Dynamic => {
                let hash_value = unsafe { &*(self.to_ptr_address() as *StringCacheEntry) };
                hash_value.string.as_slice()
            }
        }
    }
}

impl StrAllocating for Atom {
    fn into_string(self) -> String {
        self.as_slice().to_string()
    }

    fn into_owned(self) -> String {
        self.as_slice().to_string()
    }
}

impl Ord for Atom {
    fn lt(&self, other: &Atom) -> bool {
        if self.data == other.data {
            return false;
        }
        self.as_slice() < other.as_slice()
    }
}

impl TotalOrd for Atom {
    fn cmp(&self, other: &Atom) -> Ordering {
        if self.data == other.data {
            return Equal;
        }
        self.as_slice().cmp(&other.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use std::task::spawn;
    use super::{Atom, Static, Inline, Dynamic};
    use static_atoms::{DivAtom};
    use test::Bencher;

    #[test]
    fn test_as_slice() {
        let s0 = Atom::from_slice("");
        assert!(s0.as_slice() == "");

        let s1 = Atom::from_slice("class");
        assert!(s1.as_slice() == "class");

        let i0 = Atom::from_slice("blah");
        assert!(i0.as_slice() == "blah");

        let s0 = Atom::from_slice("BLAH");
        assert!(s0.as_slice() == "BLAH");

        let d0 = Atom::from_slice("zzzzzzzzzz");
        assert!(d0.as_slice() == "zzzzzzzzzz");

        let d1 = Atom::from_slice("ZZZZZZZZZZ");
        assert!(d1.as_slice() == "ZZZZZZZZZZ");
    }

    #[test]
    fn test_types() {
        let s0 = Atom::from_slice("");
        assert!(s0.get_type_and_inline_len() == (Static, 0));

        let s1 = Atom::from_slice("id");
        assert!(s1.get_type_and_inline_len() == (Static, 0));

        let i0 = Atom::from_slice("z");
        assert!(i0.get_type_and_inline_len() == (Inline, 1));

        let i1 = Atom::from_slice("zz");
        assert!(i1.get_type_and_inline_len() == (Inline, 2));

        let i2 = Atom::from_slice("zzz");
        assert!(i2.get_type_and_inline_len() == (Inline, 3));

        let i3 = Atom::from_slice("zzzz");
        assert!(i3.get_type_and_inline_len() == (Inline, 4));

        let i4 = Atom::from_slice("zzzzz");
        assert!(i4.get_type_and_inline_len() == (Inline, 5));

        let i5 = Atom::from_slice("zzzzzz");
        assert!(i5.get_type_and_inline_len() == (Inline, 6));

        let i6 = Atom::from_slice("zzzzzzz");
        assert!(i6.get_type_and_inline_len() == (Inline, 7));

        let d0 = Atom::from_slice("zzzzzzzz");
        assert!(d0.get_type_and_inline_len() == (Dynamic, 0));

        let d1 = Atom::from_slice("zzzzzzzzzzzzz");
        assert!(d1.get_type_and_inline_len() == (Dynamic, 0));
    }

    #[test]
    fn test_equality() {
        let s0 = Atom::from_slice("fn");
        let s1 = Atom::from_slice("fn");
        let s2 = Atom::from_slice("loop");

        let i0 = Atom::from_slice("blah");
        let i1 = Atom::from_slice("blah");
        let i2 = Atom::from_slice("blah2");

        let d0 = Atom::from_slice("zzzzzzzz");
        let d1 = Atom::from_slice("zzzzzzzz");
        let d2 = Atom::from_slice("zzzzzzzzz");

        assert!(s0 == s1);
        assert!(s0 != s2);

        assert!(i0 == i1);
        assert!(i0 != i2);

        assert!(d0 == d1);
        assert!(d0 != d2);

        assert!(s0 != i0);
        assert!(s0 != d0);
        assert!(i0 != d0);
    }

    #[test]
    fn ord() {
        fn check(x: &str, y: &str) {
            assert_eq!(x < y, Atom::from_slice(x) < Atom::from_slice(y));
            assert_eq!(x.cmp(&y), Atom::from_slice(x).cmp(&Atom::from_slice(y)));
        }

        check("a", "body");
        check("asdf", "body");
        check("zasdf", "body");
        check("z", "body");

        check("a", "bbbbb");
        check("asdf", "bbbbb");
        check("zasdf", "bbbbb");
        check("z", "bbbbb");
    }

    #[test]
    fn clone() {
        let s0 = Atom::from_slice("fn");
        let s1 = s0.clone();
        let s2 = Atom::from_slice("loop");

        let i0 = Atom::from_slice("blah");
        let i1 = i0.clone();
        let i2 = Atom::from_slice("blah2");

        let d0 = Atom::from_slice("zzzzzzzz");
        let d1 = d0.clone();
        let d2 = Atom::from_slice("zzzzzzzzz");

        assert!(s0 == s1);
        assert!(s0 != s2);

        assert!(i0 == i1);
        assert!(i0 != i2);

        assert!(d0 == d1);
        assert!(d0 != d2);

        assert!(s0 != i0);
        assert!(s0 != d0);
        assert!(i0 != d0);
    }

    #[test]
    fn test_equiv() {
        let s0 = Atom::from_slice("div");
        assert!(s0.equiv(&DivAtom));

        let s1 = Atom::from_slice("Div");
        assert!(!s1.equiv(&DivAtom));
    }

    #[test]
    fn test_threads() {
        for _ in range(0, 100) {
            spawn(proc() {
                let _ = Atom::from_slice("a dynamic string");
                let _ = Atom::from_slice("another string");
            });
        }
    }

    #[bench]
    fn bench_strings(b: &mut Bencher) {
        let mut strings0 = vec!();
        let mut strings1 = vec!();

        for _ in range(0, 1000) {
            strings0.push("a");
            strings1.push("b");
        }

        let mut eq_count = 0;

        b.iter(|| {
            for (s0, s1) in strings0.iter().zip(strings1.iter()) {
                if s0 == s1 {
                    eq_count += 1;
                }
            }
        });
    }

    #[bench]
    fn bench_atoms(b: &mut Bencher) {
        let mut atoms0 = vec!();
        let mut atoms1 = vec!();

        for _ in range(0, 1000) {
            atoms0.push(Atom::from_slice("a"));
            atoms1.push(Atom::from_slice("b"));
        }

        let mut eq_count = 0;

        b.iter(|| {
            for (a0, a1) in atoms0.iter().zip(atoms1.iter()) {
                if a0 == a1 {
                    eq_count += 1;
                }
            }
        });
    }
}

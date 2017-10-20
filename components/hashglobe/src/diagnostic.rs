use hash_map::HashMap;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};
use std::ptr;

use FailedAllocationError;

#[cfg(target_pointer_width = "32")]
const CANARY: usize = 0x42cafe99;
#[cfg(target_pointer_width = "64")]
const CANARY: usize = 0x42cafe9942cafe99;

#[cfg(target_pointer_width = "32")]
const POISON: usize = 0xdeadbeef;
#[cfg(target_pointer_width = "64")]
const POISON: usize = 0xdeadbeefdeadbeef;

#[derive(Clone, Debug)]
enum JournalEntry {
    Insert(usize),
    GOIW(usize),
    Remove(usize),
    DidClear(usize),
}

#[derive(Clone, Debug)]
pub struct DiagnosticHashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    map: HashMap<K, (usize, V), S>,
    journal: Vec<JournalEntry>,
    readonly: bool,
}

impl<K: Hash + Eq, V, S: BuildHasher> DiagnosticHashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    #[inline(always)]
    pub fn inner(&self) -> &HashMap<K, (usize, V), S> {
        &self.map
    }

    #[inline(never)]
    pub fn begin_mutation(&mut self) {
        self.map.verify();
        assert!(self.readonly);
        self.readonly = false;
        self.verify();
    }

    #[inline(never)]
    pub fn end_mutation(&mut self) {
        self.map.verify();
        assert!(!self.readonly);
        self.readonly = true;
        self.verify();
    }

    fn verify(&self) {
        let mut position = 0;
        let mut count = 0;
        let mut bad_canary = None;

        let mut iter = self.map.iter();
        while let Some((h, _, v)) = iter.next_with_hash() {
            let canary_ref = &v.0;
            position += 1;

            if *canary_ref == CANARY {
                continue;
            }

            count += 1;
            bad_canary = Some((h, *canary_ref, canary_ref, position));
        }
        if let Some(c) = bad_canary {
            self.report_corruption(c.0, c.1, c.2, c.3, count, self.map.diagnostic_count_hashes());
        }
    }

    #[inline(always)]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            map: HashMap::<K, (usize, V), S>::with_hasher(hash_builder),
            journal: Vec::new(),
            readonly: true,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline(always)]
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.map.contains_key(k)
    }

    #[inline(always)]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.map.get(k).map(|v| &v.1)
    }

    #[inline(always)]
    pub fn try_get_or_insert_with<F: FnOnce() -> V>(
        &mut self,
        key: K,
        default: F
    ) -> Result<&mut V, FailedAllocationError> {
        assert!(!self.readonly);
        self.journal.push(JournalEntry::GOIW(self.map.make_hash(&key).inspect()));
        let entry = self.map.try_entry(key)?;
        Ok(&mut entry.or_insert_with(|| (CANARY, default())).1)
    }

    #[inline(always)]
    pub fn try_insert(&mut self, k: K, v: V) -> Result<Option<V>, FailedAllocationError> {
        assert!(!self.readonly);
        self.journal.push(JournalEntry::Insert(self.map.make_hash(&k).inspect()));
        let old = self.map.try_insert(k, (CANARY, v))?;
        Ok(old.map(|x| x.1))
    }

    #[inline(always)]
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        assert!(!self.readonly);
        self.journal.push(JournalEntry::Remove(self.map.make_hash(k).inspect()));
        if let Some(v) = self.map.get_mut(k) {
            unsafe { ptr::write_volatile(&mut v.0, POISON); }
        }
        self.map.remove(k).map(|x| x.1)
    }

    #[inline(always)]
    pub fn clear(&mut self) where K: 'static, V: 'static  {
        // We handle scoped mutations for the caller here, since callsites that
        // invoke clear() don't benefit from the coalescing we do around insertion.
        self.begin_mutation();
        self.journal.clear();
        self.journal.push(JournalEntry::DidClear(self.map.raw_capacity()));
        self.map.clear();
        self.end_mutation();
    }

    #[inline(never)]
    fn report_corruption(
        &self,
        hash: usize,
        canary: usize,
        canary_addr: *const usize,
        position: usize,
        count: usize,
        buffer_hash_count: usize,
    ) {
        use ::std::ffi::CString;
        let key = b"HashMapJournal\0";
        let value = CString::new(format!("{:?}", self.journal)).unwrap();
        unsafe {
            Gecko_AnnotateCrashReport(
                key.as_ptr() as *const ::std::os::raw::c_char,
                value.as_ptr(),
            );
        }

        panic!(
            concat!("HashMap Corruption (sz={}, buffer_hash_sz={}, cap={}, pairsz={}, hash={:#x}, cnry={:#x}, ",
                "count={}, last_pos={}, base_addr={:?}, cnry_addr={:?}, jrnl_len={})"),
            self.map.len(),
            buffer_hash_count,
            self.map.raw_capacity(),
            ::std::mem::size_of::<(K, (usize, V))>(),
            hash,
            canary,
            count,
            position,
            self.map.raw_buffer(),
            canary_addr,
            self.journal.len(),
        );
    }
}

impl<K, V, S> PartialEq for DiagnosticHashMap<K, V, S>
    where K: Eq + Hash,
          V: PartialEq,
          S: BuildHasher
{
    fn eq(&self, other: &Self) -> bool {
        self.map.eq(&other.map)
    }
}

impl<K, V, S> Eq for DiagnosticHashMap<K, V, S>
    where K: Eq + Hash,
          V: Eq,
          S: BuildHasher
{
}

impl<K, V, S> Default for DiagnosticHashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher + Default
{
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            journal: Vec::new(),
            readonly: true,
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> Drop for DiagnosticHashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn drop(&mut self) {
        self.map.verify();
        debug_assert!(self.readonly, "Dropped while mutating");
        self.verify();
    }
}

extern "C" {
    pub fn Gecko_AnnotateCrashReport(key_str: *const ::std::os::raw::c_char,
                                     value_str: *const ::std::os::raw::c_char);
}

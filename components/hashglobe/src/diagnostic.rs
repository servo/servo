use hash_map::HashMap;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};
use table::SafeHash;

use FailedAllocationError;

#[cfg(target_pointer_width = "32")]
const CANARY: usize = 0x42cafe99;
#[cfg(target_pointer_width = "64")]
const CANARY: usize = 0x42cafe9942cafe99;

#[derive(Clone, Debug)]
enum JournalEntry {
    Insert(SafeHash),
    GetOrInsertWith(SafeHash),
    Remove(SafeHash),
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

    #[inline(always)]
    pub fn begin_mutation(&mut self) {
        assert!(self.readonly);
        self.readonly = false;
    }

    #[inline(always)]
    pub fn end_mutation(&mut self) {
        assert!(!self.readonly);
        self.readonly = true;

        let mut position = 0;
        let mut bad_canary: Option<(usize, *const usize)> = None;
        for (_,v) in self.map.iter() {
            let canary_ref = &v.0;
            if *canary_ref == CANARY {
                position += 1;
                continue;
            }
            bad_canary = Some((*canary_ref, canary_ref));
        }
        if let Some(c) = bad_canary {
            self.report_corruption(c.0, c.1, position);
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
        self.journal.push(JournalEntry::GetOrInsertWith(self.map.make_hash(&key)));
        let entry = self.map.try_entry(key)?;
        Ok(&mut entry.or_insert_with(|| (CANARY, default())).1)
    }

    #[inline(always)]
    pub fn try_insert(&mut self, k: K, v: V) -> Result<Option<V>, FailedAllocationError> {
        assert!(!self.readonly);
        self.journal.push(JournalEntry::Insert(self.map.make_hash(&k)));
        let old = self.map.try_insert(k, (CANARY, v))?;
        Ok(old.map(|x| x.1))
    }

    #[inline(always)]
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        assert!(!self.readonly);
        self.journal.push(JournalEntry::Remove(self.map.make_hash(k)));
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
        &mut self,
        canary: usize,
        canary_addr: *const usize,
        position: usize
    ) {
        unsafe {
            Gecko_AddBufferToCrashReport(
                self.journal.as_ptr() as *const _,
                self.journal.len() * ::std::mem::size_of::<JournalEntry>(),
            );
        }
        panic!(
            "HashMap Corruption (sz={}, cap={}, pairsz={}, cnry={:#x}, pos={}, base_addr={:?}, cnry_addr={:?})",
            self.map.len(),
            self.map.raw_capacity(),
            ::std::mem::size_of::<(K, (usize, V))>(),
            canary,
            position,
            self.map.raw_buffer(),
            canary_addr,
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
        debug_assert!(self.readonly, "Dropped while mutating");
    }
}

extern "C" {
    pub fn Gecko_AddBufferToCrashReport(addr: *const ::std::os::raw::c_void,
                                        bytes: usize);
}

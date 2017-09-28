use hash_map::{Entry, HashMap, Iter, IterMut, Keys, RandomState, Values};
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};

use FailedAllocationError;

#[derive(Clone, Debug)]
pub struct ProtectedHashMap<K, V, S = RandomState>
    where K: Eq + Hash,
          S: BuildHasher
{
    map: HashMap<K, V, S>,
    readonly: bool,
}

impl<K: Hash + Eq, V, S: BuildHasher> ProtectedHashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    #[inline(always)]
    pub fn inner(&self) -> &HashMap<K, V, S> {
        &self.map
    }

    #[inline(always)]
    pub fn begin_mutation(&mut self) {
        assert!(self.readonly);
        self.unprotect();
        self.readonly = false;
    }

    #[inline(always)]
    pub fn end_mutation(&mut self) {
        assert!(!self.readonly);
        self.protect();
        self.readonly = true;
    }

    #[inline(always)]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            map: HashMap::<K, V, S>::with_hasher(hash_builder),
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
    pub fn keys(&self) -> Keys<K, V> {
        self.map.keys()
    }

    #[inline(always)]
    pub fn values(&self) -> Values<K, V> {
        self.map.values()
    }

    #[inline(always)]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.map.get(k)
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<K, V> {
        self.map.iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        assert!(!self.readonly);
        self.map.iter_mut()
    }

    #[inline(always)]
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        assert!(!self.readonly);
        self.map.entry(key)
    }

    #[inline(always)]
    pub fn try_entry(&mut self, key: K) -> Result<Entry<K, V>, FailedAllocationError> {
        assert!(!self.readonly);
        self.map.try_entry(key)
    }

    #[inline(always)]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        assert!(!self.readonly);
        self.map.insert(k, v)
    }

    #[inline(always)]
    pub fn try_insert(&mut self, k: K, v: V) -> Result<Option<V>, FailedAllocationError> {
        assert!(!self.readonly);
        self.map.try_insert(k, v)
    }

    #[inline(always)]
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        assert!(!self.readonly);
        self.map.remove(k)
    }

    #[inline(always)]
    pub fn clear(&mut self) where K: 'static, V: 'static  {
        // We handle scoped mutations for the caller here, since callsites that
        // invoke clear() don't benefit from the coalescing we do around insertion.
        self.begin_mutation();
        self.map.clear();
        self.end_mutation();
    }

    fn protect(&mut self) {
        if self.map.capacity() == 0 {
            return;
        }
        let buff = self.map.raw_buffer();
        if buff.0 as usize % ::SYSTEM_PAGE_SIZE.load(::std::sync::atomic::Ordering::Relaxed) != 0 {
            // Safely handle weird allocators like ASAN that return
            // non-page-aligned buffers to page-sized allocations.
            return;
        }
        unsafe {
            Gecko_ProtectBuffer(buff.0 as *mut _, buff.1);
        }
    }

    fn unprotect(&mut self) {
        if self.map.capacity() == 0 {
            return;
        }
        let buff = self.map.raw_buffer();
        if buff.0 as usize % ::SYSTEM_PAGE_SIZE.load(::std::sync::atomic::Ordering::Relaxed) != 0 {
            // Safely handle weird allocators like ASAN that return
            // non-page-aligned buffers to page-sized allocations.
            return;
        }
        unsafe {
            Gecko_UnprotectBuffer(buff.0 as *mut _, buff.1);
        }
    }
}

impl<K, V> ProtectedHashMap<K, V, RandomState>
    where K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            readonly: true,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut result = Self {
            map: HashMap::with_capacity(capacity),
            readonly: true,
        };
        result.protect();
        result
    }
}

impl<K, V, S> PartialEq for ProtectedHashMap<K, V, S>
    where K: Eq + Hash,
          V: PartialEq,
          S: BuildHasher
{
    fn eq(&self, other: &Self) -> bool {
        self.map.eq(&other.map)
    }
}

impl<K, V, S> Eq for ProtectedHashMap<K, V, S>
    where K: Eq + Hash,
          V: Eq,
          S: BuildHasher
{
}

impl<K, V, S> Default for ProtectedHashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher + Default
{
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            readonly: true,
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> Drop for ProtectedHashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn drop(&mut self) {
        debug_assert!(self.readonly, "Dropped while mutating");
        self.unprotect();
    }
}

// Manually declare the FFI functions since we don't depend on the crate with
// the bindings.
extern "C" {
    pub fn Gecko_ProtectBuffer(buffer: *mut ::std::os::raw::c_void,
                               size: usize);
}
extern "C" {
    pub fn Gecko_UnprotectBuffer(buffer: *mut ::std::os::raw::c_void,
                                 size: usize);
}

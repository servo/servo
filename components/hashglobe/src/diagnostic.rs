use hash_map::HashMap;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};

use FailedAllocationError;

#[derive(Clone, Debug)]
pub struct DiagnosticHashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    map: HashMap<K, V, S>,
    readonly: bool,
}

impl<K: Hash + Eq, V, S: BuildHasher> DiagnosticHashMap<K, V, S>
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
        self.readonly = false;
    }

    #[inline(always)]
    pub fn end_mutation(&mut self) {
        assert!(!self.readonly);
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
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
        where K: Borrow<Q>,
              Q: Hash + Eq
    {
        self.map.get(k)
    }

    #[inline(always)]
    pub fn try_get_or_insert_with<F: FnOnce() -> V>(
        &mut self,
        key: K,
        default: F
    ) -> Result<&mut V, FailedAllocationError> {
        assert!(!self.readonly);
        let entry = self.map.try_entry(key)?;
        Ok(entry.or_insert_with(default))
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

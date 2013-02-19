use core::cmp::*;

pub trait Cache<K: Copy Eq, V: Copy> {
    static fn new(size: uint) -> Self;
    fn insert(&self, key: &K, value: V);
    fn find(&self, key: &K) -> Option<V>;
    fn find_or_create(&self, key: &K, blk: pure fn&(&K) -> V) -> V;
    fn evict_all(&self);
}

pub struct MonoCache<K, V> {
    mut entry: Option<(K,V)>,
}

pub impl<K: Copy Eq, V: Copy> MonoCache<K,V> : Cache<K,V> {
    static fn new(_size: uint) -> MonoCache<K,V> {
        MonoCache { entry: None }
    }

    fn insert(&self, key: &K, value: V) {
        self.entry = Some((copy *key, value));
    }

    fn find(&self, key: &K) -> Option<V> {
        match self.entry {
            None => None,
            Some((ref k,v)) => if *k == *key { Some(v) } else { None }
        }
    }

    fn find_or_create(&self, key: &K, blk: pure fn&(&K) -> V) -> V {
        return match self.find(key) {
            None => { 
                let value = blk(key);
                self.entry = Some((copy *key, copy value));
                move value
            },
            Some(v) => v
        };
    }
    fn evict_all(&self) {
        self.entry = None;
    }
}

#[test]
fn test_monocache() {
    // TODO: this is hideous because of Rust Issue #3902
    let cache = cache::new::<uint, @str, MonoCache<uint, @str>>(10);
    let one = @"one";
    let two = @"two";
    cache.insert(&1, one);

    assert cache.find(&1).is_some();
    assert cache.find(&2).is_none();
    cache.find_or_create(&2, |_v| { two });
    assert cache.find(&2).is_some();
    assert cache.find(&1).is_none();
}

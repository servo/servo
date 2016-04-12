import py
from py._path import cacheutil

class BasicCacheAPITest:
    cache = None
    def test_getorbuild(self):
        val = self.cache.getorbuild(-42, lambda: 42)
        assert val == 42
        val = self.cache.getorbuild(-42, lambda: 23)
        assert val == 42

    def test_cache_get_key_error(self):
        py.test.raises(KeyError, "self.cache._getentry(-23)")

    def test_delentry_non_raising(self):
        val = self.cache.getorbuild(100, lambda: 100)
        self.cache.delentry(100)
        py.test.raises(KeyError, "self.cache._getentry(100)")

    def test_delentry_raising(self):
        val = self.cache.getorbuild(100, lambda: 100)
        self.cache.delentry(100)
        py.test.raises(KeyError, "self.cache.delentry(100, raising=True)")

    def test_clear(self):
        self.cache.clear()

class TestBuildcostAccess(BasicCacheAPITest):
    cache = cacheutil.BuildcostAccessCache(maxentries=128)

    def test_cache_works_somewhat_simple(self, monkeypatch):
        cache = cacheutil.BuildcostAccessCache()
        # the default gettime
        # BuildcostAccessCache.build can
        # result into time()-time() == 0 which makes the below
        # test fail randomly.  Let's rather use incrementing
        # numbers instead.
        l = [0]
        def counter():
            l[0] = l[0] + 1
            return l[0]
        monkeypatch.setattr(cacheutil, 'gettime', counter)
        for x in range(cache.maxentries):
            y = cache.getorbuild(x, lambda: x)
            assert x == y
        for x in range(cache.maxentries):
            assert cache.getorbuild(x, None) == x
        halfentries = int(cache.maxentries / 2)
        for x in range(halfentries):
            assert cache.getorbuild(x, None) == x
            assert cache.getorbuild(x, None) == x
        # evict one entry
        val = cache.getorbuild(-1, lambda: 42)
        assert val == 42
        # check that recently used ones are still there
        # and are not build again
        for x in range(halfentries):
            assert cache.getorbuild(x, None) == x
        assert cache.getorbuild(-1, None) == 42


class TestAging(BasicCacheAPITest):
    maxsecs = 0.10
    cache = cacheutil.AgingCache(maxentries=128, maxseconds=maxsecs)

    def test_cache_eviction(self):
        self.cache.getorbuild(17, lambda: 17)
        endtime = py.std.time.time() + self.maxsecs * 10
        while py.std.time.time() < endtime:
            try:
                self.cache._getentry(17)
            except KeyError:
                break
            py.std.time.sleep(self.maxsecs*0.3)
        else:
            py.test.fail("waiting for cache eviction failed")

def test_prune_lowestweight():
    maxsecs = 0.05
    cache = cacheutil.AgingCache(maxentries=10, maxseconds=maxsecs)
    for x in range(cache.maxentries):
        cache.getorbuild(x, lambda: x)
    py.std.time.sleep(maxsecs*1.1)
    cache.getorbuild(cache.maxentries+1, lambda: 42)

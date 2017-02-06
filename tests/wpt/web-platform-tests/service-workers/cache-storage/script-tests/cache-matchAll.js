if (self.importScripts) {
    importScripts('/resources/testharness.js');
    importScripts('../resources/test-helpers.js');
}

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll('not-present-in-the-cache')
      .then(function(result) {
          assert_response_array_equivalent(
            result, [],
            'Cache.matchAll should resolve with an empty array on failure.');
        });
  }, 'Cache.matchAll with no matching entries');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.a.request.url)
      .then(function(result) {
          assert_response_array_equals(result, [entries.a.response],
                                       'Cache.matchAll should match by URL.');
        });
  }, 'Cache.matchAll with URL');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.a.request)
      .then(function(result) {
          assert_response_array_equals(
            result, [entries.a.response],
            'Cache.matchAll should match by Request.');
        });
  }, 'Cache.matchAll with Request');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(new Request(entries.a.request.url))
      .then(function(result) {
          assert_response_array_equals(
            result, [entries.a.response],
            'Cache.matchAll should match by Request.');
        });
  }, 'Cache.matchAll with new Request');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(new Request(entries.a.request.url, {method: 'HEAD'}),
                          {ignoreSearch: true})
      .then(function(result) {
          assert_response_array_equals(
            result, [],
            'Cache.matchAll should not match HEAD Request.');
        });
  }, 'Cache.matchAll with HEAD');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.a.request,
                          {ignoreSearch: true})
      .then(function(result) {
          assert_response_array_equivalent(
            result,
            [
              entries.a.response,
              entries.a_with_query.response
            ],
            'Cache.matchAll with ignoreSearch should ignore the ' +
            'search parameters of cached request.');
        });
  },
  'Cache.matchAll with ignoreSearch option (request with no search ' +
  'parameters)');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.a_with_query.request,
                          {ignoreSearch: true})
      .then(function(result) {
          assert_response_array_equivalent(
            result,
            [
              entries.a.response,
              entries.a_with_query.response
            ],
            'Cache.matchAll with ignoreSearch should ignore the ' +
            'search parameters of request.');
        });
  },
  'Cache.matchAll with ignoreSearch option (request with search parameter)');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.cat.request.url + '#mouse')
      .then(function(result) {
          assert_response_array_equivalent(
            result,
            [
              entries.cat.response,
            ],
            'Cache.matchAll should ignore URL fragment.');
        });
  }, 'Cache.matchAll with URL containing fragment');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll('http')
      .then(function(result) {
          assert_response_array_equivalent(
            result, [],
            'Cache.matchAll should treat query as a URL and not ' +
            'just a string fragment.');
        });
  }, 'Cache.matchAll with string fragment "http" as query');

prepopulated_cache_test(vary_entries, function(cache, entries) {
    return cache.matchAll('http://example.com/c')
      .then(function(result) {
          assert_response_array_equivalent(
            result,
            [
              entries.vary_cookie_absent.response
            ],
            'Cache.matchAll should exclude matches if a vary header is ' +
            'missing in the query request, but is present in the cached ' +
            'request.');
        })

      .then(function() {
          return cache.matchAll(
            new Request('http://example.com/c',
                        {headers: {'Cookies': 'none-of-the-above'}}));
        })
      .then(function(result) {
          assert_response_array_equivalent(
            result,
            [
            ],
            'Cache.matchAll should exclude matches if a vary header is ' +
            'missing in the cached request, but is present in the query ' +
            'request.');
        })

      .then(function() {
          return cache.matchAll(
            new Request('http://example.com/c',
                        {headers: {'Cookies': 'is-for-cookie'}}));
        })
      .then(function(result) {
          assert_response_array_equivalent(
            result,
            [entries.vary_cookie_is_cookie.response],
            'Cache.matchAll should match the entire header if a vary header ' +
            'is present in both the query and cached requests.');
        });
  }, 'Cache.matchAll with responses containing "Vary" header');

prepopulated_cache_test(vary_entries, function(cache, entries) {
    return cache.matchAll('http://example.com/c',
                          {ignoreVary: true})
      .then(function(result) {
          assert_response_array_equivalent(
            result,
            [
              entries.vary_cookie_is_cookie.response,
              entries.vary_cookie_is_good.response,
              entries.vary_cookie_absent.response
            ],
            'Cache.matchAll should honor "ignoreVary" parameter.');
        });
  }, 'Cache.matchAll with "ignoreVary" parameter');

done();

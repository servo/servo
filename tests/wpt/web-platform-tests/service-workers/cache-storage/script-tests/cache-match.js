if (self.importScripts) {
    importScripts('/resources/testharness.js');
    importScripts('../resources/testharness-helpers.js');
    importScripts('../resources/test-helpers.js');
}

// A set of Request/Response pairs to be used with prepopulated_cache_test().
var simple_entries = [
  {
    name: 'a',
    request: new Request('http://example.com/a'),
    response: new Response('')
  },

  {
    name: 'b',
    request: new Request('http://example.com/b'),
    response: new Response('')
  },

  {
    name: 'a_with_query',
    request: new Request('http://example.com/a?q=r'),
    response: new Response('')
  },

  {
    name: 'A',
    request: new Request('http://example.com/A'),
    response: new Response('')
  },

  {
    name: 'a_https',
    request: new Request('https://example.com/a'),
    response: new Response('')
  },

  {
    name: 'a_org',
    request: new Request('http://example.org/a'),
    response: new Response('')
  },

  {
    name: 'cat',
    request: new Request('http://example.com/cat'),
    response: new Response('')
  },

  {
    name: 'catmandu',
    request: new Request('http://example.com/catmandu'),
    response: new Response('')
  },

  {
    name: 'cat_num_lives',
    request: new Request('http://example.com/cat?lives=9'),
    response: new Response('')
  },

  {
    name: 'cat_in_the_hat',
    request: new Request('http://example.com/cat/in/the/hat'),
    response: new Response('')
  },

  {
    name: 'secret_cat',
    request: new Request('http://tom:jerry@example.com/cat'),
    response: new Response('')
  },

  {
    name: 'top_secret_cat',
    request: new Request('http://tom:j3rry@example.com/cat'),
    response: new Response('')
  }
];

// A set of Request/Response pairs to be used with prepopulated_cache_test().
// These contain a mix of test cases that use Vary headers.
var vary_entries = [
  {
    name: 'vary_cookie_is_cookie',
    request: new Request('http://example.com/c',
                         {headers: {'Cookies': 'is-for-cookie'}}),
    response: new Response('',
                           {headers: {'Vary': 'Cookies'}})
  },

  {
    name: 'vary_cookie_is_good',
    request: new Request('http://example.com/c',
                         {headers: {'Cookies': 'is-good-enough-for-me'}}),
    response: new Response('',
                           {headers: {'Vary': 'Cookies'}})
  },

  {
    name: 'vary_cookie_absent',
    request: new Request('http://example.com/c'),
    response: new Response('',
                           {headers: {'Vary': 'Cookies'}})
  }
];

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll('not-present-in-the-cache')
      .then(function(result) {
          assert_array_equivalent(
            result, [],
            'Cache.matchAll should resolve with an empty array on failure.');
        });
  }, 'Cache.matchAll with no matching entries');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.match('not-present-in-the-cache')
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.match failures should resolve with undefined.');
        });
  }, 'Cache.match with no matching entries');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.a.request.url)
      .then(function(result) {
          assert_array_objects_equals(result, [entries.a.response],
                                      'Cache.matchAll should match by URL.');
        });
  }, 'Cache.matchAll with URL');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.match(entries.a.request.url)
      .then(function(result) {
          assert_object_equals(result, entries.a.response,
                               'Cache.match should match by URL.');
        });
  }, 'Cache.match with URL');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.a.request)
      .then(function(result) {
          assert_array_objects_equals(
            result, [entries.a.response],
            'Cache.matchAll should match by Request.');
        });
  }, 'Cache.matchAll with Request');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.match(entries.a.request)
      .then(function(result) {
          assert_object_equals(result, entries.a.response,
                               'Cache.match should match by Request.');
        });
  }, 'Cache.match with Request');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(new Request(entries.a.request.url))
      .then(function(result) {
          assert_array_objects_equals(
            result, [entries.a.response],
            'Cache.matchAll should match by Request.');
        });
  }, 'Cache.matchAll with new Request');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.match(new Request(entries.a.request.url))
      .then(function(result) {
          assert_object_equals(result, entries.a.response,
                               'Cache.match should match by Request.');
        });
  }, 'Cache.match with new Request');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.a.request,
                          {ignoreSearch: true})
      .then(function(result) {
          assert_array_equivalent(
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
    return cache.match(entries.a.request,
                       {ignoreSearch: true})
      .then(function(result) {
          assert_object_in_array(
            result,
            [
              entries.a.response,
              entries.a_with_query.response
            ],
            'Cache.match with ignoreSearch should ignore the ' +
            'search parameters of cached request.');
        });
  },
  'Cache.match with ignoreSearch option (request with no search ' +
  'parameters)');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.a_with_query.request,
                          {ignoreSearch: true})
      .then(function(result) {
          assert_array_equivalent(
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
    return cache.match(entries.a_with_query.request,
                       {ignoreSearch: true})
      .then(function(result) {
          assert_object_in_array(
            result,
            [
              entries.a.response,
              entries.a_with_query.response
            ],
            'Cache.match with ignoreSearch should ignore the ' +
            'search parameters of request.');
        });
  },
  'Cache.match with ignoreSearch option (request with search parameter)');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.cat.request.url + '#mouse')
      .then(function(result) {
          assert_array_equivalent(
            result,
            [
              entries.cat.response,
            ],
            'Cache.matchAll should ignore URL fragment.');
        });
  }, 'Cache.matchAll with URL containing fragment');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.match(entries.cat.request.url + '#mouse')
      .then(function(result) {
          assert_object_equals(result, entries.cat.response,
                               'Cache.match should ignore URL fragment.');
        });
  }, 'Cache.match with URL containing fragment');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll('http')
      .then(function(result) {
          assert_array_equivalent(
            result, [],
            'Cache.matchAll should treat query as a URL and not ' +
            'just a string fragment.');
        });
  }, 'Cache.matchAll with string fragment "http" as query');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.match('http')
      .then(function(result) {
          assert_equals(
            result, undefined,
            'Cache.match should treat query as a URL and not ' +
            'just a string fragment.');
        });
  }, 'Cache.match with string fragment "http" as query');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.matchAll(entries.secret_cat.request.url)
      .then(function(result) {
          assert_array_equivalent(
            result, [entries.secret_cat.response],
            'Cache.matchAll should not ignore embedded credentials');
        });
  }, 'Cache.matchAll with URL containing credentials');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    return cache.match(entries.secret_cat.request.url)
      .then(function(result) {
          assert_object_equals(
            result, entries.secret_cat.response,
            'Cache.match should not ignore embedded credentials');
        });
  }, 'Cache.match with URL containing credentials');

prepopulated_cache_test(vary_entries, function(cache, entries) {
    return cache.matchAll('http://example.com/c')
      .then(function(result) {
          assert_array_equivalent(
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
          assert_array_equivalent(
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
          assert_array_equivalent(
            result,
            [entries.vary_cookie_is_cookie.response],
            'Cache.matchAll should match the entire header if a vary header ' +
            'is present in both the query and cached requests.');
        });
  }, 'Cache.matchAll with responses containing "Vary" header');

prepopulated_cache_test(vary_entries, function(cache, entries) {
    return cache.match('http://example.com/c')
      .then(function(result) {
          assert_object_in_array(
            result,
            [
              entries.vary_cookie_absent.response
            ],
            'Cache.match should honor "Vary" header.');
        });
  }, 'Cache.match with responses containing "Vary" header');

prepopulated_cache_test(vary_entries, function(cache, entries) {
    return cache.matchAll('http://example.com/c',
                          {ignoreVary: true})
      .then(function(result) {
          assert_array_equivalent(
            result,
            [
              entries.vary_cookie_is_cookie.response,
              entries.vary_cookie_is_good.response,
              entries.vary_cookie_absent.response,
            ],
            'Cache.matchAll should honor "ignoreVary" parameter.');
        });
  }, 'Cache.matchAll with "ignoreVary" parameter');

cache_test(function(cache) {
    var request = new Request('http://example.com');
    var response;
    var request_url = new URL('../resources/simple.txt', location.href).href;
    return fetch(request_url)
      .then(function(fetch_result) {
          response = fetch_result;
          assert_equals(
            response.url, request_url,
            '[https://fetch.spec.whatwg.org/#dom-response-url] ' +
            'Reponse.url should return the URL of the response.');
          return cache.put(request, response.clone());
        })
      .then(function() {
          return cache.match(request.url);
        })
      .then(function(result) {
          assert_object_equals(
            result, response,
            'Cache.match should return a Response object that has the same ' +
            'properties as the stored response.');
          return cache.match(response.url);
        })
      .then(function(result) {
          assert_equals(
            result, undefined,
            'Cache.match should not match cache entry based on response URL.');
        });
  }, 'Cache.match with Request and Response objects with different URLs');

cache_test(function(cache) {
    var request_url = new URL('../resources/simple.txt', location.href).href;
    return fetch(request_url)
      .then(function(fetch_result) {
          return cache.put(new Request(request_url), fetch_result);
        })
      .then(function() {
          return cache.match(request_url);
        })
      .then(function(result) {
          return result.text();
        })
      .then(function(body_text) {
          assert_equals(body_text, 'a simple text file\n',
                        'Cache.match should return a Response object with a ' +
                        'valid body.');
        })
      .then(function() {
          return cache.match(request_url);
        })
      .then(function(result) {
          return result.text();
        })
      .then(function(body_text) {
          assert_equals(body_text, 'a simple text file\n',
                        'Cache.match should return a Response object with a ' +
                        'valid body each time it is called.');
        });
  }, 'Cache.match invoked multiple times for the same Request/Response');

prepopulated_cache_test(simple_entries, function(cache, entries) {
    var request = new Request(entries.a.request, { method: 'POST' });
    return cache.match(request)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.match should not find a match');
        });
  }, 'Cache.match with POST Request');

// Helpers ---

// Run |test_function| with a Cache object as its only parameter. Prior to the
// call, the Cache is populated by cache entries from |entries|. The latter is
// expected to be an Object mapping arbitrary keys to objects of the form
// {request: <Request object>, response: <Response object>}. There's no
// guarantee on the order in which entries will be added to the cache.
//
// |test_function| should return a Promise that can be used with promise_test.
function prepopulated_cache_test(entries, test_function, description) {
  cache_test(function(cache) {
      var p = Promise.resolve();
      var hash = {};
      entries.forEach(function(entry) {
          p = p.then(function() {
              return cache.put(entry.request.clone(),
                               entry.response.clone())
                .catch(function(e) {
                    assert_unreached('Test setup failed for entry ' +
                                     entry.name + ': ' + e);
                  });
            });
          hash[entry.name] = entry;
        });
      p = p.then(function() {
          assert_equals(Object.keys(hash).length, entries.length);
        });

      return p.then(function() {
          return test_function(cache, hash);
        });
    }, description);
}

done();

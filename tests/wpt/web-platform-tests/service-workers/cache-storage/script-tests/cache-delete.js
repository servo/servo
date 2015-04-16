if (self.importScripts) {
    importScripts('/resources/testharness.js');
    importScripts('../resources/testharness-helpers.js');
    importScripts('../resources/test-helpers.js');
}

var test_url = 'https://example.com/foo';

// Construct a generic Request object. The URL is |test_url|. All other fields
// are defaults.
function new_test_request() {
  return new Request(test_url);
}

// Construct a generic Response object.
function new_test_response() {
  return new Response('Hello world!', { status: 200 });
}

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.delete(),
      new TypeError(),
      'Cache.delete should reject with a TypeError when called with no ' +
      'arguments.');
  }, 'Cache.delete with no arguments');

cache_test(function(cache) {
    return cache.put(new_test_request(), new_test_response())
      .then(function() {
          return cache.delete(test_url);
        })
      .then(function(result) {
          assert_true(result,
                      'Cache.delete should resolve with "true" if an entry ' +
                      'was successfully deleted.');
          return cache.match(test_url);
        })
      .then(function(result) {
          assert_equals(result, undefined,
            'Cache.delete should remove matching entries from cache.');
        });
  }, 'Cache.delete called with a string URL');

cache_test(function(cache) {
    var request = new Request(test_url, { method: 'POST', body: 'Abc' });
    return cache.put(request.clone(), new_test_response())
      .then(function() {
          return cache.delete(request);
        })
      .then(function(result) {
          assert_true(result,
                      'Cache.delete should resolve with "true" if an entry ' +
                      'was successfully deleted.');
          assert_false(request.bodyUsed,
                       'Cache.delete should not consume request body.');
        });
  }, 'Cache.delete called with a Request object');

cache_test(function(cache) {
    var request = new Request(test_url, { method: 'POST', body: 'Abc' });
    return cache.put(request.clone(), new_test_response())
      .then(function() {
          return request.text();
        })
      .then(function() {
          assert_true(request.bodyUsed,
                      '[https://fetch.spec.whatwg.org/#body-mixin] ' +
                      'Request.bodyUsed should be true after text() method ' +
                      'resolves.');
        })
      .then(function() {
          return cache.delete(request);
        })
      .then(function(result) {
          assert_true(result,
                      'Cache.delete should resolve with "true" if an entry ' +
                      'was successfully deleted.');
        });
  }, 'Cache.delete with a Request object containing used body');

cache_test(function(cache) {
    return cache.delete(test_url)
      .then(function(result) {
          assert_false(result,
                       'Cache.delete should resolve with "false" if there ' +
                       'are no matching entries.');
        });
  }, 'Cache.delete with a non-existent entry');

var cache_entries = {
  a: {
    request: new Request('http://example.com/abc'),
    response: new Response('')
  },

  b: {
    request: new Request('http://example.com/b'),
    response: new Response('')
  },

  a_with_query: {
    request: new Request('http://example.com/abc?q=r'),
    response: new Response('')
  }
};

function prepopulated_cache_test(test_function, description) {
  cache_test(function(cache) {
      return Promise.all(Object.keys(cache_entries).map(function(k) {
          return cache.put(cache_entries[k].request.clone(),
                           cache_entries[k].response.clone());
        }))
        .then(function() {
            return test_function(cache);
          });
    }, description);
}

done();

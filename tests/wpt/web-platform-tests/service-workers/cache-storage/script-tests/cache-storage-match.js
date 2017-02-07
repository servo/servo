if (self.importScripts) {
    importScripts('/resources/testharness.js');
    importScripts('../resources/test-helpers.js');
}

(function() {
  var next_index = 1;

  // Returns a transaction (request, response, and url) for a unique URL.
  function create_unique_transaction(test) {
    var uniquifier = String(next_index++);
    var url = 'http://example.com/' + uniquifier;

    return {
      request: new Request(url),
      response: new Response('hello'),
      url: url
    };
  }

  self.create_unique_transaction = create_unique_transaction;
})();

cache_test(function(cache) {
    var transaction = create_unique_transaction();

    return cache.put(transaction.request.clone(), transaction.response.clone())
      .then(function() {
          return self.caches.match(transaction.request);
        })
      .then(function(response) {
          assert_response_equals(response, transaction.response,
                                 'The response should not have changed.');
        });
}, 'CacheStorageMatch with no cache name provided');

cache_test(function(cache) {
    var transaction = create_unique_transaction();

    var test_cache_list = ['a', 'b', 'c'];
    return cache.put(transaction.request.clone(), transaction.response.clone())
      .then(function() {
          return Promise.all(test_cache_list.map(function(key) {
              return self.caches.open(key);
            }));
        })
      .then(function() {
          return self.caches.match(transaction.request);
        })
      .then(function(response) {
          assert_response_equals(response, transaction.response,
                                 'The response should not have changed.');
        });
}, 'CacheStorageMatch from one of many caches');

promise_test(function(test) {
    var transaction = create_unique_transaction();

    var test_cache_list = ['x', 'y', 'z'];
    return Promise.all(test_cache_list.map(function(key) {
        return self.caches.open(key);
      }))
      .then(function() { return self.caches.open('x'); })
      .then(function(cache) {
          return cache.put(transaction.request.clone(),
                           transaction.response.clone());
        })
      .then(function() {
          return self.caches.match(transaction.request, {cacheName: 'x'});
        })
      .then(function(response) {
          assert_response_equals(response, transaction.response,
                                 'The response should not have changed.');
        })
      .then(function() {
          return self.caches.match(transaction.request, {cacheName: 'y'});
        })
      .then(function(response) {
          assert_equals(response, undefined,
                        'Cache y should not have a response for the request.');
        });
}, 'CacheStorageMatch from one of many caches by name');

cache_test(function(cache) {
    var transaction = create_unique_transaction();
    return cache.put(transaction.url, transaction.response.clone())
      .then(function() {
          return self.caches.match(transaction.request);
        })
      .then(function(response) {
          assert_response_equals(response, transaction.response,
                                 'The response should not have changed.');
        });
}, 'CacheStorageMatch a string request');

cache_test(function(cache) {
    var transaction = create_unique_transaction();
    return cache.put(transaction.request.clone(), transaction.response.clone())
      .then(function() {
          return self.caches.match(new Request(transaction.request.url,
                                              {method: 'HEAD'}));
        })
      .then(function(response) {
          assert_equals(response, undefined,
                        'A HEAD request should not be matched');
        });
}, 'CacheStorageMatch a HEAD request');

promise_test(function(test) {
    var transaction = create_unique_transaction();
    return self.caches.match(transaction.request)
      .then(function(response) {
          assert_equals(response, undefined,
                        'The response should not be found.');
        });
}, 'CacheStorageMatch with no cached entry');

promise_test(function(test) {
    var transaction = create_unique_transaction();
    return self.caches.has('foo')
      .then(function(has_foo) {
          assert_false(has_foo, "The cache should not exist.");
          return self.caches.match(transaction.request, {cacheName: 'foo'});
        })
      .then(function(response) {
          assert_equals(response, undefined,
                        'The match with bad cache name should resolve to ' +
                        'undefined.');
          return self.caches.has('foo');
        })
      .then(function(has_foo) {
          assert_false(has_foo, "The cache should still not exist.");
        });
}, 'CacheStorageMatch with no caches available but name provided');

done();

if (self.importScripts) {
    importScripts('/resources/testharness.js');
    importScripts('../resources/testharness-helpers.js');
    importScripts('../resources/test-helpers.js');
}

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.add(),
      new TypeError(),
      'Cache.add should throw a TypeError when no arguments are given.');
  }, 'Cache.add called with no arguments');

cache_test(function(cache) {
    return cache.add('../resources/simple.txt')
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.add should resolve with undefined on success.');
        });
  }, 'Cache.add called with relative URL specified as a string');

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.add('javascript://this-is-not-http-mmkay'),
      new TypeError(),
      'Cache.add should throw a TypeError for non-HTTP/HTTPS URLs.');
  }, 'Cache.add called with non-HTTP/HTTPS URL');

cache_test(function(cache) {
    var request = new Request('../resources/simple.txt');
    return cache.add(request)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.add should resolve with undefined on success.');
        });
  }, 'Cache.add called with Request object');

cache_test(function(cache) {
    var request = new Request('../resources/simple.txt');
    return cache.add(request)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.add should resolve with undefined on success.');
        })
      .then(function() {
          return cache.add(request);
        })
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.add should resolve with undefined on success.');
        });
  }, 'Cache.add called twice with the same Request object');

cache_test(function(cache) {
    return cache.add('this-does-not-exist-please-dont-create-it')
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.add should resolve with undefined on success.');
        });
  }, 'Cache.add with request that results in a status of 404');

cache_test(function(cache) {
    return cache.add('../resources/fetch-status.py?status=500')
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.add should resolve with undefined on success.');
        });
  }, 'Cache.add with request that results in a status of 500');

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.addAll(),
      new TypeError(),
      'Cache.addAll with no arguments should throw TypeError.');
  }, 'Cache.addAll with no arguments');

cache_test(function(cache) {
    // Assumes the existence of ../resources/simple.txt and ../resources/blank.html
    var urls = ['../resources/simple.txt', undefined, '../resources/blank.html'];
    return assert_promise_rejects(
      cache.addAll(),
      new TypeError(),
      'Cache.addAll should throw TypeError for an undefined argument.');
  }, 'Cache.addAll with a mix of valid and undefined arguments');

cache_test(function(cache) {
    // Assumes the existence of ../resources/simple.txt and ../resources/blank.html
    var urls = ['../resources/simple.txt', self.location.href, '../resources/blank.html'];
    return cache.addAll(urls)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.addAll should resolve with undefined on ' +
                        'success.');
        });
  }, 'Cache.addAll with string URL arguments');

cache_test(function(cache) {
    // Assumes the existence of ../resources/simple.txt and ../resources/blank.html
    var urls = ['../resources/simple.txt', self.location.href, '../resources/blank.html'];
    var requests = urls.map(function(url) {
        return new Request(url);
      });
    return cache.addAll(requests)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.addAll should resolve with undefined on ' +
                        'success.');
        });
  }, 'Cache.addAll with Request arguments');

cache_test(function(cache) {
    // Assumes that ../resources/simple.txt and ../resources/blank.html exist. The second
    // resource does not.
    var urls = ['../resources/simple.txt', 'this-resource-should-not-exist', '../resources/blank.html'];
    var requests = urls.map(function(url) {
        return new Request(url);
      });
    return cache.addAll(requests)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.addAll should resolve with undefined on ' +
                        'success.');
        });
  }, 'Cache.addAll with a mix of succeeding and failing requests');

cache_test(function(cache) {
    var request = new Request('../resources/simple.txt');
    return assert_promise_rejects(
      cache.addAll([request, request]),
      'InvalidStateError',
      'Cache.addAll should throw InvalidStateError if the same request is added ' +
      'twice.');
  }, 'Cache.addAll called with the same Request object specified twice');

done();

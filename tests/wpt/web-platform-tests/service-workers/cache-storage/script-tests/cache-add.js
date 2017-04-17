if (self.importScripts) {
    importScripts('/resources/testharness.js');
    importScripts('../resources/test-helpers.js');
}

cache_test(function(cache, test) {
    return promise_rejects(
      test,
      new TypeError(),
      cache.add(),
      'Cache.add should throw a TypeError when no arguments are given.');
  }, 'Cache.add called with no arguments');

cache_test(function(cache) {
    return cache.add('../resources/simple.txt')
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.add should resolve with undefined on success.');
          return cache.match('../resources/simple.txt');
        })
        .then(function(response) {
          assert_class_string(response, 'Response',
                              'Cache.add should put a resource in the cache.');
          return response.text();
        })
        .then(function(body) {
          assert_equals(body, 'a simple text file\n',
                        'Cache.add should retrieve the correct body.');
        });
  }, 'Cache.add called with relative URL specified as a string');

cache_test(function(cache, test) {
    return promise_rejects(
      test,
      new TypeError(),
      cache.add('javascript://this-is-not-http-mmkay'),
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

cache_test(function(cache, test) {
    var request = new Request('../resources/simple.txt',
                              {method: 'POST', body: 'This is a body.'});
    return promise_rejects(
      test,
      new TypeError(),
      cache.add(request),
      'Cache.add should throw a TypeError for non-GET requests.');
  }, 'Cache.add called with POST request');

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
    var request = new Request('../resources/simple.txt');
    return request.text()
      .then(function() {
          assert_false(request.bodyUsed);
        })
      .then(function() {
          return cache.add(request);
        });
  }, 'Cache.add with request with null body (not consumed)');

cache_test(function(cache, test) {
    return promise_rejects(
      test,
      new TypeError(),
      cache.add('../resources/fetch-status.py?status=206'),
      'Cache.add should reject on partial response');
  }, 'Cache.add with 206 response');

cache_test(function(cache, test) {
    var urls = ['../resources/fetch-status.py?status=206',
                '../resources/fetch-status.py?status=200'];
    var requests = urls.map(function(url) {
        return new Request(url);
      });
    return promise_rejects(
      test,
      new TypeError(),
      cache.addAll(requests),
      'Cache.addAll should reject with TypeError if any request fails');
  }, 'Cache.addAll with 206 response');

cache_test(function(cache, test) {
    return promise_rejects(
      test,
      new TypeError(),
      cache.add('this-does-not-exist-please-dont-create-it'),
      'Cache.add should reject if response is !ok');
  }, 'Cache.add with request that results in a status of 404');


cache_test(function(cache, test) {
    return promise_rejects(
      test,
      new TypeError(),
      cache.add('../resources/fetch-status.py?status=500'),
      'Cache.add should reject if response is !ok');
  }, 'Cache.add with request that results in a status of 500');

cache_test(function(cache, test) {
    return promise_rejects(
      test,
      new TypeError(),
      cache.addAll(),
      'Cache.addAll with no arguments should throw TypeError.');
  }, 'Cache.addAll with no arguments');

cache_test(function(cache, test) {
    // Assumes the existence of ../resources/simple.txt and ../resources/blank.html
    var urls = ['../resources/simple.txt', undefined, '../resources/blank.html'];
    return promise_rejects(
      test,
      new TypeError(),
      cache.addAll(),
      'Cache.addAll should throw TypeError for an undefined argument.');
  }, 'Cache.addAll with a mix of valid and undefined arguments');

cache_test(function(cache) {
    return cache.addAll([])
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.addAll should resolve with undefined on ' +
                        'success.');
          return cache.keys();
        })
      .then(function(result) {
          assert_equals(result.length, 0,
                        'There should be no entry in the cache.');
        });
  }, 'Cache.addAll with an empty array');

cache_test(function(cache) {
    // Assumes the existence of ../resources/simple.txt and
    // ../resources/blank.html
    var urls = ['../resources/simple.txt',
                self.location.href,
                '../resources/blank.html'];
    return cache.addAll(urls)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.addAll should resolve with undefined on ' +
                        'success.');
          return Promise.all(
            urls.map(function(url) { return cache.match(url); }));
        })
      .then(function(responses) {
          assert_class_string(
            responses[0], 'Response',
            'Cache.addAll should put a resource in the cache.');
          assert_class_string(
            responses[1], 'Response',
            'Cache.addAll should put a resource in the cache.');
          assert_class_string(
            responses[2], 'Response',
            'Cache.addAll should put a resource in the cache.');
          return Promise.all(
            responses.map(function(response) { return response.text(); }));
        })
      .then(function(bodies) {
          assert_equals(
            bodies[0], 'a simple text file\n',
            'Cache.add should retrieve the correct body.');
          assert_equals(
            bodies[2], '<!DOCTYPE html>\n<title>Empty doc</title>\n',
            'Cache.add should retrieve the correct body.');
        });
  }, 'Cache.addAll with string URL arguments');

cache_test(function(cache) {
    // Assumes the existence of ../resources/simple.txt and
    // ../resources/blank.html
    var urls = ['../resources/simple.txt',
                self.location.href,
                '../resources/blank.html'];
    var requests = urls.map(function(url) {
        return new Request(url);
      });
    return cache.addAll(requests)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.addAll should resolve with undefined on ' +
                        'success.');
          return Promise.all(
            urls.map(function(url) { return cache.match(url); }));
        })
      .then(function(responses) {
          assert_class_string(
            responses[0], 'Response',
            'Cache.addAll should put a resource in the cache.');
          assert_class_string(
            responses[1], 'Response',
            'Cache.addAll should put a resource in the cache.');
          assert_class_string(
            responses[2], 'Response',
            'Cache.addAll should put a resource in the cache.');
          return Promise.all(
            responses.map(function(response) { return response.text(); }));
        })
      .then(function(bodies) {
          assert_equals(
            bodies[0], 'a simple text file\n',
            'Cache.add should retrieve the correct body.');
          assert_equals(
            bodies[2], '<!DOCTYPE html>\n<title>Empty doc</title>\n',
            'Cache.add should retrieve the correct body.');
        });
  }, 'Cache.addAll with Request arguments');

cache_test(function(cache, test) {
    // Assumes that ../resources/simple.txt and ../resources/blank.html exist.
    // The second resource does not.
    var urls = ['../resources/simple.txt',
                'this-resource-should-not-exist',
                '../resources/blank.html'];
    var requests = urls.map(function(url) {
        return new Request(url);
      });
    return promise_rejects(
      test,
      new TypeError(),
      cache.addAll(requests),
      'Cache.addAll should reject with TypeError if any request fails')
      .then(function() {
          return Promise.all(urls.map(function(url) {
              return cache.match(url);
            }));
      })
      .then(function(matches) {
          assert_array_equals(
            matches,
            [undefined, undefined, undefined],
            'If any response fails, no response should be added to cache');
      });
  }, 'Cache.addAll with a mix of succeeding and failing requests');

cache_test(function(cache, test) {
    var request = new Request('../resources/simple.txt');
    return promise_rejects(
      test,
      'InvalidStateError',
      cache.addAll([request, request]),
      'Cache.addAll should throw InvalidStateError if the same request is added ' +
      'twice.');
  }, 'Cache.addAll called with the same Request object specified twice');

done();

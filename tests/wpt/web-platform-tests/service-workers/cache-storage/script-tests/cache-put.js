if (self.importScripts) {
    importScripts('/resources/testharness.js');
    importScripts('../resources/testharness-helpers.js');
    importScripts('../resources/test-helpers.js');
}

var test_url = 'https://example.com/foo';
var test_body = 'Hello world!';

cache_test(function(cache) {
    var request = new Request(test_url);
    var response = new Response(test_body);
    return cache.put(request, response)
      .then(function(result) {
          assert_equals(result, undefined,
                        'Cache.put should resolve with undefined on success.');
        });
  }, 'Cache.put called with simple Request and Response');

cache_test(function(cache) {
    var test_url = new URL('../resources/simple.txt', location.href).href;
    var request = new Request(test_url);
    var response;
    return fetch(test_url)
      .then(function(fetch_result) {
          response = fetch_result.clone();
          return cache.put(request, fetch_result);
        })
      .then(function() {
          return cache.match(test_url);
        })
      .then(function(result) {
          assert_object_equals(result, response,
                               'Cache.put should update the cache with ' +
                               'new request and response.');
          return result.text();
        })
      .then(function(body) {
          assert_equals(body, 'a simple text file\n',
                        'Cache.put should store response body.');
        });
  }, 'Cache.put called with Request and Response from fetch()');

cache_test(function(cache) {
    var request = new Request(test_url);
    var response = new Response(test_body);
    assert_false(request.bodyUsed,
                 '[https://fetch.spec.whatwg.org/#dom-body-bodyused] ' +
                 'Request.bodyUsed should be initially false.');
    return cache.put(request, response)
      .then(function() {
        assert_false(request.bodyUsed,
                     'Cache.put should not mark empty request\'s body used');
      });
  }, 'Cache.put with Request without a body');

cache_test(function(cache) {
    var request = new Request(test_url);
    var response = new Response();
    assert_false(response.bodyUsed,
                 '[https://fetch.spec.whatwg.org/#dom-body-bodyused] ' +
                 'Response.bodyUsed should be initially false.');
    return cache.put(request, response)
      .then(function() {
        assert_false(response.bodyUsed,
                     'Cache.put should not mark empty response\'s body used');
      });
  }, 'Cache.put with Response without a body');

cache_test(function(cache) {
    var request = new Request(test_url);
    var response = new Response(test_body);
    return cache.put(request, response.clone())
      .then(function() {
          return cache.match(test_url);
        })
      .then(function(result) {
          assert_object_equals(result, response,
                               'Cache.put should update the cache with ' +
                               'new Request and Response.');
        });
  }, 'Cache.put with a Response containing an empty URL');

cache_test(function(cache) {
    var request = new Request(test_url);
    var response = new Response('', {
        status: 200,
        headers: [['Content-Type', 'text/plain']]
      });
    return cache.put(request, response)
      .then(function() {
          return cache.match(test_url);
        })
      .then(function(result) {
          assert_equals(result.status, 200, 'Cache.put should store status.');
          assert_equals(result.headers.get('Content-Type'), 'text/plain',
                        'Cache.put should store headers.');
          return result.text();
        })
      .then(function(body) {
          assert_equals(body, '',
                        'Cache.put should store response body.');
        });
  }, 'Cache.put with an empty response body');

cache_test(function(cache) {
    var test_url = new URL('../resources/fetch-status.py?status=500', location.href).href;
    var request = new Request(test_url);
    var response;
    return fetch(test_url)
      .then(function(fetch_result) {
          assert_equals(fetch_result.status, 500,
                        'Test framework error: The status code should be 500.');
          response = fetch_result.clone();
          return cache.put(request, fetch_result);
        })
      .then(function() {
          return cache.match(test_url);
        })
      .then(function(result) {
          assert_object_equals(result, response,
                               'Cache.put should update the cache with ' +
                               'new request and response.');
          return result.text();
        })
      .then(function(body) {
          assert_equals(body, '',
                        'Cache.put should store response body.');
        });
  }, 'Cache.put with HTTP 500 response');

cache_test(function(cache) {
    var alternate_response_body = 'New body';
    var alternate_response = new Response(alternate_response_body,
                                          { statusText: 'New status' });
    return cache.put(new Request(test_url),
                     new Response('Old body', { statusText: 'Old status' }))
      .then(function() {
          return cache.put(new Request(test_url), alternate_response.clone());
        })
      .then(function() {
          return cache.match(test_url);
        })
      .then(function(result) {
          assert_object_equals(result, alternate_response,
                               'Cache.put should replace existing ' +
                               'response with new response.');
          return result.text();
        })
      .then(function(body) {
          assert_equals(body, alternate_response_body,
                        'Cache put should store new response body.');
        });
  }, 'Cache.put called twice with matching Requests and different Responses');

cache_test(function(cache) {
    var first_url = test_url;
    var second_url = first_url + '#(O_o)';
    var alternate_response_body = 'New body';
    var alternate_response = new Response(alternate_response_body,
                                          { statusText: 'New status' });
    return cache.put(new Request(first_url),
                     new Response('Old body', { statusText: 'Old status' }))
      .then(function() {
          return cache.put(new Request(second_url), alternate_response.clone());
        })
      .then(function() {
          return cache.match(test_url);
        })
      .then(function(result) {
          assert_object_equals(result, alternate_response,
                               'Cache.put should replace existing ' +
                               'response with new response.');
          return result.text();
        })
      .then(function(body) {
          assert_equals(body, alternate_response_body,
                        'Cache put should store new response body.');
        });
  }, 'Cache.put called twice with request URLs that differ only by a fragment');

cache_test(function(cache) {
    var entries = {
      dark: {
        url: 'http://darkhelmet:12345@example.com/spaceballs',
        body: 'Moranis'
      },

      skroob: {
        url: 'http://skroob:12345@example.com/spaceballs',
        body: 'Brooks'
      },

      control: {
        url: 'http://example.com/spaceballs',
        body: 'v(o.o)v'
      }
    };

    return Promise.all(Object.keys(entries).map(function(key) {
        return cache.put(new Request(entries[key].url),
                         new Response(entries[key].body));
      }))
      .then(function() {
          return Promise.all(Object.keys(entries).map(function(key) {
              return cache.match(entries[key].url)
                .then(function(result) {
                    return result.text();
                  })
                .then(function(body) {
                    assert_equals(body, entries[key].body,
                                  'Cache put should store response body.');
                  });
            }));
        });
  }, 'Cache.put with request URLs containing embedded credentials');

cache_test(function(cache) {
    var url = 'http://example.com/foo';
    return cache.put(url, new Response('some body'))
      .then(function() { return cache.match(url); })
      .then(function(response) { return response.text(); })
      .then(function(body) {
          assert_equals(body, 'some body',
                        'Cache.put should accept a string as request.');
        });
  }, 'Cache.put with a string request');

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.put(new Request(test_url), 'Hello world!'),
      new TypeError(),
      'Cache.put should only accept a Response object as the response.');
  }, 'Cache.put with an invalid response');

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.put(new Request('file:///etc/passwd'),
                new Response(test_body)),
      new TypeError(),
      'Cache.put should reject non-HTTP/HTTPS requests with a TypeError.');
  }, 'Cache.put with a non-HTTP/HTTPS request');

cache_test(function(cache) {
    var response = new Response(test_body);
    return cache.put(new Request('relative-url'), response.clone())
      .then(function() {
          return cache.match(new URL('relative-url', location.href).href);
        })
      .then(function(result) {
          assert_object_equals(result, response,
                               'Cache.put should accept a relative URL ' +
                               'as the request.');
        });
  }, 'Cache.put with a relative URL');

cache_test(function(cache) {
    var request = new Request('http://example.com/foo', { method: 'HEAD' });
    return assert_promise_rejects(
      cache.put(request, new Response(test_body)),
      new TypeError(),
      'Cache.put should throw a TypeError for non-GET requests.');
  }, 'Cache.put with a non-GET request');

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.put(new Request(test_url), null),
      new TypeError(),
      'Cache.put should throw a TypeError for a null response.');
  }, 'Cache.put with a null response');

cache_test(function(cache) {
    var request = new Request(test_url, {method: 'POST', body: test_body});
    return assert_promise_rejects(
      cache.put(request, new Response(test_body)),
      new TypeError(),
      'Cache.put should throw a TypeError for a POST request.');
  }, 'Cache.put with a POST request');

cache_test(function(cache) {
    var response = new Response(test_body);
    assert_false(response.bodyUsed,
                 '[https://fetch.spec.whatwg.org/#dom-body-bodyused] ' +
                 'Response.bodyUsed should be initially false.');
    return response.text().then(function() {
      assert_true(
        response.bodyUsed,
        '[https://fetch.spec.whatwg.org/#concept-body-consume-body] ' +
          'The text() method should set "body used" flag.');
      return assert_promise_rejects(
        cache.put(new Request(test_url), response),
        new TypeError,
        '[https://slightlyoff.github.io/ServiceWorker/spec/service_worker/index.html#cache-put] ' +
        'Cache put should reject with TypeError when Response ' +
        'body is already used.');
      });
  }, 'Cache.put with a used response body');

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.put(new Request(test_url),
                new Response(test_body, { headers: { VARY: '*' }})),
      new TypeError(),
      'Cache.put should reject VARY:* Responses with a TypeError.');
  }, 'Cache.put with a VARY:* Response');

cache_test(function(cache) {
    return assert_promise_rejects(
      cache.put(new Request(test_url),
                new Response(test_body,
                             { headers: { VARY: 'Accept-Language,*' }})),
      new TypeError(),
      'Cache.put should reject Responses with an embedded VARY:* with a ' +
      'TypeError.');
  }, 'Cache.put with an embedded VARY:* Response');

done();

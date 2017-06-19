self.addEventListener('install', function(event) {
    var scope = registration.scope;
    var scope_url = new URL(scope);

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({});
          });
      }, 'Invalid options');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: scope, origins: ['*']});
          });
      }, 'Scopes not an array');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [{}], origins: ['*']});
          });
      }, 'Scopes not a string in array');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: ['/foo'], origins: ['*']});
          });
      }, 'Relative url not under scope');

    test(function() {
        var url = new URL(scope_url);
        url.host = 'example.com';
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [url.href], origins: ['*']});
          });
      }, 'Absolute url not under scope');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [], origins: ['*']});
          });
      }, 'Empty scope array');

    async_test(function(t) {
        self.setTimeout(t.step_func(function() {
            assert_throws('InvalidStateError', function() {
                event.registerForeignFetch({scopes: [scope], origins: ['*']});
              });
            t.done();
          }), 1);
      }, 'Call after event returned');

    test(function() {
        event.registerForeignFetch({scopes: [scope], origins: ['*']});
      }, 'Valid scopes with wildcard origin string');

    test(function() {
        event.registerForeignFetch({scopes: [scope, scope + '/foo'], origins: ['*']});
      }, 'Absolute urls');

    test(function() {
        // Figure out scope relative to location of this script:
        var local_dir = location.pathname;
        local_dir = local_dir.substr(0, local_dir.lastIndexOf('/'));
        assert_true(scope_url.pathname.startsWith(local_dir));
        var relative_scope = scope_url.pathname.substr(local_dir.length + 1);

        event.registerForeignFetch({scopes: [
          scope_url.pathname,
          relative_scope,
          './' + relative_scope,
          relative_scope + '/foo'], origins: ['*']});
      }, 'Relative urls');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [scope]});
          });
      }, 'No origins specified');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [scope], origins: {}});
          });
      }, 'Origins not a string or array');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [scope], origins: [{}]});
          });
      }, 'Origins contains something not a string');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [scope], origins: ['/foo']});
          });
      }, 'Origin not an absolute URL');

    test(function() {
        event.registerForeignFetch({scopes: [scope], origins: ['*']});
      }, 'Wildcard origin string in array');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [scope], origins: 'https://example.com/'});
          });
      }, 'Origin string');

    test(function() {
        event.registerForeignFetch({scopes: [scope], origins: ['https://example.com/']});
      }, 'Origin string in array');

    test(function() {
        event.registerForeignFetch({
            scopes: [scope], origins: ['https://example.com/', 'https://chromium.org']});
      }, 'Array with multiple origins');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [scope],
                                        origins: ['*', 'https://example.com/']});
          });
      }, 'Origins includes wildcard and other strings');

    test(function() {
        assert_throws(new TypeError(), function() {
            event.registerForeignFetch({scopes: [scope],
                                        origins: ['https://example.com/', '*']});
          });
      }, 'Origins includes other strings and wildcard');
  });

// Import testharness after install handler to make sure our install handler
// runs first. Otherwise only one test will run.
importScripts('/resources/testharness.js');

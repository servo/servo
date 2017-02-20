self.addEventListener('install', function(event) {
    test(function() {
        assert_throws(new TypeError(), function() {
            new ForeignFetchEvent('type');
          });
      }, 'ForeignFetchEvent constructor with no init dict');

    test(function() {
        assert_throws(new TypeError(), function() {
            new ForeignFetchEvent('type', {});
          });
      }, 'ForeignFetchEvent constructor with empty init dict');

    test(function() {
        assert_throws(new TypeError(), function() {
            new ForeignFetchEvent('type', { request: null });
          });
      }, 'ForeignFetchEvent constructor with null request');

    test(function() {
        var request = new Request('https://www.example.com/');
        var event = new ForeignFetchEvent('type', { request: request, origin: 'origin' });
        assert_equals(event.type, 'type');
        assert_equals(event.request, request);
        assert_equals(event.origin, 'origin');
      }, 'ForeignFetchEvent constructor with all init dict members');
  });

// Import testharness after install handler to make sure our install handler
// runs first. Otherwise only one test will run.
importScripts('/resources/testharness.js');

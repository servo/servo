importScripts('worker-testharness.js');

promise_test(function() {
    // wait for the worker to reach "installing" state, otherwise skipWaiting()
    // will fail. Bug 1228277
    return new Promise(function(res, rej) {
      oninstall = res;
    }).then(() => skipWaiting())
      .then(function(result) {
          assert_equals(result, undefined,
                        'Promise should be resolved with undefined');
        })
      .then(function() {
          var promises = [];
          for (var i = 0; i < 8; ++i)
            promises.push(self.skipWaiting());
          return Promise.all(promises);
        })
      .then(function(results) {
          results.forEach(function(r) {
              assert_equals(r, undefined,
                            'Promises should be resolved with undefined');
            });
        });
  }, 'skipWaiting');

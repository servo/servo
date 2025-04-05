// META: title=Blob Content Type
// META: script=resources/support.js
// META: timeout=long

indexeddb_test(
    function upgrade(t, db) {
        db.createObjectStore('store');
    },
    function success(t, db) {
        var type = 'x-files/trust-no-one';

        var blob = new Blob(['mulder', 'scully'], {type: type});
        assert_equals(blob.type, type, 'Blob type should match constructor option');

        var tx = db.transaction('store', 'readwrite');
        tx.objectStore('store').put(blob, 'key');

        tx.oncomplete = t.step_func(function() {
          var tx = db.transaction('store', 'readonly');
          tx.objectStore('store').get('key').onsuccess =
              t.step_func(function(e) {
                var result = e.target.result;
                assert_equals(result.type, type, 'Blob type should survive round-trip');

                // Test createObjectURL i.e. a blob:// URL.
                var url = URL.createObjectURL(result);
                var xhr = new XMLHttpRequest(), async = true;
                xhr.open('GET', url, async);
                xhr.send();
                xhr.onreadystatechange = t.step_func(function() {
                  if (xhr.readyState !== XMLHttpRequest.DONE)
                    return;
                  assert_equals(
                      xhr.getResponseHeader('Content-Type'), type,
                      'Blob type should be preserved when fetched');

                  // Test POSTing blob in request.
                  var xhr2 = new XMLHttpRequest();
                  xhr2.open('POST', '../xhr/resources/content.py', true);
                  xhr2.send(result);

                  xhr2.onreadystatechange = t.step_func(function() {
                    if (xhr2.readyState !== XMLHttpRequest.DONE)
                      return;
                    assert_equals(xhr2.status, 200);
                    assert_equals(xhr2.getResponseHeader('X-Request-Content-Type'), type);
                    assert_equals(xhr2.response, 'mulderscully');
                    t.done();
                  });
                });
              });
        });
    },
    'Ensure that content type round trips when reading blob data'
);

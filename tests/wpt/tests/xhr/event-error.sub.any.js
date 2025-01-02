// META: title=XMLHttpRequest Test: event - error

async_test(function(t) {
  var client = new XMLHttpRequest();
  client.onerror = t.step_func(function (e) {
    assert_true(e instanceof ProgressEvent);
    assert_equals(e.type, "error");
    t.done();
  });

  client.open('GET', 'http://nonexistent.{{host}}:{{ports[http][0]}}');
  client.send('null');
}, 'onerror should be called');

async_test((t) => {
  const xhr = new XMLHttpRequest();
  xhr.open('GET', 'resources/bad-chunk-encoding.py');
  xhr.addEventListener('load', t.unreached_func('load'));
  xhr.addEventListener('error', t.step_func((e) => {
    assert_equals(e.loaded, 0, 'loaded');
    assert_equals(e.total, 0, 'total');
  }));
  xhr.addEventListener('loadend', t.step_func_done((e) => {
    assert_equals(e.loaded, 0, 'loaded');
    assert_equals(e.total, 0, 'total');
  }));
  xhr.send();
}, 'error while reading body should report zeros for loaded and total');

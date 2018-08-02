// META: title=XMLHttpRequest.responseType

/**
 * Author: Mathias Bynens <http://mathiasbynens.be/>
 * Author: Ms2ger <mailto:Ms2ger@gmail.com>
 *
 * Spec: <https://xhr.spec.whatwg.org/#the-responsetype-attribute>
 */
test(function() {
  var xhr = new XMLHttpRequest();
  assert_equals(xhr.responseType, '');
}, 'Initial value of responseType');

var types = ['', 'json', 'document', 'arraybuffer', 'blob', 'text', "nosuchtype"];

function isIgnoredType(type) {
  if (type == "nosuchtype") {
    return true;
  }

  if (type != "document") {
    return false;
  }

  // "document" is ignored only on workers.
  return GLOBAL.isWorker();
}

function expectedType(type) {
  if (!isIgnoredType(type)) {
    return type;
  }

  return "";
}

types.forEach(function(type) {
  test(function() {
    var xhr = new XMLHttpRequest();
    xhr.responseType = type;
    assert_equals(xhr.responseType, expectedType(type));
  }, 'Set responseType to ' + format_value(type) + ' when readyState is UNSENT.');

  test(function() {
    var xhr = new XMLHttpRequest();
    xhr.open('get', '/');
    xhr.responseType = type;
    assert_equals(xhr.responseType, expectedType(type));
  }, 'Set responseType to ' + format_value(type) + ' when readyState is OPENED.');

  async_test(function() {
    var xhr = new XMLHttpRequest();
    xhr.open('get', '/');
    xhr.onreadystatechange = this.step_func(function() {
      if (xhr.readyState === XMLHttpRequest.HEADERS_RECEIVED) {
        xhr.responseType = type;
        assert_equals(xhr.responseType, expectedType(type));
        this.done();
      }
    });
    xhr.send();
  }, 'Set responseType to ' + format_value(type) + ' when readyState is HEADERS_RECEIVED.');

  async_test(function() {
    var xhr = new XMLHttpRequest();
    xhr.open('get', '/');
    xhr.onreadystatechange = this.step_func(function() {
      if (xhr.readyState === XMLHttpRequest.LOADING) {
        if (isIgnoredType(type)) {
          xhr.responseType = type;
        } else {
          assert_throws("InvalidStateError", function() {
            xhr.responseType = type;
          });
        }
        assert_equals(xhr.responseType, "");
        this.done();
      }
    });
    xhr.send();
  }, 'Set responseType to ' + format_value(type) + ' when readyState is LOADING.');

  async_test(function() {
    var xhr = new XMLHttpRequest();
    xhr.open('get', '/');
    xhr.onreadystatechange = this.step_func(function() {
      if (xhr.readyState === XMLHttpRequest.DONE) {
        var text = xhr.responseText;
        assert_not_equals(text, "");
        if (isIgnoredType(type)) {
          xhr.responseType = type;
        } else {
          assert_throws("InvalidStateError", function() {
            xhr.responseType = type;
          });
        }
        assert_equals(xhr.responseType, "");
        assert_equals(xhr.responseText, text);
        this.done();
      }
    });
    xhr.send();
  }, 'Set responseType to ' + format_value(type) + ' when readyState is DONE.');

  // Note: the case of setting responseType first, and then calling synchronous
  // open(), is tested in open-method-responsetype-set-sync.htm.
  test(function() {
    var xhr = new XMLHttpRequest();
    xhr.open('get', '/', false);
    if (GLOBAL.isWorker() || isIgnoredType(type)) {
      // Setting responseType on workers is valid even for a sync XHR.
      xhr.responseType = type;
      assert_equals(xhr.responseType, expectedType(type));
    } else {
      assert_throws("InvalidAccessError", function() {
        xhr.responseType = type;
      });
    }
  }, 'Set responseType to ' + format_value(type) + ' when readyState is OPENED and the sync flag is set.');

  test(function() {
    var xhr = new XMLHttpRequest();
    xhr.open('get', '/', false);
    xhr.send();
    assert_equals(xhr.readyState, XMLHttpRequest.DONE);
    if (isIgnoredType(type)) {
      xhr.responseType = type;
    } else {
      assert_throws("InvalidStateError", function() {
        xhr.responseType = type;
      });
    }
    assert_equals(xhr.responseType, "");
  }, 'Set responseType to ' + format_value(type) + ' when readyState is DONE and the sync flag is set.');
});

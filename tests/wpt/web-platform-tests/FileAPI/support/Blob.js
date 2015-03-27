function test_blob(fn, expectations) {
  var expected = expectations.expected,
      type = expectations.type,
      desc = expectations.desc;

  var t = async_test(desc);
  t.step(function() {
    var blob = fn();
    assert_true(blob instanceof Blob);
    assert_false(blob instanceof File);
    assert_equals(blob.type, type);
    assert_equals(blob.size, expected.length);

    var fr = new FileReader();
    fr.onload = t.step_func_done(function(event) {
      assert_equals(this.result, expected);
    }, fr);
    fr.onerror = t.step_func(function(e) {
      assert_unreached("got error event on FileReader");
    });
    fr.readAsText(blob, "UTF-8");
  });
}

function test_blob_binary(fn, expectations) {
  var expected = expectations.expected,
      type = expectations.type,
      desc = expectations.desc;

  var t = async_test(desc);
  t.step(function() {
    var blob = fn();
    assert_true(blob instanceof Blob);
    assert_false(blob instanceof File);
    assert_equals(blob.type, type);
    assert_equals(blob.size, expected.length);

    var fr = new FileReader();
    fr.onload = t.step_func_done(function(event) {
      assert_true(this.result instanceof ArrayBuffer,
                  "Result should be an ArrayBuffer");
      assert_array_equals(new Uint8Array(this.result), expected);
    }, fr);
    fr.onerror = t.step_func(function(e) {
      assert_unreached("got error event on FileReader");
    });
    fr.readAsArrayBuffer(blob);
  });
}

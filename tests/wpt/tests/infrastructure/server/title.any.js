// META: global=window,dedicatedworker,sharedworker
// META: title=foobar
test(t => {
  if (GLOBAL.isWindow()) {
    assert_equals(document.title, 'foobar');
    assert_false('META_TITLE' in self);
  } else {
    assert_equals(META_TITLE, 'foobar');
  }
  assert_equals(t.name, 'foobar');
});

done();

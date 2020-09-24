importScripts("/resources/testharness.js");

test((t) => {
  const ms = new MediaSource();
  assert_equals(ms.readyState, "closed");
}, "MediaSource construction succeeds with initial closed readyState in dedicated worker");

test((t) => {
  const ms = new MediaSource();
  const url = URL.createObjectURL(ms);
  assert_true(url != null);
  assert_true(url.match(/^blob:.+/) != null);
}, "URL.createObjectURL(mediaSource) in dedicated worker returns a Blob URI");

test((t) => {
  const ms = new MediaSource();
  const url1 = URL.createObjectURL(ms);
  const url2 = URL.createObjectURL(ms);
  URL.revokeObjectURL(url1);
  URL.revokeObjectURL(url2);
}, "URL.revokeObjectURL(mediaSource) in dedicated worker with two url for same MediaSource");

done();

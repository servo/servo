//META: title=navigator.wakeLock.request() with invalid type

promise_test(async t => {
  return promise_rejects_js(t, TypeError, navigator.wakeLock.request());
}, "'TypeError' is thrown when set an empty wake lock type");

promise_test(t => {
  const invalidTypes = [
    "invalid",
    null,
    123,
    {},
    "",
    true
  ];
  return Promise.all(invalidTypes.map(invalidType => {
    return promise_rejects_js(t, TypeError, navigator.wakeLock.request(invalidType));
  }));
}, "'TypeError' is thrown when set an invalid wake lock type");

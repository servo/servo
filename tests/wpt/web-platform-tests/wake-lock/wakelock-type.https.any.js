//META: title=WakeLock.request() with invalid type

promise_test(async t => {
  await promise_rejects(t, new TypeError(), WakeLock.request());
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
    return promise_rejects(t, new TypeError(), WakeLock.request(invalidType));
  }));
}, "'TypeError' is thrown when set an invalid wake lock type");

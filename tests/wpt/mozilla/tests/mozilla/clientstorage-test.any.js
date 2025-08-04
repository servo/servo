test(function() {
  let clientStorageTest = new ClientStorageTest();

  let result = clientStorageTest.test();
  assert_equals(result, 42);
});

async_test(function(t) {
  let clientStorageTest = new ClientStorageTest();

  clientStorageTest.onpong = function() {
    t.done();
  }

  clientStorageTest.ping();
});

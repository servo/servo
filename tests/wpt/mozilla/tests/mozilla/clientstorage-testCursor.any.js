async_test(function(t) {
  let clientStorageTest = new ClientStorageTest();

  let cursor = clientStorageTest.openCursor();

  let count = 0;

  cursor.onresponse = function(event) {
    count++;

    let number = event.loaded;
    assert_equals(number, count);

    if (count < 3) {
      cursor.continue_();
    } else {
      t.done();
    }
  };

  cursor.continue_();
});

'use strict';

promise_test(async test => {
  let root = await navigator.storage.getDirectory();
  assert_equals(root.name, '');
}, 'getDirectory returns a directory whose name is the empty string');

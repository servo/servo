// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var await = 1;

async function getClass() {
  return class {
    x = await;
  };
}

getClass().then(cl => {
  assert.sameValue(new cl().x, 1);
});

assert.throws(SyntaxError, function() {
  eval("async () => class { [await] = 1 };");
});

assert.throws(SyntaxError, function() {
  eval("async () => class { x = await 1 };");
});

// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

assert.throws(ReferenceError, function() {
  {
    let y = f();
    function f() { y; }
  }
});

assert.throws(ReferenceError, function() {
  switch (1) {
    case 0:
      let x;
    case 1:
      (function() { x; })();
  }
});

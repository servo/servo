// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

assert.throws(SyntaxError, function() {
  eval("class X { x: 1 }");
});

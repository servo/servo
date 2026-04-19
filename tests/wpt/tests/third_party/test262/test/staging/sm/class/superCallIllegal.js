// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// super() invalid outside derived class constructors, including in dynamic
// functions and eval
assert.throws(SyntaxError, () => new Function("super();"));
assert.throws(SyntaxError, () => eval("super()"));


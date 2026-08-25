// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Bug 1667094.

var obj = {
  exec() {
    return function(){};
  }
};

assert.sameValue(RegExp.prototype.test.call(obj, ""), true);


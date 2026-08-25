// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Math-shell.js]
description: |
  pending
esid: pending
---*/
for (var i = -1074; i < 1023; i++)
    assertNear(Math.log2(Math.pow(2, i)), i);

assertNear(Math.log2(5), 2.321928094887362);
assertNear(Math.log2(7), 2.807354922057604);
assertNear(Math.log2(Math.E), Math.LOG2E);


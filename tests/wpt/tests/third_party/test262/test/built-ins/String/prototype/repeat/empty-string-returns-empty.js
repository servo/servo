// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.13
description: >
  An empty repeated n times will return an empty string.
info: |
  21.1.3.13 String.prototype.repeat ( count )

  8. Let T be a String value that is made from n copies of S appended together.
  If n is 0, T is the empty String.
  9. Return T.
---*/

assert.sameValue(''.repeat(1), '', '"".repeat(1)');
assert.sameValue(''.repeat(3), '', '"".repeat(3)');

var maxSafe32bitInt = 2147483647;
assert.sameValue(''.repeat(maxSafe32bitInt), '', '"".repeat(maxSafe32bitInt)');

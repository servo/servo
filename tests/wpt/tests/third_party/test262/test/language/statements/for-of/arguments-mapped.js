// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4
description: Mapped arguments object traversal using for..of
info: |
    "Mapped" arguments objects should be able to be traversed using a `for..of`
    loop.
flags: [noStrict]
---*/

var i = 0;

(function() {
  for (var value of arguments) {
    assert.sameValue(value, arguments[i], 'argument at index ' + i);
    i++;
  }
}(0, 'a', true, false, null, undefined, NaN));

assert.sameValue(i, 7, 'Visits all arguments');

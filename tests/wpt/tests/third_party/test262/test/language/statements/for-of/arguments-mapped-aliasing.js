// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4
description: >
    Mapped arguments object mutation via alias during traversal using for..of
info: |
    "Mapped" arguments objects should be able to be traversed using a `for..of`
    loop, and dynamic changes to the formal parameters should be reflected in
    the iterated values.
flags: [noStrict]
---*/

var expected = [1, 3, 1];
var i = 0;

(function(a, b, c) {
  for (var value of arguments) {
    a = b;
    b = c;
    c = i;
    assert.sameValue(value, expected[i], 'argument at index ' + i);
    i++;
  }

}(1, 2, 3));

assert.sameValue(i, 3, 'Visits all arguments');

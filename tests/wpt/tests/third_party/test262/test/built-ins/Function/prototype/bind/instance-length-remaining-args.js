// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.bind
description: >
  "length" value of a bound function is set to remaining number
  of arguments expected by target function. Extra arguments are ignored.
info: |
  Function.prototype.bind ( thisArg, ...args )

  [...]
  5. Let targetHasLength be ? HasOwnProperty(Target, "length").
  6. If targetHasLength is true, then
    a. Let targetLen be ? Get(Target, "length").
    b. If Type(targetLen) is not Number, let L be 0.
    c. Else,
      i. Set targetLen to ! ToInteger(targetLen).
      ii. Let L be the larger of 0 and the result of targetLen minus the number of elements of args.
  7. Else, let L be 0.
  8. Perform ! SetFunctionLength(F, L).
  [...]
---*/

function foo() {}

assert.sameValue(foo.bind(null).length, 0, '0/0');
assert.sameValue(foo.bind(null, 1).length, 0, '1/0');

function bar(x, y) {}

assert.sameValue(bar.bind(null).length, 2, '0/2');
assert.sameValue(bar.bind(null, 1).length, 1, '1/2');
assert.sameValue(bar.bind(null, 1, 2).length, 0, '2/2');
assert.sameValue(bar.bind(null, 1, 2, 3).length, 0, '3/2');

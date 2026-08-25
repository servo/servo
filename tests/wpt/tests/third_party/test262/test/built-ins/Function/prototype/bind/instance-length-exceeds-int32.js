// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.bind
description: >
  The target function length can exceed 2**31-1.
info: |
  19.2.3.2 Function.prototype.bind ( thisArg, ...args )

  ...
  6. If targetHasLength is true, then
    a. Let targetLen be ? Get(Target, "length").
    b. If Type(targetLen) is not Number, let L be 0.
    c. Else,
      i. Let targetLen be ToInteger(targetLen).
      ii. Let L be the larger of 0 and the result of targetLen minus the number of elements of args.
  ...
  8. Perform ! SetFunctionLength(F, L).
  ...
---*/

function f(){}
Object.defineProperty(f, "length", {value: 2147483648});

assert.sameValue(f.bind().length, 2147483648);

Object.defineProperty(f, "length", {value: Number.MAX_SAFE_INTEGER});
assert.sameValue(f.bind().length, Number.MAX_SAFE_INTEGER);

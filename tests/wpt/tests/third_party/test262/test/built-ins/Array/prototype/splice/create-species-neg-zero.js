// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.splice
description: The value `-0` is converted to `0`
info: |
    [...]
    3. Let relativeStart be ? ToInteger(start).
    4. If relativeStart < 0, let actualStart be max((len + relativeStart), 0);
       else let actualStart be min(relativeStart, len).
    5. If the number of actual arguments is 0, then
       [...]
    6. Else if the number of actual arguments is 1, then
       [...]
    7. Else,
       a. Let insertCount be the number of actual arguments minus 2.
       b. Let dc be ? ToInteger(deleteCount).
       c. Let actualDeleteCount be min(max(dc, 0), len - actualStart).
    [...]
    9. Let A be ? ArraySpeciesCreate(O, actualDeleteCount).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    2. If length is -0, let length be +0.
    [...]
    5. Let C be ? Get(originalArray, "constructor").
    [...]
    7. If Type(C) is Object, then
       a. Let C be ? Get(C, @@species).
       b. If C is null, let C be undefined.
    [...]
    10. Return ? Construct(C, « length »).
features: [Symbol.species]
---*/

var args;
var Ctor = function() {
  args = arguments;
};
var a = [];
a.constructor = {};
a.constructor[Symbol.species] = Ctor;

a.splice(0, -0);

assert.sameValue(args.length, 1);
assert.sameValue(args[0], 0);

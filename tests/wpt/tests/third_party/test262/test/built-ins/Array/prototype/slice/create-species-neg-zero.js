// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.slice
description: The value `-0` is converted to `0`
info: |
    [...]
    3. Let relativeStart be ? ToInteger(start).
    4. If relativeStart < 0, let k be max((len + relativeStart), 0); else let k
       be min(relativeStart, len).
    5. If end is undefined, let relativeEnd be len; else let relativeEnd be ?
       ToInteger(end).
    6. If relativeEnd < 0, let final be max((len + relativeEnd), 0); else let
       final be min(relativeEnd, len).
    7. Let count be max(final - k, 0).
    8. Let A be ? ArraySpeciesCreate(O, count).
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

a.slice(0, -0);

assert.sameValue(args.length, 1);
assert.sameValue(args[0], 0);

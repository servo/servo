// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
  Create species constructor with length exceeding integer limit and ensure MOP
  operations are called in correct order.
info: |
  ...
   9. Let A be ? ArraySpeciesCreate(O, actualDeleteCount).
  10. Let k be 0.
  11. Repeat, while k < actualDeleteCount
    a. Let from be ! ToString(actualStart+k).
    b. Let fromPresent be ? HasProperty(O, from).
    c. If fromPresent is true, then
       i. Let fromValue be ? Get(O, from).
      ii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(k), fromValue).
    d. Increment k by 1.
  12. Perform ? Set(A, "length", actualDeleteCount, true).
  ...
includes: [compareArray.js, proxyTrapsHelper.js]
features: [Symbol.species, exponentiation]
---*/

function StopSplice() {}

var traps = [];
var targetLength;

var array = ["no-hole", /* hole */ , "stop"];
array.constructor = {
  [Symbol.species]: function(n) {
    targetLength = n;
    return target;
  }
};

var source = new Proxy(array, allowProxyTraps({
  get(t, pk, r) {
    traps.push(`source.[[Get]]:${String(pk)}`);

    // length property exceeding 2^53-1.
    if (pk === "length")
      return 2 ** 53 + 2;

    return Reflect.get(t, pk, r);
  },
  has(t, pk, r) {
    traps.push(`source.[[Has]]:${String(pk)}`);

    return Reflect.get(t, pk, r);
  },
}));

var target = new Proxy([], allowProxyTraps({
  defineProperty(t, pk, desc) {
    traps.push(`target.[[DefineProperty]]:${String(pk)}`);

    if (pk === "0" || pk === "1")
      return Reflect.defineProperty(t, pk, desc);

    throw new StopSplice();
  }
}));

assert.throws(StopSplice, function() {
  // deleteCount argument exceeding 2^53-1.
  Array.prototype.splice.call(source, 0, 2 ** 53 + 4);
}, '// deleteCount argument exceeding 2^53-1. Array.prototype.splice.call(source, 0, 2 ** 53 + 4) throws a StopSplice exception');

assert.sameValue(targetLength, 2 ** 53 - 1,
  'The value of targetLength is expected to be 2 ** 53 - 1');

assert.compareArray(traps, [
  "source.[[Get]]:length",

  "source.[[Get]]:constructor",

  "source.[[Has]]:0",
  "source.[[Get]]:0",
  "target.[[DefineProperty]]:0",

  "source.[[Has]]:1",

  "source.[[Has]]:2",
  "source.[[Get]]:2",
  "target.[[DefineProperty]]:2",
], 'The value of traps is expected to be [\n  "source.[[Get]]:length",\n\n  "source.[[Get]]:constructor",\n\n  "source.[[Has]]:0",\n  "source.[[Get]]:0",\n  "target.[[DefineProperty]]:0",\n\n  "source.[[Has]]:1",\n\n  "source.[[Has]]:2",\n  "source.[[Get]]:2",\n  "target.[[DefineProperty]]:2",\n]');

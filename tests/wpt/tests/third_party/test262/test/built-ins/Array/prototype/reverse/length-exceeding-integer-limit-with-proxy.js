// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reverse
description: >
  Ensure correct MOP operations are called when length exceeds 2^53-1.
includes: [compareArray.js, proxyTrapsHelper.js]
features: [exponentiation]
---*/

function StopReverse() {}

var arrayLike = {
  0: "zero",
  /* 1: hole, */
  2: "two",
  /* 3: hole, */

  get 4() {
    throw new StopReverse();
  },

  9007199254740987: "2**53-5",
  /* 9007199254740988: hole, */
  /* 9007199254740989: hole, */
  9007199254740990: "2**53-2",

  length: 2 ** 53 + 2,
};

var traps = [];

var proxy = new Proxy(arrayLike, allowProxyTraps({
  getOwnPropertyDescriptor(t, pk) {
    traps.push(`GetOwnPropertyDescriptor:${String(pk)}`);
    return Reflect.getOwnPropertyDescriptor(t, pk);
  },
  defineProperty(t, pk, desc) {
    traps.push(`DefineProperty:${String(pk)}`);
    return Reflect.defineProperty(t, pk, desc);
  },
  has(t, pk) {
    traps.push(`Has:${String(pk)}`);
    return Reflect.has(t, pk);
  },
  get(t, pk, r) {
    traps.push(`Get:${String(pk)}`);
    return Reflect.get(t, pk, r);
  },
  set(t, pk, v, r) {
    traps.push(`Set:${String(pk)}`);
    return Reflect.set(t, pk, v, r);
  },
  deleteProperty(t, pk) {
    traps.push(`Delete:${String(pk)}`);
    return Reflect.deleteProperty(t, pk);
  },
}))

// Uses a separate exception than Test262Error, so that errors from allowProxyTraps
// are properly propagated.
assert.throws(StopReverse, function() {
  Array.prototype.reverse.call(proxy);
}, 'Array.prototype.reverse.call(proxy) throws a StopReverse exception');

assert.compareArray(traps, [
  // Initial get length operation.
  "Get:length",

  // Lower and upper index are both present.
  "Has:0",
  "Get:0",
  "Has:9007199254740990",
  "Get:9007199254740990",
  "Set:0",
  "GetOwnPropertyDescriptor:0",
  "DefineProperty:0",
  "Set:9007199254740990",
  "GetOwnPropertyDescriptor:9007199254740990",
  "DefineProperty:9007199254740990",

  // Lower and upper index are both absent.
  "Has:1",
  "Has:9007199254740989",

  // Lower index is present, upper index is absent.
  "Has:2",
  "Get:2",
  "Has:9007199254740988",
  "Delete:2",
  "Set:9007199254740988",
  "GetOwnPropertyDescriptor:9007199254740988",
  "DefineProperty:9007199254740988",

  // Lower index is absent, upper index is present.
  "Has:3",
  "Has:9007199254740987",
  "Get:9007199254740987",
  "Set:3",
  "GetOwnPropertyDescriptor:3",
  "DefineProperty:3",
  "Delete:9007199254740987",

  // Stop exception.
  "Has:4",
  "Get:4",
], 'The value of traps is expected to be [\n  // Initial get length operation.\n  "Get:length",\n\n  // Lower and upper index are both present.\n  "Has:0",\n  "Get:0",\n  "Has:9007199254740990",\n  "Get:9007199254740990",\n  "Set:0",\n  "GetOwnPropertyDescriptor:0",\n  "DefineProperty:0",\n  "Set:9007199254740990",\n  "GetOwnPropertyDescriptor:9007199254740990",\n  "DefineProperty:9007199254740990",\n\n  // Lower and upper index are both absent.\n  "Has:1",\n  "Has:9007199254740989",\n\n  // Lower index is present, upper index is absent.\n  "Has:2",\n  "Get:2",\n  "Has:9007199254740988",\n  "Delete:2",\n  "Set:9007199254740988",\n  "GetOwnPropertyDescriptor:9007199254740988",\n  "DefineProperty:9007199254740988",\n\n  // Lower index is absent, upper index is present.\n  "Has:3",\n  "Has:9007199254740987",\n  "Get:9007199254740987",\n  "Set:3",\n  "GetOwnPropertyDescriptor:3",\n  "DefineProperty:3",\n  "Delete:9007199254740987",\n\n  // Stop exception.\n  "Has:4",\n  "Get:4",\n]');

assert.sameValue(arrayLike.length, 2 ** 53 + 2, 'The value of arrayLike.length is expected to be 2 ** 53 + 2');

assert.sameValue(arrayLike[0], "2**53-2", 'The value of arrayLike[0] is expected to be "2**53-2"');
assert.sameValue(1 in arrayLike, false, 'The result of evaluating (1 in arrayLike) is expected to be false');
assert.sameValue(2 in arrayLike, false, 'The result of evaluating (2 in arrayLike) is expected to be false');
assert.sameValue(arrayLike[3], "2**53-5", 'The value of arrayLike[3] is expected to be "2**53-5"');

assert.sameValue(9007199254740987 in arrayLike, false, 'The result of evaluating (9007199254740987 in arrayLike) is expected to be false');
assert.sameValue(arrayLike[9007199254740988], "two", 'The value of arrayLike[9007199254740988] is expected to be "two"');
assert.sameValue(9007199254740989 in arrayLike, false, 'The result of evaluating (9007199254740989 in arrayLike) is expected to be false');
assert.sameValue(arrayLike[9007199254740990], "zero", 'The value of arrayLike[9007199254740990] is expected to be "zero"');

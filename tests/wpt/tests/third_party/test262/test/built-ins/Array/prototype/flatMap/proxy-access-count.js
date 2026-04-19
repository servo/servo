// Copyright (C) 2018 Richard Lawrence. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: >
  properties are accessed correct number of times by .flatMap
info: |
  Array.prototype.flatMap ( mapperFunction [ , thisArg ] )

  ...
  6. Perform ? FlattenIntoArray(A, O, sourceLen, 0, 1, mapperFunction, T).

  FlattenIntoArray (target, source, sourceLen, start, depth [ , mapperFunction, thisArg ])

  3. Repeat, while sourceIndex < sourceLen
    a. Let P be ! ToString(sourceIndex).
    b. Let exists be ? HasProperty(source, P).
    c. If exists is true, then
      i. Let element be ? Get(source, P).
features: [Array.prototype.flatMap]
includes: [compareArray.js]
---*/

assert.sameValue(
  typeof Array.prototype.flatMap,
  'function',
  'The value of `typeof Array.prototype.flatMap` is expected to be "function"'
);

const getCalls = [], hasCalls = [];
const handler = {
  get : function (t, p, r) { getCalls.push(p); return Reflect.get(t, p, r); },
  has : function (t, p, r) { hasCalls.push(p); return Reflect.has(t, p, r); }
}

const tier2 = new Proxy([4, 3], handler);
const tier1 = new Proxy([2, [3, 4, 2, 2], 5, tier2, 6], handler);

Array.prototype.flatMap.call(tier1, function(a){ return a; });

assert.compareArray(getCalls, ["length", "constructor", "0", "1", "2", "3", "length", "0", "1", "4"], 'The value of getCalls is expected to be ["length", "constructor", "0", "1", "2", "3", "length", "0", "1", "4"]');
assert.compareArray(hasCalls, ["0", "1", "2", "3", "0", "1", "4"], 'The value of hasCalls is expected to be ["0", "1", "2", "3", "0", "1", "4"]');

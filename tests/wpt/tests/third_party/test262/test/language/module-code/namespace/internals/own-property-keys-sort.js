// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-ownpropertykeys
description: >
    The [[OwnPropertyKeys]] internal method reflects the sorted order
info: |
    1. Let exports be a copy of the value of O's [[Exports]] internal slot.
    2. Let symbolKeys be ! OrdinaryOwnPropertyKeys(O).
    3. Append all the entries of symbolKeys to the end of exports.
    4. Return exports.
flags: [module]
features: [Reflect, Symbol.toStringTag]
---*/

var x;
export { x as π }; // u03c0
export { x as az };
export { x as __ };
export { x as za };
export { x as Z };
export { x as \u03bc };
export { x as z };
export { x as zz };
export { x as a };
export { x as A };
export { x as aa };
export { x as λ }; // u03bb
export { x as _ };
export { x as $$ };
export { x as $ };
export default null;

import * as ns from './own-property-keys-sort.js';

var stringKeys = Object.getOwnPropertyNames(ns);

assert.sameValue(stringKeys.length, 16);
assert.sameValue(stringKeys[0], '$', 'stringKeys[0] === "$"');
assert.sameValue(stringKeys[1], '$$', 'stringKeys[1] === "$$"');
assert.sameValue(stringKeys[2], 'A', 'stringKeys[2] === "A"');
assert.sameValue(stringKeys[3], 'Z', 'stringKeys[3] === "Z"');
assert.sameValue(stringKeys[4], '_', 'stringKeys[4] === "_"');
assert.sameValue(stringKeys[5], '__', 'stringKeys[5] === "__"');
assert.sameValue(stringKeys[6], 'a', 'stringKeys[6] === "a"');
assert.sameValue(stringKeys[7], 'aa', 'stringKeys[7] === "aa"');
assert.sameValue(stringKeys[8], 'az', 'stringKeys[8] === "az"');
assert.sameValue(stringKeys[9], 'default', 'stringKeys[9] === "default"');
assert.sameValue(stringKeys[10], 'z', 'stringKeys[10] === "z"');
assert.sameValue(stringKeys[11], 'za', 'stringKeys[11] === "za"');
assert.sameValue(stringKeys[12], 'zz', 'stringKeys[12] === "zz"');
assert.sameValue(stringKeys[13], '\u03bb', 'stringKeys[13] === "\u03bb"');
assert.sameValue(stringKeys[14], '\u03bc', 'stringKeys[14] === "\u03bc"');
assert.sameValue(stringKeys[15], '\u03c0', 'stringKeys[15] === "\u03c0"');

var allKeys = Reflect.ownKeys(ns);
assert(
  allKeys.length >= 17,
  'at least as many keys as defined by the module and the specification'
);
assert.sameValue(allKeys[0], '$', 'allKeys[0] === "$"');
assert.sameValue(allKeys[1], '$$', 'allKeys[1] === "$$"');
assert.sameValue(allKeys[2], 'A', 'allKeys[2] === "A"');
assert.sameValue(allKeys[3], 'Z', 'allKeys[3] === "Z"');
assert.sameValue(allKeys[4], '_', 'allKeys[4] === "_"');
assert.sameValue(allKeys[5], '__', 'allKeys[5] === "__"');
assert.sameValue(allKeys[6], 'a', 'allKeys[6] === "a"');
assert.sameValue(allKeys[7], 'aa', 'allKeys[7] === "aa"');
assert.sameValue(allKeys[8], 'az', 'allKeys[8] === "az"');
assert.sameValue(allKeys[9], 'default', 'allKeys[9] === "default"');
assert.sameValue(allKeys[10], 'z', 'allKeys[10] === "z"');
assert.sameValue(allKeys[11], 'za', 'allKeys[11] === "za"');
assert.sameValue(allKeys[12], 'zz', 'allKeys[12] === "zz"');
assert.sameValue(allKeys[13], '\u03bb', 'allKeys[13] === "\u03bb"');
assert.sameValue(allKeys[14], '\u03bc', 'allKeys[14] === "\u03bc"');
assert.sameValue(allKeys[15], '\u03c0', 'allKeys[15] === "\u03c0"');
assert(
  allKeys.indexOf(Symbol.toStringTag) > 15,
  'keys array includes Symbol.toStringTag'
);

// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Return value when matches occur with the `global` flag
es6id: 21.2.5.6
info: |
    [...]
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       e. Let A be ArrayCreate(0).
       [...]
       g. Repeat,
          i. Let result be RegExpExec(rx, S).
          ii. ReturnIfAbrupt(result).
          iii. If result is null, then
               1. If n=0, return null.
               2. Else, return A.
features: [Symbol.match]
---*/

var result = /.(.)./g[Symbol.match]('abcdefghi');

assert(Array.isArray(result));

assert(
  !Object.prototype.hasOwnProperty.call(result, 'index'),
  'Does not define an `index` "own" property'
);
assert.sameValue(
  result.index, undefined, 'Does not define an `index` property'
);
assert(
  !Object.prototype.hasOwnProperty.call(result, 'input'),
  'Does not define an `input` "own" property'
);
assert.sameValue(
  result.input, undefined, 'Does not define an `input` property'
);

assert.sameValue(result.length, 3);
assert.sameValue(result[0], 'abc');
assert.sameValue(result[1], 'def');
assert.sameValue(result[2], 'ghi');

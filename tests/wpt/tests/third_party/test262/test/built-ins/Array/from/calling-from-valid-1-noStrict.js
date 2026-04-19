// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.
/*---
esid: sec-array.from
description: Map function without thisArg on non strict mode
info: |
  22.1.2.1 Array.from ( items [ , mapfn [ , thisArg ] ] )

  ...
  10. Let len be ToLength(Get(arrayLike, "length")).
  11. ReturnIfAbrupt(len).
  12. If IsConstructor(C) is true, then
    a. Let A be Construct(C, «len»).
  13. Else,
    b. Let A be ArrayCreate(len).
  14. ReturnIfAbrupt(A).
  15. Let k be 0.
  16. Repeat, while k < len
    a. Let Pk be ToString(k).
    b. Let kValue be Get(arrayLike, Pk).
    c. ReturnIfAbrupt(kValue).
    d. If mapping is true, then
      i. Let mappedValue be Call(mapfn, T, «kValue, k»).
  ...
flags: [noStrict]
---*/

var list = {
  '0': 41,
  '1': 42,
  '2': 43,
  length: 3
};
var calls = [];

function mapFn(value) {
  calls.push({
    args: arguments,
    thisArg: this
  });
  return value * 2;
}

var result = Array.from(list, mapFn);

assert.sameValue(result.length, 3, 'The value of result.length is expected to be 3');
assert.sameValue(result[0], 82, 'The value of result[0] is expected to be 82');
assert.sameValue(result[1], 84, 'The value of result[1] is expected to be 84');
assert.sameValue(result[2], 86, 'The value of result[2] is expected to be 86');

assert.sameValue(calls.length, 3, 'The value of calls.length is expected to be 3');

assert.sameValue(calls[0].args.length, 2, 'The value of calls[0].args.length is expected to be 2');
assert.sameValue(calls[0].args[0], 41, 'The value of calls[0].args[0] is expected to be 41');
assert.sameValue(calls[0].args[1], 0, 'The value of calls[0].args[1] is expected to be 0');
assert.sameValue(calls[0].thisArg, this, 'The value of calls[0].thisArg is expected to be this');

assert.sameValue(calls[1].args.length, 2, 'The value of calls[1].args.length is expected to be 2');
assert.sameValue(calls[1].args[0], 42, 'The value of calls[1].args[0] is expected to be 42');
assert.sameValue(calls[1].args[1], 1, 'The value of calls[1].args[1] is expected to be 1');
assert.sameValue(calls[1].thisArg, this, 'The value of calls[1].thisArg is expected to be this');

assert.sameValue(calls[2].args.length, 2, 'The value of calls[2].args.length is expected to be 2');
assert.sameValue(calls[2].args[0], 43, 'The value of calls[2].args[0] is expected to be 43');
assert.sameValue(calls[2].args[1], 2, 'The value of calls[2].args[1] is expected to be 2');
assert.sameValue(calls[2].thisArg, this, 'The value of calls[2].thisArg is expected to be this');

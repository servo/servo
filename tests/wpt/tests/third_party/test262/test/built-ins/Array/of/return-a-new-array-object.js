// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.of
description: >
  Returns a new Array.
info: |
  Array.of ( ...items )

  1. Let len be the actual number of arguments passed to this function.
  2. Let items be the List of arguments passed to this function.
  3. Let C be the this value.
  4. If IsConstructor(C) is true, then
    a. Let A be Construct(C, «len»).
  5. Else,
    b. Let A be ArrayCreate(len).
  ...
  11. Return A.
---*/

var result = Array.of();
assert(result instanceof Array, 'The result of evaluating (result instanceof Array) is expected to be true');

result = Array.of.call(undefined);
assert(
  result instanceof Array,
  'The result of evaluating (result instanceof Array) is expected to be true'
);

result = Array.of.call(Math.cos);
assert(
  result instanceof Array,
  'The result of evaluating (result instanceof Array) is expected to be true'
);

result = Array.of.call(Math.cos.bind(Math));
assert(
  result instanceof Array,
  'The result of evaluating (result instanceof Array) is expected to be true'
);

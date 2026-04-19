// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.of
description: >
  Return abrupt from this' constructor
info: |
  Array.of ( ...items )

  1. Let len be the actual number of arguments passed to this function.
  2. Let items be the List of arguments passed to this function.
  3. Let C be the this value.
  4. If IsConstructor(C) is true, then
    a. Let A be Construct(C, «len»).
  5. Else,
    b. Let A be ArrayCreate(len).
  6. ReturnIfAbrupt(A).
  ...
---*/

function T() {
  throw new Test262Error();
}

assert.throws(Test262Error, function() {
  Array.of.call(T);
}, 'Array.of.call(T) throws a Test262Error exception');

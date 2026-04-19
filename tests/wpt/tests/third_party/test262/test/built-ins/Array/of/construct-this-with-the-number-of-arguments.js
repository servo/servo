// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.of
es6id: 22.1.2.3
description: Passes the number of arguments to the constructor it calls.
info: |
  Array.of ( ...items )

  1. Let len be the actual number of arguments passed to this function.
  2. Let items be the List of arguments passed to this function.
  3. Let C be the this value.
  4. If IsConstructor(C) is true, then
    a. Let A be Construct(C, «len»).
  ...
---*/

var len;
var hits = 0;

function C(length) {
  len = length;
  hits++;
}

Array.of.call(C);
assert.sameValue(len, 0, 'The value of len is expected to be 0');
assert.sameValue(hits, 1, 'The value of hits is expected to be 1');

Array.of.call(C, 'a', 'b')
assert.sameValue(len, 2, 'The value of len is expected to be 2');
assert.sameValue(hits, 2, 'The value of hits is expected to be 2');

Array.of.call(C, false, null, undefined);
assert.sameValue(
  len, 3,
  'The value of len is expected to be 3'
);
assert.sameValue(hits, 3, 'The value of hits is expected to be 3');

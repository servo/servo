// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.of
description: >
  Returns an instance from a custom constructor.
info: |
  Array.of ( ...items )

  ...
  4. If IsConstructor(C) is true, then
    a. Let A be Construct(C, «len»).
  ...
  11. Return A.
---*/

function Coop() {}

var coop = Array.of.call(Coop, 'Mike', 'Rick', 'Leo');

assert.sameValue(
  coop.length, 3,
  'The value of coop.length is expected to be 3'
);
assert.sameValue(
  coop[0], 'Mike',
  'The value of coop[0] is expected to be "Mike"'
);
assert.sameValue(
  coop[1], 'Rick',
  'The value of coop[1] is expected to be "Rick"'
);
assert.sameValue(
  coop[2], 'Leo',
  'The value of coop[2] is expected to be "Leo"'
);
assert(coop instanceof Coop, 'The result of evaluating (coop instanceof Coop) is expected to be true');

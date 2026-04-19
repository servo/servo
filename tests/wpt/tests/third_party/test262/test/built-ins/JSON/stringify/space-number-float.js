// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Numeric space parameter is truncated to integer part.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  6. If Type(space) is Number, then
    a. Set space to min(10, ! ToInteger(space)).
---*/

var obj = {
  a1: {
    b1: [1, 2, 3, 4],
    b2: {
      c1: 1,
      c2: 2,
    },
  },
  a2: 'a2',
};

assert.sameValue(
  JSON.stringify(obj, null, -1.99999),
  JSON.stringify(obj, null, -1)
);

assert.sameValue(
  JSON.stringify(obj, null, new Number(5.11111)),
  JSON.stringify(obj, null, 5)
);

assert.sameValue(
  JSON.stringify(obj, null, 6.99999),
  JSON.stringify(obj, null, 6)
);

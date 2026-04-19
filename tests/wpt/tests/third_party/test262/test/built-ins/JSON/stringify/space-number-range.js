// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Numeric space parameter is clamped to 0..10 range.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  6. If Type(space) is Number, then
    a. Set space to min(10, ! ToInteger(space)).
    b. If space < 1, let gap be the empty String; otherwise let gap be the
    String value containing space occurrences of the code unit 0x0020 (SPACE).
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
  JSON.stringify(obj, null, new Number(-5)),
  JSON.stringify(obj, null, 0)
);

assert.sameValue(
  JSON.stringify(obj, null, 10),
  JSON.stringify(obj, null, 100)
);

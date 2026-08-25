// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Space parameter of wrong type is silently ignored.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  8. Else,
    a. Let gap be the empty String.
features: [Symbol]
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

assert.sameValue(JSON.stringify(obj), JSON.stringify(obj, null, null));
assert.sameValue(JSON.stringify(obj), JSON.stringify(obj, null, true));
assert.sameValue(JSON.stringify(obj), JSON.stringify(obj, null, new Boolean(false)));
assert.sameValue(JSON.stringify(obj), JSON.stringify(obj, null, Symbol()));
assert.sameValue(JSON.stringify(obj), JSON.stringify(obj, null, {}));

// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Result of replacer function is stringified.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
---*/

var obj = {
  a1: {
    b1: [1, 2],
    b2: {
      c1: true,
      c2: false,
    },
  },
  a2: 'a2',
};

var replacer = function(key, value) {
  assert.sameValue(value, null);

  switch (key) {
    case '': return {a1: null, a2: null};
    case 'a1': return {b1: null, b2: null};
    case 'a2': return 'a2';

    case 'b1': return [null, null];
    case 'b2': return {c1: null, c2: null};

    case '0': return 1;
    case '1': return 2;
    case 'c1': return true;
    case 'c2': return false;
  }

  throw new Test262Error('unreachable');
};

assert.sameValue(
  JSON.stringify(null, replacer),
  JSON.stringify(obj)
);

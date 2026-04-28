// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Values that are neither strings nor numbers are ignored.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  4. If Type(replacer) is Object, then
    [...]
    4. Repeat, while k < len,
      a. Let v be ? Get(replacer, ! ToString(k)).
      [...]
      f. If item is not undefined and item is not currently an element of PropertyList, then
        i. Append item to the end of PropertyList.
features: [Proxy, Symbol]
---*/

var obj = new Proxy({}, {
  get: function(_target, key) {
    if (key !== 'toJSON') {
      throw new Test262Error();
    }
  },
});

var replacer = [
  true,
  false,
  null,
  {toString: function() { return 'toString'; }},
  Symbol(),
];

assert.sameValue(JSON.stringify(obj, replacer), '{}');

// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Converts number primitives from replacer array to strings.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  4. If Type(replacer) is Object, then
    [...]
    4. Repeat, while k < len,
      a. Let v be ? Get(replacer, ! ToString(k)).
      [...]
      d. Else if Type(v) is Number, set item to ! ToString(v).
---*/

var obj = {
  '0': 0,
  '1': 1,
  '-4': 2,
  '0.3': 3,
  '-Infinity': 4,
  'NaN': 5,
};

var replacer = [
  -0,
  1,
  -4,
  0.3,
  -Infinity,
  NaN,
];

assert.sameValue(
  JSON.stringify(obj, replacer),
  JSON.stringify(obj)
);

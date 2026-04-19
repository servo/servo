// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Undefined values in replacer array are ignored.
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
---*/

assert.sameValue(JSON.stringify({undefined: 1}, [undefined]), '{}');
assert.sameValue(JSON.stringify({key: 1, undefined: 2}, [,,,]), '{}');

var sparse = new Array(3);
sparse[1] = 'key';

assert.sameValue(
  JSON.stringify({undefined: 1, key: 2}, sparse),
  '{"key":2}'
);

// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Replacer array is deduped before Get.
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

var getCalls = 0;
var value = {
  get key() {
    getCalls += 1;
    return true;
  },
};

assert.sameValue(JSON.stringify(value, ['key', 'key']), '{"key":true}');
assert.sameValue(getCalls, 1);

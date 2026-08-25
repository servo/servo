// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Converts String objects from replacer array to primitives using ToString.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  4. If Type(replacer) is Object, then
    [...]
    4. Repeat, while k < len,
      a. Let v be ? Get(replacer, ! ToString(k)).
      [...]
      e. Else if Type(v) is Object, then
        i. If v has a [[StringData]] or [[NumberData]] internal slot,
        set item to ? ToString(v).
---*/

var str = new String('str');
str.toString = function() { return 'toString'; };
str.valueOf = function() { throw new Test262Error('should not be called'); };

var value = {
  str: 1,
  toString: 2,
  valueOf: 3,
};

assert.sameValue(JSON.stringify(value, [str]), '{"toString":2}');

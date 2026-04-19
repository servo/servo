// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Converts Number objects from replacer array to primitives using ToString.
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

var num = new Number(10);
num.toString = function() { return 'toString'; };
num.valueOf = function() { throw new Test262Error('should not be called'); };

var value = {
  10: 1,
  toString: 2,
  valueOf: 3,
};

assert.sameValue(JSON.stringify(value, [num]), '{"toString":2}');

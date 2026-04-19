// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replace
description: >
  If searchValue's Symbol.replace property is null, no error is thrown.
info: |
  String.prototype.replace ( searchValue, replaceValue )

  [...]
  2. If searchValue is neither undefined nor null, then
    a. Let replacer be ? GetMethod(searchValue, @@replace).
    b. If replacer is not undefined, then
      [...]
  [...]
  12. Return newString.

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
features: [Symbol.replace]
---*/

var searchValue = {};
searchValue[Symbol.replace] = null;
searchValue.toString = function() { return "3"; };
searchValue.valueOf = function() { throw new Test262Error("should not be called"); };

var replacer = function() { return "<foo>"; };
assert.sameValue("ab3c".replace(searchValue, replacer), "ab<foo>c");
assert.sameValue("ab3c".replace(searchValue, "<foo>"), "ab<foo>c");

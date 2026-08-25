// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  If searchValue's Symbol.replace property is null, no error is thrown.
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  [...]
  2. If searchValue is neither undefined nor null, then
    [...]
    c. Let replacer be ? GetMethod(searchValue, @@replace).
    d. If replacer is not undefined, then
      [...]
  [...]
  16. Return result.

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
features: [String.prototype.replaceAll, Symbol.replace]
---*/

var searchValue = {};
searchValue[Symbol.replace] = null;
searchValue.toString = function() { return "2"; };
searchValue.valueOf = function() { throw new Test262Error("should not be called"); };

var replacer = function() { return "<foo>"; };
assert.sameValue("a2b2c".replaceAll(searchValue, replacer), "a<foo>b<foo>c");
assert.sameValue("a2b2c".replaceAll(searchValue, "<foo>"), "a<foo>b<foo>c");

// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimend
description: >
    Abrupt completion when Symbol.toPrimitive returns an object
info: |
  Runtime Semantics: TrimString ( string, where )
  1. Let str be ? RequireObjectCoercible(string).
  2. Let S be ? ToString(str).
   ...

  ToString ( argument )
  If arguement is Object:
    1. Let primValue be ? ToPrimitive(argument, hint String).
   ...

  ToPrimitive ( input [, PreferredType ])
   ...
    d. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
    e. If exoticToPrim is not undefined, then
      i. Let result be ? Call(exoticToPrim, input, « hint »).
      ii. If Type(result) is not Object, return result.
      iii. Throw a TypeError exception.
   ...
features: [string-trimming, String.prototype.trimEnd, Symbol.toPrimitive]
---*/

var thisVal = {
  [Symbol.toPrimitive]: function() {
    return {};
  },
};

assert.sameValue(typeof String.prototype.trimEnd, 'function');
assert.throws(TypeError, function() {
  String.prototype.trimEnd.call(thisVal);
});

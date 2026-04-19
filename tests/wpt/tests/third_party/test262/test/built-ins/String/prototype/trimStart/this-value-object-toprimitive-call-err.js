// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimstart
description: >
  Abrupt completion when getting Symbol.toPrimitive method
info: |
  Runtime Semantics: TrimString ( string, where )
  1. Let str be ? RequireObjectCoercible(string).
  2. Let S be ? ToString(str).
   ...

  ToString ( argument )
  If argument is Object:
    1. Let primValue be ? ToPrimitive(argument, hint String).
   ...

  ToPrimitive ( input [, PreferredType ])
   ...
    d. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
   ...
features: [string-trimming, String.prototype.trimStart, Symbol.toPrimitive]
---*/

var thisVal = {
  get [Symbol.toPrimitive]() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  String.prototype.trimStart.call(thisVal);
});

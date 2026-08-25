// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimend
description: >
    Abrupt completion when Symbol.toPrimitive abrupt completes.
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
    e. If exoticToPrim is not undefined, then
      i. Let result be ? Call(exoticToPrim, input, « hint »).
   ...
features: [string-trimming, String.prototype.trimEnd, Symbol.toPrimitive]
---*/

var thisVal = {
  [Symbol.toPrimitive]: function() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  String.prototype.trimEnd.call(thisVal);
});

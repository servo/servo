// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimend
description: >
  Priority of Symbol[toPrimitive] when converting object to string for trimming
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
    d. Let exoticToPrim be ? GetMethod(input, @@toPrimitive)
    e. If exoticToPrim is not undefined, then
      i. Let result be ? Call(exoticToPrim, input, « hint »).
      ii. If Type(result) is not Object, return result.
   ...
features: [string-trimming, String.prototype.trimEnd, Symbol.toPrimitive]
---*/


var toPrimitiveAccessed = 0;
var toStringAccessed = 0;
var valueOfAccessed = 0;
var thisVal = {
  get [Symbol.toPrimitive]() {
    toPrimitiveAccessed += 1;
    return function() {
      return '42 ';
    };
  },
  get toString() {
    toStringAccessed += 1;
    return function() {
      return '';
    };
  },
  get valueOf() {
    valueOfAccessed += 1;
    return function() {
      return '';
    };
  },
};

// Test that thisVal[Symbol.toPrimitive] has been called.

var result = String.prototype.trimEnd.call(thisVal);

assert.sameValue(
  toPrimitiveAccessed,
  1,
  'thisVal[Symbol.toPrimitive] expected to have been accessed.'
);
assert.sameValue(
  result,
  '42',
  'thisVal[Symbol.toPrimitive] expected to have been called.'
);

// Test that thisVal.toString and thisVal.valueOf have not been accessedo

assert.sameValue(
  toStringAccessed,
  0,
  'thisVal.toString should not have been accessed.'
);
assert.sameValue(
  valueOfAccessed,
  0,
  'thisVal.valueOf should not have been accessed.'
);

// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimend
description: >
  Priority of toString when converting object to string for trimming
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
    b. Else if PreferredType is hint String, let hint be "string".
   ...
    d. Let exoticToPrim be ? GetMethod(input, @@toPrimitive)
    e. If exoticToPrim is not undefined, then
      i. Let result be ? Call(exoticToPrim, input, « hint »).
      ii. If Type(result) is not Object, return result.
      iii. Throw a TypeError exception.
    f. If hint is "default", set hint to "number".
    g. Return ? OrdinaryToPrimitive(input, hint).
   ...

  OrdinaryToPrimitive( O, hint )
   ...
    3. If hint is "string", then
      a. Let methodNames be « "toString", "valueOf" ».
   ...
    5. For each name in methodNames in List order, do
      a. Let method be ? Get(O, name).
      b. If IsCallable(method) is true, then
        i. Let result be ? Call(method, O).
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
    return undefined;
  },
  get toString() {
    toStringAccessed += 1;
    return function() {
      return '42 ';
    };
  },
  get valueOf() {
    valueOfAccessed += 1;
    return function() {
      return '';
    };
  },
};

// Test that toString is called when Symbol.toPrimitive is undefined.

var result = String.prototype.trimEnd.call(thisVal)

assert.sameValue(
  toPrimitiveAccessed,
  1,
  'thisVal.toString expected to have been accessed.'
);
assert.sameValue(
  result,
  '42',
  'thisVal.toString expected to have been called.'
);

// Test that thisVal[toPrimitive] has been accessed.

assert.sameValue(
  toPrimitiveAccessed,
  1,
  'thisVal[Symbol.toPrimitive should have been accessed.'
);

// Test that thisVal.valueOf has not been accessed.

assert.sameValue(
  valueOfAccessed,
  0,
  'thisVal.valueOf should not have been accessed.'
);

// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string-exotic-objects-getownproperty-p
description: >
  Property descriptor for numeric "own" properties of an exotic String object
info: |
  [...]
  12. Let resultStr be a String value of length 1, containing one code unit
      from str, specifically the code unit at index index.
  13. Return a PropertyDescriptor{[[Value]]: resultStr, [[Writable]]: false,
      [[Enumerable]]: true, [[Configurable]]: false}. 
includes: [propertyHelper.js]
---*/

var str = new String('abc');

assert.sameValue(str[0], 'a');
verifyProperty(str, '0', {
  writable: false,
  enumerable: true,
  configurable: false,
});

assert.sameValue(str[1], 'b');
verifyProperty(str, '1', {
  writable: false,
  enumerable: true,
  configurable: false,
});

assert.sameValue(str[2], 'c');
verifyProperty(str, '2', {
  writable: false,
  enumerable: true,
  configurable: false,
});

// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split (separator, limit) returns an Array object into which substrings of the result of converting this object to a string have
    been stored. The substrings are determined by searching from left to right for occurrences of
    separator; these occurrences are not part of any substring in the returned array, but serve to divide up
    the string value. The value of separator may be a string of any length or it may be a RegExp object
es5id: 15.5.4.14_A2_T36
description: Call split(1,"boo"), instance is Number
---*/

var __instance = new Number(100111122133144155);

Number.prototype.split = String.prototype.split;

var __split = __instance.split(1, "boo");

var __expected = [];

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(
  __split.length,
  __expected.length,
  'The value of __split.length is expected to equal the value of __expected.length'
);

assert.sameValue(__split[0], __expected[0], 'The value of __split[0] is expected to equal the value of __expected[0]');

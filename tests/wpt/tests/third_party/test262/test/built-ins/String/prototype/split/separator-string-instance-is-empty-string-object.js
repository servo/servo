// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split (separator, limit) returns an Array object into which substrings of the result of converting this object to a string have
    been stored. The substrings are determined by searching from left to right for occurrences of
    separator; these occurrences are not part of any substring in the returned array, but serve to divide up
    the string value. The value of separator may be a string of any length or it may be a RegExp object
es5id: 15.5.4.14_A2_T15
description: Separator string, instance is empty String object
---*/

var __string = new String();

var __split = __string.split("");

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 0, 'The value of __split.length is 0');
assert.sameValue(__split[0], undefined, 'The value of __split[0] is expected to equal `undefined`');

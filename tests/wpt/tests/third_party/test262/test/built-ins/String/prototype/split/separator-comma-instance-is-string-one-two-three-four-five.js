// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split (separator, limit) returns an Array object into which substrings of the result of converting this object to a string have
    been stored. The substrings are determined by searching from left to right for occurrences of
    separator; these occurrences are not part of any substring in the returned array, but serve to divide up
    the string value. The value of separator may be a string of any length or it may be a RegExp object
es5id: 15.5.4.14_A2_T1
description: Separator comma, instance is String("one,two,three,four,five")
---*/

var __string = new String("one,two,three,four,five");

var __split = __string.split(",");

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 5, 'The value of __split.length is 5');
assert.sameValue(__split[0], "one", 'The value of __split[0] is "one"');
assert.sameValue(__split[1], "two", 'The value of __split[1] is "two"');
assert.sameValue(__split[2], "three", 'The value of __split[2] is "three"');
assert.sameValue(__split[3], "four", 'The value of __split[3] is "four"');
assert.sameValue(__split[4], "five", 'The value of __split[4] is "five"');

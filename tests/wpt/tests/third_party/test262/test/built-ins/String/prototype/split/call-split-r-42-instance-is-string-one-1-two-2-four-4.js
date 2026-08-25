// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split (separator, limit) returns an Array object into which substrings of the result of converting this object to a string have
    been stored. The substrings are determined by searching from left to right for occurrences of
    separator; these occurrences are not part of any substring in the returned array, but serve to divide up
    the string value. The value of separator may be a string of any length or it may be a RegExp object
es5id: 15.5.4.14_A2_T12
description: Call split("r-42"), instance is String("one-1 two-2 four-4")
---*/

var __string = new String("one-1 two-2 four-4");

var __split = __string.split("r-42");

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 1, 'The value of __split.length is 1');
assert.sameValue(__split[0], "one-1 two-2 four-4", 'The value of __split[0] is "one-1 two-2 four-4"');

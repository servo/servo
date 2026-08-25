// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split() returns an Array object with:
    i) length equaled to 1,
    ii) [[Get]](0) equaled to the result of converting this object to a string
es5id: 15.5.4.14_A3_T11
description: Instance is String(" ")
---*/

var __string = new String(" ");

var __split = __string.split();

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 1, 'The value of __split.length is 1');
assert.sameValue(__split[0], " ", 'The value of __split[0] is " "');

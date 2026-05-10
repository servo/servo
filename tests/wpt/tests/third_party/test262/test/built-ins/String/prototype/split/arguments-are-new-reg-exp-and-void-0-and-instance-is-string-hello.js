// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split (separator, limit) returns an Array object into which substrings of the result of converting this object to a string have
    been stored. If separator is a regular expression then
    inside of SplitMatch helper the [[Match]] method of R is called giving it the arguments corresponding
es5id: 15.5.4.14_A4_T16
description: >
    Arguments are new RegExp and void 0, and instance is
    String("hello")
---*/

var __string = new String("hello");

var __re = new RegExp;

var __split = __string.split(__re, void 0);

var __expected = ["h", "e", "l", "l", "o"];

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

//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
for (var index = 0; index < __expected.length; index++) {
  assert.sameValue(
    __split[index],
    __expected[index],
    'The value of __split[index] is expected to equal the value of __expected[index]'
  );
}
//
//////////////////////////////////////////////////////////////////////////////

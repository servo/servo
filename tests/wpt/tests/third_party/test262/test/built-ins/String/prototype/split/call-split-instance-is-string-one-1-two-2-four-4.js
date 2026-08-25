// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split (separator, limit) returns an Array object into which substrings of the result of converting this object to a string have
    been stored. The substrings are determined by searching from left to right for occurrences of
    separator; these occurrences are not part of any substring in the returned array, but serve to divide up
    the string value. The value of separator may be a string of any length or it may be a RegExp object
es5id: 15.5.4.14_A2_T6
description: Call split(''), instance is String("one-1 two-2 four-4")
---*/

var __string = new String("one-1 two-2 three-3");

var __split = __string.split('');

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(
  __split.length,
  __string.length,
  'The value of __split.length is expected to equal the value of __string.length'
);

//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
for (var i = 0; i < __string.length; i++) {
  assert.sameValue(
    __split[i],
    __string.charAt(i),
    'The value of __split[i] must return the same value returned by __string.charAt(i)'
  );
}
//
//////////////////////////////////////////////////////////////////////////////

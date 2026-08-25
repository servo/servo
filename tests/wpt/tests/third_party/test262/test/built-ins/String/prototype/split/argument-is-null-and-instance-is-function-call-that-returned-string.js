// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split(separator, limit):
    i) can be transferred to other kinds of objects for use as a method.
    separator and limit can be any kinds of object since:
    ii) if separator is not RegExp ToString(separator) performs and
    iii) ToInteger(limit) performs
es5id: 15.5.4.14_A1_T5
description: >
    Argument is null, and instance is function call that returned
    string
---*/

//since ToString(null) evaluates to "null" split(null) evaluates to split("null",0)
var __split = function() {
  return "gnulluna"
}().split(null);

assert.sameValue(typeof __split, "object", 'The value of `typeof __split` is "object"');

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 2, 'The value of __split.length is 2');
assert.sameValue(__split[0], "g", 'The value of __split[0] is "g"');
assert.sameValue(__split[1], "una", 'The value of __split[1] is "una"');

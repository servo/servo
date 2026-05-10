// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split(separator, limit):
    i) can be transferred to other kinds of objects for use as a method.
    separator and limit can be any kinds of object since:
    ii) if separator is not RegExp ToString(separator) performs and
    iii) ToInteger(limit) performs
es5id: 15.5.4.14_A1_T17
description: Argument is regexp, and instance is Number
---*/

var __re = /\u0037\u0037/g;

Number.prototype.split = String.prototype.split;

var __split = (6776767677.006771122677555).split(__re);

assert.sameValue(typeof __split, "object", 'The value of `typeof __split` is "object"');

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 4, 'The value of __split.length is 4');
assert.sameValue(__split[0], "6", 'The value of __split[0] is "6"');
assert.sameValue(__split[1], "67676", 'The value of __split[1] is "67676"');
assert.sameValue(__split[2], ".006", 'The value of __split[2] is ".006"');
assert.sameValue(__split[3], "1", 'The value of __split[3] is "1"');

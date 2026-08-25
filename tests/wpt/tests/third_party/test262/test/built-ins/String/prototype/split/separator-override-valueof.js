// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split(separator, limit):
    i) can be transferred to other kinds of objects for use as a method.
    separator and limit can be any kinds of object since:
    ii) if separator is not RegExp ToString(separator) performs and
    iii) ToInteger(limit) performs
es5id: 15.5.4.14_A1_T9
description: >
    Argument is function(){}(), and instance is String(object), object
    have overrided toString and valueOf functions
---*/

var __obj = {
  valueOf: function() {},
  toString: void 0
};

//since ToString(undefined) evaluates to "undefined"
var __split = new String(__obj).split(function() {}());

assert.sameValue(typeof __split, "object", 'The value of `typeof __split` is "object"');

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 1, 'The value of __split.length is 1');
assert.sameValue(__split[0], "undefined", 'The value of __split[0] is "undefined"');

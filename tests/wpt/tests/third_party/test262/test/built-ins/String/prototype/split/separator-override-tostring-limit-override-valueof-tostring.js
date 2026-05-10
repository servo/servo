// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split(separator, limit):
    i) can be transferred to other kinds of objects for use as a method.
    separator and limit can be any kinds of object since:
    ii) if separator is not RegExp ToString(separator) performs and
    iii) ToInteger(limit) performs
es5id: 15.5.4.14_A1_T13
description: >
    Arguments are objects, and instance is string.  First object have
    overrided toString function.  Second object have overrided valueOf
    and toString functions
---*/

var __obj = {
  toString: function() {
    return "\u0042\u0042";
  }
}
var __obj2 = {
  valueOf: function() {
    return {};
  },
  toString: function() {
    return "2";
  }
}

var __split = "ABB\u0041BABAB\u0042cc^^\u0042Bvv%%B\u0042xxx".split(__obj, __obj2);

assert.sameValue(typeof __split, "object", 'The value of `typeof __split` is "object"');

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 2, 'The value of __split.length is 2');
assert.sameValue(__split[0], "A", 'The value of __split[0] is "A"');
assert.sameValue(__split[1], "ABABA", 'The value of __split[1] is "ABABA"');

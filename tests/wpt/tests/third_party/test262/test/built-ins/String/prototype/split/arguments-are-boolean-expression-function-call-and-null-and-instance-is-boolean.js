// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split(separator, limit):
    i) can be transferred to other kinds of objects for use as a method.
    separator and limit can be any kinds of object since:
    ii) if separator is not RegExp ToString(separator) performs and
    iii) ToInteger(limit) performs
es5id: 15.5.4.14_A1_T2
description: >
    Arguments are boolean expression, function call and null, and
    instance is Boolean
---*/

var __instance = new Boolean;

__instance.split = String.prototype.split;

var __split = __instance.split("A" !== "\u0041", function() {
  return 0;
}(), null);

assert.sameValue(typeof __split, "object", 'The value of `typeof __split` is "object"');

assert.sameValue(
  __split.constructor,
  Array,
  'The value of __split.constructor is expected to equal the value of Array'
);

assert.sameValue(__split.length, 0, 'The value of __split.length is 0');

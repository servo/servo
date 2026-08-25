// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    String.prototype.split(separator, limit):
    i) can be transferred to other kinds of objects for use as a method.
    separator and limit can be any kinds of object since:
    ii) if separator is not RegExp ToString(separator) performs and
    iii) ToInteger(limit) performs
es5id: 15.5.4.14_A1_T3
description: Checking by using eval
---*/

var split = String.prototype.split.bind(this);

var __obj__lim = {
  valueOf: function() {
    return 5;
  }
};

try {
  toString = Object.prototype.toString;
} catch (e) {;
}

//Checks are only valid if we can overwrite the global object's toString method
//(which ES5 doesn't even require to exist)
if (toString === Object.prototype.toString) {
  var __class__ = toString();

  var __split = split(eval("\"[\""), __obj__lim);

  assert.sameValue(typeof __split, "object", 'The value of `typeof __split` is "object"');

  assert.sameValue(
    __split.constructor,
    Array,
    'The value of __split.constructor is expected to equal the value of Array'
  );

  assert.sameValue(__split.length, 2, 'The value of __split.length is 2');
  assert.sameValue(__split[1].substring(0, 6), "object", '__split[1].substring(0, 6) must return "object"');
}

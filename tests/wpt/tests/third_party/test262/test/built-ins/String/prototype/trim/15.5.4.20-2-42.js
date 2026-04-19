// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-42
description: >
    String.prototype.trim - TypeError exception was thrown  when
    'this' is an object that both toString and valueOf wouldn't return
    primitive value.
---*/

var toStringAccessed = false;
var valueOfAccessed = false;
var obj = {
  toString: function() {
    toStringAccessed = true;
    return {};
  },
  valueOf: function() {
    valueOfAccessed = true;
    return {};
  }
};
assert.throws(TypeError, function() {
  String.prototype.trim.call(obj);
});
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert(toStringAccessed, 'toStringAccessed !== true');

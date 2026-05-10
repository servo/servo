// Copyright (C) 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.tostring
description: >
  Throws a TypeError if called on neither String primitive nor String object
info: |
  String.prototype.toString ( )

  1. Return ? thisStringValue(this value).

  thisStringValue ( value )

  [...]
  3. Throw a TypeError exception.
---*/

var toString = String.prototype.toString;

assert.throws(TypeError, function() {
  toString.call(false);
});

assert.throws(TypeError, function() {
  toString.call(1);
});

assert.throws(TypeError, function() {
  toString.call(null);
});

assert.throws(TypeError, function() {
  toString.call();
});

assert.throws(TypeError, function() {
  toString.call(Symbol('desc'));
});

assert.throws(TypeError, function() {
  toString.call({
    toString: function() {
      return 'str';
    },
  });
});

assert.throws(TypeError, function() {
  toString.call(['s', 't', 'r']);
});

assert.throws(TypeError, function() {
  ''.concat({toString: toString});
});

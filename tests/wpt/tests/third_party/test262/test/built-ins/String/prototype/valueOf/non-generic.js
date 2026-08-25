// Copyright (C) 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.valueof
description: >
  Throws a TypeError if called on neither String primitive nor String object
info: |
  String.prototype.valueOf ( )

  1. Return ? thisStringValue(this value).

  thisStringValue ( value )

  [...]
  3. Throw a TypeError exception.
---*/

var valueOf = String.prototype.valueOf;

assert.throws(TypeError, function() {
  valueOf.call(true);
});

assert.throws(TypeError, function() {
  valueOf.call(-0);
});

assert.throws(TypeError, function() {
  valueOf.call(null);
});

assert.throws(TypeError, function() {
  valueOf.call();
});

assert.throws(TypeError, function() {
  valueOf.call(Symbol('desc'));
});

assert.throws(TypeError, function() {
  valueOf.call({
    toString: function() {
      return 'str';
    },
  });
});

assert.throws(TypeError, function() {
  valueOf.call(['s', 't', 'r']);
});

assert.throws(TypeError, function() {
  'str' + {valueOf: valueOf};
});

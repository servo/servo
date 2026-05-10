// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.tostring
description: >
  Throws a TypeError if called on neither String primitive nor String object
  (honoring the Realm of the current execution context)
info: |
  String.prototype.toString ( )

  1. Return ? thisStringValue(this value).

  thisStringValue ( value )

  [...]
  3. Throw a TypeError exception.
features: [cross-realm]
---*/

var other = $262.createRealm().global;
var otherToString = other.String.prototype.toString;

assert.throws(other.TypeError, function() {
  otherToString.call(true);
});

assert.throws(other.TypeError, function() {
  otherToString.call(0);
});

assert.throws(other.TypeError, function() {
  otherToString.call(null);
});

assert.throws(other.TypeError, function() {
  otherToString.call();
});

assert.throws(other.TypeError, function() {
  otherToString.call(Symbol('desc'));
});

assert.throws(other.TypeError, function() {
  otherToString.call({
    valueOf: function() {
      return 'str';
    },
  });
});

assert.throws(other.TypeError, function() {
  otherToString.call([1]);
});

assert.throws(other.TypeError, function() {
  'str'.concat({toString: otherToString});
});

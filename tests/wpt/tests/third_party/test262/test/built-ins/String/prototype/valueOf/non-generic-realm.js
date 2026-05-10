// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.valueof
description: >
  Throws a TypeError if called on neither String primitive nor String object
  (honoring the Realm of the current execution context)
info: |
  String.prototype.valueOf ( )

  1. Return ? thisStringValue(this value).

  thisStringValue ( value )

  [...]
  3. Throw a TypeError exception.
features: [cross-realm]
---*/

var other = $262.createRealm().global;
var otherValueOf = other.String.prototype.valueOf;

assert.throws(other.TypeError, function() {
  otherValueOf.call(false);
});

assert.throws(other.TypeError, function() {
  otherValueOf.call(-1);
});

assert.throws(other.TypeError, function() {
  otherValueOf.call(null);
});

assert.throws(other.TypeError, function() {
  otherValueOf.call();
});

assert.throws(other.TypeError, function() {
  otherValueOf.call(Symbol('desc'));
});

assert.throws(other.TypeError, function() {
  otherValueOf.call({
    valueOf: function() {
      return '';
    },
  });
});

assert.throws(other.TypeError, function() {
  otherValueOf.call([3]);
});

assert.throws(other.TypeError, function() {
  '' + {valueOf: otherValueOf};
});

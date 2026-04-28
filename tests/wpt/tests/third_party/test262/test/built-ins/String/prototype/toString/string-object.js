// Copyright (C) 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.tostring
description: >
  If called on a String object, returns [[StringData]] slot
info: |
  String.prototype.toString ( )

  1. Return ? thisStringValue(this value).

  thisStringValue ( value )

  [...]
  2. If Type(value) is Object and value has a [[StringData]] internal slot, then
    a. Let s be value.[[StringData]].
    b. Assert: Type(s) is String.
    c. Return s.
---*/

var toString = String.prototype.toString;

assert.sameValue(Object('str').toString(), 'str');
assert.sameValue(toString.call(new String('')), '');
assert.sameValue('a'.concat(Object('b')), 'ab');

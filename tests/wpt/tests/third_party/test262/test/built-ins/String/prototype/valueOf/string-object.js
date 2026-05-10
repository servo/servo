// Copyright (C) 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.valueof
description: >
  If called on a String object, returns [[StringData]] slot
info: |
  String.prototype.valueOf ( )

  1. Return ? thisStringValue(this value).

  thisStringValue ( value )

  [...]
  2. If Type(value) is Object and value has a [[StringData]] internal slot, then
    a. Let s be value.[[StringData]].
    b. Assert: Type(s) is String.
    c. Return s.
---*/

var valueOf = String.prototype.valueOf;

assert.sameValue(Object('').valueOf(), '');
assert.sameValue(valueOf.call(new String('str')), 'str');
assert.sameValue('a' + new String('b'), 'ab');

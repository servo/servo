// Copyright (C) 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.tostring
description: >
  If called on String primitive, returns it
info: |
  String.prototype.toString ( )

  1. Return ? thisStringValue(this value).

  thisStringValue ( value )

  1. If Type(value) is String, return value.
---*/

var toString = String.prototype.toString;

assert.sameValue(''.toString(), '');
assert.sameValue(toString.call('str'), 'str');

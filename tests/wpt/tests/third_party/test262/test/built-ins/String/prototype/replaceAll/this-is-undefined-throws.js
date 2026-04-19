// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  Throws TypeError when `this` is undefined
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  1. Let O be RequireObjectCoercible(this value).
  ...

  RequireObjectCoercible ( argument )

  - Undefined: Throw a TypeError exception.
  - Null: Throw a TypeError exception.
features: [String.prototype.replaceAll]
---*/

assert.sameValue(
  typeof String.prototype.replaceAll,
  'function',
  'function must exist'
);

assert.throws(TypeError, function() {
  String.prototype.replaceAll.call(undefined);
});

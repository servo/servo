// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype-@@tostringtag
description: >
  Check that the initial value of the property is "Intl.NumberFormat" and that any changes
  made by reconfiguring are reflected.
---*/

assert.sameValue(Intl.NumberFormat.prototype[Symbol.toStringTag], 'Intl.NumberFormat');
assert.sameValue(
  Object.prototype.toString.call(new Intl.NumberFormat()),
  '[object Intl.NumberFormat]'
);

Object.defineProperty(Intl.NumberFormat.prototype, Symbol.toStringTag, {
  value: 'Alpha'
});

assert.sameValue(Intl.NumberFormat.prototype[Symbol.toStringTag], 'Alpha');
assert.sameValue(
  Object.prototype.toString.call(new Intl.NumberFormat()),
  '[object Alpha]'
);

delete Intl.NumberFormat.prototype[Symbol.toStringTag];

assert.sameValue(Intl.NumberFormat.prototype[Symbol.toStringTag], undefined);
assert.sameValue(
  Object.prototype.toString.call(new Intl.NumberFormat()),
  '[object Object]'
);

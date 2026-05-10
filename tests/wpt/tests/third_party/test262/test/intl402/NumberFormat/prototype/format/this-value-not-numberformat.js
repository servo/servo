// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: >
  Tests that Intl.NumberFormat.prototype.format throws a TypeError
  if called on a non-object value or an object that hasn't been
  initialized as a NumberFormat.
---*/

const invalidTargets = [undefined, null, true, 0, 'NumberFormat', [], {}, Symbol()];
const fn = Object.getOwnPropertyDescriptor(Intl.NumberFormat.prototype, 'format').get;

invalidTargets.forEach(target => {
  assert.throws(
    TypeError,
    () => fn.call(target),
    `Calling format getter on ${String(target)} should throw a TypeError.`
  );
});

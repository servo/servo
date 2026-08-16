// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  When the receiver already has an own "stack" data property, the setter
  performs a [[Set]] on it (preserving its current attributes).
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If v is not a String, throw a TypeError exception.
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).
  5. Return undefined.

  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  [...]
  3. Let desc be ? this.[[GetOwnProperty]](p).
  4. If desc is undefined, then
    a. Perform ? CreateDataPropertyOrThrow(this, p, v).
  5. Else,
    a. Perform ? Set(this, p, v, true).
  [...]
includes: [propertyHelper.js, nativeErrors.js]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var err = new Ctor('msg');
  Object.defineProperty(err, 'stack', {
    value: 'original',
    writable: true,
    enumerable: false,
    configurable: false,
  });

  set.call(err, 'updated');

  // The existing descriptor is preserved; only the value changes (per [[Set]]).
  verifyProperty(err, 'stack', {
    value: 'updated',
    writable: true,
    enumerable: false,
    configurable: false,
  });
}

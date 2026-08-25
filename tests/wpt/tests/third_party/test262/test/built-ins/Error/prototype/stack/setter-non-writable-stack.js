// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  When the receiver has an own non-writable "stack" data property, the setter
  throws a TypeError because the underlying [[Set]] is invoked with Throw=true.
info: |
  set Error.prototype.stack

  [...]
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).

  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  [...]
  3. Let desc be ? this.[[GetOwnProperty]](p).
  [...]
  5. Else,
    a. Perform ? Set(this, p, v, true).
includes: [propertyHelper.js, nativeErrors.js]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];

  var err = new Ctor('msg');
  Object.defineProperty(err, 'stack', {
    value: 'original',
    writable: false,
    enumerable: false,
    configurable: true,
  });

  assert.throws(TypeError, function () {
    set.call(err, 'updated');
  }, Ctor.name + ': non-writable own "stack"');

  verifyProperty(err, 'stack', {
    value: 'original',
    writable: false,
    enumerable: false,
    configurable: true,
  });

  // Frozen instance with an own "stack" data property: still throws.
  var frozen = new Ctor('msg');
  Object.defineProperty(frozen, 'stack', {
    value: 'frozen-original',
    writable: false,
    enumerable: false,
    configurable: false,
  });
  Object.preventExtensions(frozen);

  assert.throws(TypeError, function () {
    set.call(frozen, 'updated');
  }, Ctor.name + ': frozen with non-writable own "stack"');

  verifyProperty(frozen, 'stack', {
    value: 'frozen-original',
    writable: false,
    enumerable: false,
    configurable: false,
  });
}

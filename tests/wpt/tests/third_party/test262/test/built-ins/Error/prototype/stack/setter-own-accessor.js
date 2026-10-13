// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  When the receiver has an own "stack" accessor property, the setter performs
  [[Set]] which invokes the own setter (or throws if the accessor has no
  setter).
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
includes: [nativeErrors.js]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];

  // (a) Own accessor with a setter: the own setter receives v.
  var observed;
  var withSetter = new Ctor('msg');
  Object.defineProperty(withSetter, 'stack', {
    get: function () { return observed; },
    set: function (v) { observed = v; },
    enumerable: false,
    configurable: true,
  });

  set.call(withSetter, 'sentinel');
  assert.sameValue(observed, 'sentinel', Ctor.name + ': own setter received the value');

  // (b) Own accessor with no setter: Set with Throw=true throws TypeError.
  var withoutSetter = new Ctor('msg');
  Object.defineProperty(withoutSetter, 'stack', {
    get: function () { return 'getter-only'; },
    enumerable: false,
    configurable: true,
  });

  assert.throws(TypeError, function () {
    set.call(withoutSetter, 'sentinel');
  }, Ctor.name + ': own accessor without a setter');
}

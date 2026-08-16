// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-error-instances
description: >
  A freshly-constructed Error instance has no own "stack" property; the
  property is reached via the inherited accessor on Error.prototype.
info: |
  Properties of Error Instances

  Error instances are ordinary objects that inherit properties from the Error
  prototype object and have an [[ErrorData]] internal slot whose value is
  undefined.

  Error.prototype.stack is an accessor property with attributes
  { [[Enumerable]]: false, [[Configurable]]: true }.
includes: [nativeErrors.js]
features: [error-stack-accessor]
---*/

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var err = new Ctor('msg');

  assert.sameValue(
    Object.prototype.hasOwnProperty.call(err, 'stack'),
    false,
    Ctor.name + ': hasOwnProperty("stack") is false'
  );

  assert.sameValue(
    Object.getOwnPropertyDescriptor(err, 'stack'),
    undefined,
    Ctor.name + ': getOwnPropertyDescriptor returns undefined'
  );

  // For NativeErrors, the immediate prototype is e.g. TypeError.prototype,
  // which does NOT have its own "stack" property; the accessor lives only on
  // Error.prototype, two links up.
  if (Ctor !== Error) {
    assert.sameValue(
      Object.getOwnPropertyDescriptor(Object.getPrototypeOf(err), 'stack'),
      undefined,
      Ctor.name + ': stack is not an own property of NativeError.prototype'
    );
  }

  var desc = Object.getOwnPropertyDescriptor(Error.prototype, 'stack');
  assert.notSameValue(desc, undefined, Ctor.name + ': Error.prototype has the accessor');
  assert.sameValue(typeof desc.get, 'function', Ctor.name + ': accessor get is a function');
  assert.sameValue(typeof desc.set, 'function', Ctor.name + ': accessor set is a function');
}

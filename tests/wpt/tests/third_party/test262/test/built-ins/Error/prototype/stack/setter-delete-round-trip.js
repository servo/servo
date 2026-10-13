// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  After the setter installs an own "stack" data property, deleting it
  re-exposes the inherited accessor on Error.prototype.
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If v is not a String, throw a TypeError exception.
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).
  5. Return undefined.

  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  [...]
  4. If desc is undefined, then
    a. Perform ? CreateDataPropertyOrThrow(this, p, v).
includes: [nativeErrors.js]
features: [error-stack-accessor]
---*/

var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;
var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var err = new Ctor('msg');

  // delete on a fresh instance: nothing to remove, returns true.
  assert.sameValue(delete err.stack, true, Ctor.name + ': delete on fresh instance returns true');
  assert.sameValue(
    Object.prototype.hasOwnProperty.call(err, 'stack'),
    false,
    Ctor.name + ': still no own property after delete'
  );

  // The inherited accessor still produces a string.
  assert.sameValue(typeof get.call(err), 'string', Ctor.name + ': accessor still works');

  // After setting, an own data property is installed.
  set.call(err, 'sentinel');
  assert.sameValue(
    Object.prototype.hasOwnProperty.call(err, 'stack'),
    true,
    Ctor.name + ': own property installed after set'
  );
  assert.sameValue(err.stack, 'sentinel', Ctor.name + ': data property shadows accessor');

  // delete removes the own data property.
  assert.sameValue(delete err.stack, true, Ctor.name + ': delete removes own data property');
  assert.sameValue(
    Object.prototype.hasOwnProperty.call(err, 'stack'),
    false,
    Ctor.name + ': own property removed'
  );

  // The inherited accessor is exposed again, and [[ErrorData]] still drives it.
  assert.sameValue(typeof err.stack, 'string', Ctor.name + ': inherited accessor re-exposed');
  assert.sameValue(typeof get.call(err), 'string', Ctor.name + ': accessor still returns a string');
}

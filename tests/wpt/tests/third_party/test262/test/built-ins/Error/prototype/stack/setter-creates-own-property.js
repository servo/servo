// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  When the receiver does not have an own "stack" property, the setter creates
  one as a writable, enumerable, configurable data property whose value is v.
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
  [...]
includes: [nativeErrors.js, propertyHelper.js]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

var errors = [];
for (var i = 0; i < allErrorConstructors.length; ++i) {
  var Ctor = allErrorConstructors[i];
  errors.push([Ctor.name, makeNativeError(Ctor, true)]);
}

for (var i = 0; i < errors.length; ++i) {
  var name = errors[i][0];
  var err = errors[i][1];

  assert.sameValue(
    Object.prototype.hasOwnProperty.call(err, 'stack'),
    false,
    name + ': precondition: instance has no own "stack" property at construction'
  );

  var result = set.call(err, 'sentinel-' + name);
  assert.sameValue(result, undefined, name + ': setter returns undefined');

  verifyProperty(err, 'stack', {
    value: 'sentinel-' + name,
    writable: true,
    enumerable: true,
    configurable: true,
  });
}

// The same applies to a plain object that lacks [[ErrorData]]: the setter
// does not check for the internal slot before installing the property.
var plain = {};
set.call(plain, 'on-plain');
verifyProperty(plain, 'stack', {
  value: 'on-plain',
  writable: true,
  enumerable: true,
  configurable: true,
});

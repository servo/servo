// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  When the receiver lacks an own "stack" property and has any integrity level
  applied (preventExtensions, seal, freeze), the setter throws a TypeError
  because CreateDataPropertyOrThrow cannot add a property to a non-extensible
  object.
info: |
  set Error.prototype.stack

  [...]
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).

  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  [...]
  3. Let desc be ? this.[[GetOwnProperty]](p).
  4. If desc is undefined, then
    a. Perform ? CreateDataPropertyOrThrow(this, p, v).

  CreateDataPropertyOrThrow ( O, P, V )

  [...]
  3. Let success be ? CreateDataProperty(O, P, V).
  4. If success is false, throw a TypeError exception.
includes: [nativeErrors.js]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

var integrityLevels = [
  { name: 'preventExtensions', fn: Object.preventExtensions },
  { name: 'seal', fn: Object.seal },
  { name: 'freeze', fn: Object.freeze }
];

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];

  for (var j = 0; j < integrityLevels.length; ++j) {
    var level = integrityLevels[j];
    var label = Ctor.name + '/' + level.name;

    var err = new Ctor('msg');
    assert.sameValue(
      Object.prototype.hasOwnProperty.call(err, 'stack'),
      false,
      label + ': precondition: instance has no own "stack" property at construction'
    );

    level.fn(err);

    assert.throws(TypeError, function () {
      set.call(err, 'sentinel');
    }, label + ': instance without own "stack" rejects setter');

    assert.sameValue(
      Object.prototype.hasOwnProperty.call(err, 'stack'),
      false,
      label + ': no own "stack" property was created'
    );
  }
}

// Same behavior on plain objects at each integrity level.
for (var k = 0; k < integrityLevels.length; ++k) {
  var lvl = integrityLevels[k];
  var obj = lvl.fn({});
  assert.throws(TypeError, function () {
    set.call(obj, 'sentinel');
  }, lvl.name + ' plain object without own "stack"');
}

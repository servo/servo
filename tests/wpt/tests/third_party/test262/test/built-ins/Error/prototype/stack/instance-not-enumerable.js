// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype-stack
description: >
  "stack" is a non-enumerable accessor on Error.prototype, and Error instances
  do not have an own "stack" property, so "stack" is not visible via
  Object.keys, for-in, propertyIsEnumerable, or JSON.stringify.
info: |
  Error.prototype.stack is an accessor property with attributes
  { [[Enumerable]]: false, [[Configurable]]: true }.

  Properties of Error Instances

  Error instances are ordinary objects that inherit properties from the Error
  prototype object and have an [[ErrorData]] internal slot whose value is
  undefined.
includes: [nativeErrors.js]
features: [error-stack-accessor]
---*/

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var err = new Ctor('msg');

  assert.sameValue(
    Object.keys(err).indexOf('stack'),
    -1,
    Ctor.name + ': Object.keys does not include "stack"'
  );

  assert.sameValue(
    Object.getOwnPropertyNames(err).indexOf('stack'),
    -1,
    Ctor.name + ': getOwnPropertyNames does not include "stack"'
  );

  var sawStack = false;
  for (var key in err) {
    if (key === 'stack') {
      sawStack = true;
    }
  }
  assert.sameValue(sawStack, false, Ctor.name + ': for-in does not yield "stack"');

  assert.sameValue(
    Object.prototype.propertyIsEnumerable.call(err, 'stack'),
    false,
    Ctor.name + ': propertyIsEnumerable returns false'
  );

  assert.sameValue(
    Object.prototype.propertyIsEnumerable.call(Error.prototype, 'stack'),
    false,
    Ctor.name + ': Error.prototype.propertyIsEnumerable("stack") is false'
  );

  // JSON.stringify omits non-enumerable / non-own properties; a fresh Error
  // instance has no enumerable own properties at all.
  assert.sameValue(
    JSON.stringify(err),
    '{}',
    Ctor.name + ': JSON.stringify output is empty object'
  );

  // Object.assign reads only enumerable own properties; the inherited
  // accessor is not own, so "stack" is not copied.
  var copy = Object.assign({}, err);
  assert.sameValue(
    Object.prototype.hasOwnProperty.call(copy, 'stack'),
    false,
    Ctor.name + ': Object.assign({}, err) does not copy "stack"'
  );
}

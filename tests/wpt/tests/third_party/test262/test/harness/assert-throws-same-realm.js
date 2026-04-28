// Copyright (C) 2021 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions that throw instances of the realm specified constructor function
    do not satisfy the assertion with cross realms collisions.
---*/

var intrinsicTypeError = TypeError; 
var threw = false;
var realmGlobal = $262.createRealm().global;

try {
  assert.throws(TypeError, function() {
    throw new realmGlobal.TypeError();
  });
} catch (err) {
  threw = true;
  if (err.constructor !== Test262Error) {
    throw new Error(
      'Expected a Test262Error but a "' + err.constructor.name + 
      '" was thrown.'
    );
  }
}

if (threw === false) {
  throw new Error('Expected a Test262Error, but no error was thrown.');
}

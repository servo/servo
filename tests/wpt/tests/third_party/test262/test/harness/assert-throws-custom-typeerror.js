// Copyright (C) 2021 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions that throw instances of the specified constructor function
    satisfy the assertion, without collision with error constructors of the
    same name.
---*/

var intrinsicTypeError = TypeError; 
var threw = false;

(function() {
  function TypeError() {}

  assert.throws(TypeError, function() {
    throw new TypeError();
  }, 'Throws an instance of the matching custom TypeError');

  try {
    assert.throws(intrinsicTypeError, function() {
      throw new TypeError();
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

  threw = false;

  try {
    assert.throws(TypeError, function() {
      throw new intrinsicTypeError();
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
})();

// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    We overload valueOf method so it return non Primitive value
    Thus [[DefaultValue]] must return Object.toString() value
es5id: 8.12.8_A3
description: >
    Try to overload toString method, that returned Primitive, and
    valueOf method, that returned new Object
---*/

try
{
  var __obj = {
    toString: function() {
      return "1"
    },
    valueOf: function() {
      return new Object();
    }
  }

  assert.sameValue(
    Number(__obj),
    1,
    'Number("{toString: function() {return "1"}, valueOf: function() {return new Object();}}) must return 1'
  );
}
catch (e)
{
  throw new Test262Error('#1.2: var __obj = {toNumber: function() {return "1"}, valueOf: function() {return new Object();}}; Number(__obj) === 1. Actual: ' + (e));
}

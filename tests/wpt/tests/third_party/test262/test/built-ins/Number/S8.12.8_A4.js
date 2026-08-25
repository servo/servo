// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    We overload valueOf method so it return non Primitive value and toString method so it return non Primitive value too
    Thus [[DefaultValue]] must generate TypeError error
es5id: 8.12.8_A4
description: >
    Try to overload toString and valueOf methods, they returned new
    Objects
---*/

try
{
  var __obj = {
    valueOf: function() {
      return new Object;
    },
    toString: function() {
      return new Object();
    }
  }
  Number(__obj);
  throw new Test262Error('#1.1: var __obj = {valueOf:function(){return new Object;},toNumber: function() {return new Object();}}; Number(__obj) throw TypeError. Actual: ' + (Number(__obj)));
}
catch (e)
{
  assert.sameValue(
    e instanceof TypeError,
    true,
    'The result of evaluating (e instanceof TypeError) is expected to be true'
  );
}

// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    This should generate a TypeError,
    Cause we overload toString method so it return non Primitive value
    See ECMA reference at http://bugzilla.mozilla.org/show_bug.cgi?id=167325
es5id: 8.12.8_A1
description: Try to overload toString method
---*/

try
{
  var __obj = {
    toString: function() {
      return new Object();
    }
  }
  String(__obj);
  throw new Test262Error('#1.1: var __obj = {toString: function() {return new Object();}}; String(__obj) throw TypeError. Actual: ' + (String(__obj)));
}
catch (e)
{
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1.2: var __obj = {toString: function() {return new Object();}}; String(__obj) throw TypeError. Actual: ' + (e));
  }
}

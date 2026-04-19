// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.search (regexp) returns ...
es5id: 15.5.4.12_A2_T7
description: Simple search sentence inside string
---*/

var aString = new String("test string probe");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (aString.search("string pro") !== 5) {
  throw new Test262Error('#1: var aString = new String("test string probe"); aString.search("string pro")=== 5. Actual: ' + aString.search("string pro"));
}
//
//////////////////////////////////////////////////////////////////////////////

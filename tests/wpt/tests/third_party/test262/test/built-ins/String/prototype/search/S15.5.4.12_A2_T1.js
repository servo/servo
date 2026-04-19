// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.search (regexp) returns ...
es5id: 15.5.4.12_A2_T1
description: Simple search substring inside string
---*/

var aString = new String("test string");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (aString.search("string") !== 5) {
  throw new Test262Error('#1: var aString = new String("test string"); aString.search("string")=== 5. Actual: ' + aString.search("string"));
}
//
//////////////////////////////////////////////////////////////////////////////

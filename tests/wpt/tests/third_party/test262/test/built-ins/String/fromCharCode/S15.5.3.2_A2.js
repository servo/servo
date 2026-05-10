// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.fromCharCode () returns empty string
es5id: 15.5.3.2_A2
description: Call String.fromCharCode()
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String.fromCharCode() !== "") {
  throw new Test262Error('#1: String.fromCharCode () returns empty string. Actual: ' + String.fromCharCode());
}
//
//////////////////////////////////////////////////////////////////////////////

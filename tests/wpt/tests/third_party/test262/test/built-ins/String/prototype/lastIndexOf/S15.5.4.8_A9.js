// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The String.prototype.lastIndexOf.length property does not have the
    attribute DontDelete
es5id: 15.5.4.8_A9
description: >
    Checking if deleting the String.prototype.lastIndexOf.length
    property fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.prototype.lastIndexOf.hasOwnProperty('length'))) {
  throw new Test262Error('#0: String.prototype.lastIndexOf.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.lastIndexOf.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!delete String.prototype.lastIndexOf.length) {
  throw new Test262Error('#1: delete String.prototype.lastIndexOf.length return true');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.lastIndexOf.hasOwnProperty('length')) {
  throw new Test262Error('#2: delete String.prototype.lastIndexOf.length; String.prototype.lastIndexOf.hasOwnProperty(\'length\') return false. Actual: ' + String.prototype.lastIndexOf.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

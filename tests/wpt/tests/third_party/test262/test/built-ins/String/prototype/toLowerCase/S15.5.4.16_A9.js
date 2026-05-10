// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The String.prototype.toLowerCase.length property does not have the
    attribute DontDelete
es5id: 15.5.4.16_A9
description: >
    Checking if deleting the String.prototype.toLowerCase.length
    property fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.prototype.toLowerCase.hasOwnProperty('length'))) {
  throw new Test262Error('#0: String.prototype.toLowerCase.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.toLowerCase.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!delete String.prototype.toLowerCase.length) {
  throw new Test262Error('#1: delete String.prototype.toLowerCase.length return true');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.toLowerCase.hasOwnProperty('length')) {
  throw new Test262Error('#2: delete String.prototype.toLowerCase.length; String.prototype.toLowerCase.hasOwnProperty(\'length\') return false. Actual: ' + String.prototype.toLowerCase.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

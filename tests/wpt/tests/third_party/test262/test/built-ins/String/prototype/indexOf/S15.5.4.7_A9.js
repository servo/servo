// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The String.prototype.indexOf.length property does not have the attribute
    DontDelete
es5id: 15.5.4.7_A9
description: >
    Checking if deleting the String.prototype.indexOf.length property
    fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.prototype.indexOf.hasOwnProperty('length'))) {
  throw new Test262Error('#0: String.prototype.indexOf.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.indexOf.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!delete String.prototype.indexOf.length) {
  throw new Test262Error('#1: delete String.prototype.indexOf.length raturn true');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.indexOf.hasOwnProperty('length')) {
  throw new Test262Error('#2: delete String.prototype.indexOf.length; String.prototype.indexOf.hasOwnProperty(\'length\') return false. Actual: ' + String.prototype.indexOf.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

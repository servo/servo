// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The String.prototype.concat.length property does not have the attribute
    DontDelete
es5id: 15.5.4.6_A9
description: >
    Checking if deleting the String.prototype.concat.length property
    fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.prototype.concat.hasOwnProperty('length'))) {
  throw new Test262Error('#0: String.prototype.concat.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.concat.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!delete String.prototype.concat.length) {
  throw new Test262Error('#1: delete String.prototype.concat.length return true');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.concat.hasOwnProperty('length')) {
  throw new Test262Error('#2: delete String.prototype.concat.length; String.prototype.concat.hasOwnProperty(\'length\') return false. Actual: ' + String.prototype.concat.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

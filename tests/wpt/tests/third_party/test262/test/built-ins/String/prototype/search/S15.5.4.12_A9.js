// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The String.prototype.search.length property does not have the attribute
    DontDelete
es5id: 15.5.4.12_A9
description: >
    Checking if deleting the String.prototype.search.length property
    fails
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#0
if (!(String.prototype.search.hasOwnProperty('length'))) {
  throw new Test262Error('#0: String.prototype.search.hasOwnProperty(\'length\') return true. Actual: ' + String.prototype.search.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!delete String.prototype.search.length) {
  throw new Test262Error('#1: delete String.prototype.search.length return true');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.search.hasOwnProperty('length')) {
  throw new Test262Error('#2: delete String.prototype.search.length; String.prototype.search.hasOwnProperty(\'length\') return false. Actual: ' + String.prototype.search.hasOwnProperty('length'));
}
//
//////////////////////////////////////////////////////////////////////////////

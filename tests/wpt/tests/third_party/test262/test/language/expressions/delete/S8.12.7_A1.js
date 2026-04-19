// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the [[Delete]] method of O is called with property name P,
    and If the property has the DontDelete attribute, return false
esid: sec-delete-operator-runtime-semantics-evaluation
description: Try to delete Math.E, that has the DontDelete attribute
flags: [noStrict]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (delete Math.E !== false) {
  throw new Test262Error('#1: delete Math.E === false. Actual: ' + delete Math.E);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (Math.E === undefined) {
  throw new Test262Error('#2: delete Math.E; Math.E !== undefined');
}
//
//////////////////////////////////////////////////////////////////////////////

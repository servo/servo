// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The length property of encodeURIComponent does not have the attribute
    DontDelete
esid: sec-encodeuricomponent-uricomponent
description: Checking use hasOwnProperty, delete
---*/

//CHECK#1
if (encodeURIComponent.hasOwnProperty('length') !== true) {
  throw new Test262Error('#1: encodeURIComponent.hasOwnProperty(\'length\') === true. Actual: ' + (encodeURIComponent.hasOwnProperty('length')));
}

delete encodeURIComponent.length;

//CHECK#2
if (encodeURIComponent.hasOwnProperty('length') !== false) {
  throw new Test262Error('#2: delete encodeURIComponent.length; encodeURIComponent.hasOwnProperty(\'length\') === false. Actual: ' + (encodeURIComponent.hasOwnProperty('length')));
}

//CHECK#3
if (encodeURIComponent.length === undefined) {
  throw new Test262Error('#3: delete encodeURIComponent.length; encodeURIComponent.length !== undefined');
}

// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: array.forEach can be frozen while in progress
esid: sec-array.prototype.foreach
description: Freezes array.forEach during a forEach to see if it works
---*/

function foo() {
  ['z'].forEach(function() {
    Object.freeze(Array.prototype.forEach);
  });
}
foo();

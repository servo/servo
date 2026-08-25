// Copyright 2011 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3_A1
description: >
    When calling a strict anonymous function as a  function, "this"
    should be bound to undefined.
flags: [onlyStrict]
---*/

var that = (function() { return this; })();
if (that !== undefined) {
  throw new Test262Error('#1: "this" leaked as: ' + that);
}

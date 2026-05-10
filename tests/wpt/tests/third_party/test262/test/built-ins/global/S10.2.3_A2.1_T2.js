// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Global object properties have attributes { DontEnum }
es5id: 10.2.3_A2.1_T2
description: Global execution context - Function Properties
---*/

//CHECK#1
for (var x in this) {
  if (x === 'eval') {
    throw new Test262Error("#1: 'eval' have attribute DontEnum");
  } else if (x === 'parseInt') {
    throw new Test262Error("#1: 'parseInt' have attribute DontEnum");
  } else if (x === 'parseFloat') {
    throw new Test262Error("#1: 'parseFloat' have attribute DontEnum");
  } else if (x === 'isNaN') {
    throw new Test262Error("#1: 'isNaN' have attribute DontEnum");
  } else if (x === 'isFinite') {
    throw new Test262Error("#1: 'isFinite' have attribute DontEnum");
  } else if (x === 'decodeURI') {
    throw new Test262Error("#1: 'decodeURI' have attribute DontEnum");
  } else if (x === 'decodeURIComponent') {
    throw new Test262Error("#1: 'decodeURIComponent' have attribute DontEnum");
  } else if (x === 'encodeURI') {
    throw new Test262Error("#1: 'encodeURI' have attribute DontEnum");
  } else if (x === 'encodeURIComponent') {
    throw new Test262Error("#1: 'encodeURIComponent' have attribute DontEnum");
  }
}

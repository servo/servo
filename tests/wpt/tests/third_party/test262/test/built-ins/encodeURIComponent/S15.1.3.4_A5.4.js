// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of encodeURIComponent is 1
esid: sec-encodeuricomponent-uricomponent
description: encodeURIComponent.length === 1
---*/

//CHECK#1
if (encodeURIComponent.length !== 1) {
  throw new Test262Error('#1: encodeURIComponent.length === 1. Actual: ' + (encodeURIComponent.length));
}

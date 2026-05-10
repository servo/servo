// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The decodeURIComponent property has not prototype property
esid: sec-decodeuricomponent-encodeduricomponent
description: Checking decodeURIComponent.prototype
---*/

//CHECK#1
if (decodeURIComponent.prototype !== undefined) {
  throw new Test262Error('#1: decodeURIComponent.prototype === undefined. Actual: ' + (decodeURIComponent.prototype));
}

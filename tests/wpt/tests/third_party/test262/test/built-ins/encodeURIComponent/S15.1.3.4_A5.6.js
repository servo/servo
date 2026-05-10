// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The encodeURIComponent property has not prototype property
esid: sec-encodeuricomponent-uricomponent
description: Checking encodeURIComponent.prototype
---*/

//CHECK#1
if (encodeURIComponent.prototype !== undefined) {
  throw new Test262Error('#1: encodeURIComponent.prototype === undefined. Actual: ' + (encodeURIComponent.prototype));
}

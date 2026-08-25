// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The isNaN property has not prototype property
esid: sec-isnan-number
description: Checking isNaN.prototype
---*/
assert.sameValue(isNaN.prototype, undefined, 'The value of isNaN.prototype is expected to equal undefined');

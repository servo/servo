// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The isFinite property has not prototype property
esid: sec-isfinite-number
description: Checking isFinite.prototype
---*/
assert.sameValue(isFinite.prototype, undefined, 'The value of isFinite.prototype is expected to equal undefined');

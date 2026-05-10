// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The parseInt property has not prototype property
esid: sec-parseint-string-radix
description: Checking parseInt.prototype
---*/

assert.sameValue(Object.prototype.hasOwnProperty.call(parseInt, "prototype"), false, 'Object.prototype.hasOwnProperty.call(parseInt, "prototype") must return false');

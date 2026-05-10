// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.split has not prototype property
es5id: 15.5.4.14_A6
description: Checking String.prototype.split.prototype
---*/

assert.sameValue(
  String.prototype.split.prototype,
  undefined,
  'The value of String.prototype.split.prototype is expected to equal `undefined`'
);

// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the split method is 2
es5id: 15.5.4.14_A11
description: Checking String.prototype.split.length
---*/

assert(
  String.prototype.split.hasOwnProperty("length"),
  'String.prototype.split.hasOwnProperty("length") must return true'
);

assert.sameValue(String.prototype.split.length, 2, 'The value of String.prototype.split.length is 2');

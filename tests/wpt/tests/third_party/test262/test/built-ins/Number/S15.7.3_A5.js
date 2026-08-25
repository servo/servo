// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Number constructor has the property "NEGATIVE_INFINITY"
es5id: 15.7.3_A5
description: Checking existence of the property "NEGATIVE_INFINITY"
---*/
assert(
  Number.hasOwnProperty("NEGATIVE_INFINITY"),
  'Number.hasOwnProperty("NEGATIVE_INFINITY") must return true'
);

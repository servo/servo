// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Object constructor has the property "prototype"
es5id: 15.2.3_A1
description: Checking existence of the property "prototype"
---*/
assert(
  !!Object.hasOwnProperty("prototype"),
  'The value of !!Object.hasOwnProperty("prototype") is expected to be true'
);

// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If the property has the ReadOnly attribute, [[CanPut]](P) return false
es5id: 8.12.4_A1
description: Try put other value for Math.E property
flags: [noStrict]
---*/

var __e = Math.E;
Math.E = 1;
if (Math.E !== __e){
  throw new Test262Error('#1: __e = Math.E; Math.E = 1; Math.E === __e. Actual: ' + (Math.E));
}

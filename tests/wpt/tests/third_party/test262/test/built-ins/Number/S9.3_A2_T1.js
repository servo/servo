// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of number conversion from null value is +0
es5id: 9.3_A2_T1
description: null convert to Number by explicit transformation
---*/
assert.sameValue(Number(null), 0, 'Number(null) must return 0');

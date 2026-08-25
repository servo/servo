// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The NaN is DontDelete
es5id: 15.1.1.1_A3_T2
description: Use delete
flags: [noStrict]
---*/
assert.sameValue(delete NaN, false, 'The value of `delete NaN` is expected to be false');

// TODO: Convert to verifyProperty() format.

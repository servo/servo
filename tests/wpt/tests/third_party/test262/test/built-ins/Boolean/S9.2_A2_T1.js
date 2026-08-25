// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of boolean conversion from null value is false
esid: sec-toboolean
description: null convert to Boolean by explicit transformation
---*/
assert.sameValue(Boolean(null), false, 'Boolean(null) must return false');

// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of boolean conversion from boolean value is no conversion
esid: sec-toboolean
description: true and false convert to Boolean by explicit transformation
---*/
assert.sameValue(Boolean(true), true, 'Boolean(true) must return true');
assert.sameValue(Boolean(false), false, 'Boolean(false) must return false');

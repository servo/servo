// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number() returns +0
es5id: 15.7.1.1_A2
description: Call Number() and check result
---*/
assert.sameValue(typeof Number(), "number", 'The value of `typeof Number()` is expected to be "number"');
assert.sameValue(Number(), 0, 'Number() must return 0');

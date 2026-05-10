// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-undefined
description: undefined is not writable, should not throw in non-strict mode
flags: [noStrict]
---*/

undefined = 5;
assert.sameValue(typeof undefined, "undefined", 'typeof undefined');

var nosuchproperty;
assert.sameValue(nosuchproperty, undefined, 'nosuchproperty');

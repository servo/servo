// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of boolean conversion from undefined value is false
esid: sec-toboolean
description: >
    Undefined, void and others are converted to Boolean by explicit
    transformation
---*/

assert.sameValue(Boolean(undefined), false, 'Boolean(undefined) must return false');
assert.sameValue(Boolean(void 0), false, 'Boolean(void 0) must return false');
assert.sameValue(Boolean(eval("var x")), false, 'Boolean(eval("var x")) must return false');
assert.sameValue(Boolean(), false, 'Boolean() must return false');

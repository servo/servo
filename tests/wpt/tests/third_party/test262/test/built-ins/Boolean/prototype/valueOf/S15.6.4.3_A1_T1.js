// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Boolean.prototype.valueOf() returns this boolean value
esid: sec-boolean.prototype.valueof
description: no arguments
---*/
assert.sameValue(Boolean.prototype.valueOf(), false, 'Boolean.prototype.valueOf() must return false');
assert.sameValue((new Boolean()).valueOf(), false, '(new Boolean()).valueOf() must return false');
assert.sameValue((new Boolean(0)).valueOf(), false, '(new Boolean(0)).valueOf() must return false');
assert.sameValue((new Boolean(-1)).valueOf(), true, '(new Boolean(-1)).valueOf() must return true');
assert.sameValue((new Boolean(1)).valueOf(), true, '(new Boolean(1)).valueOf() must return true');

assert.sameValue(
  (new Boolean(new Object())).valueOf(),
  true,
  '(new Boolean(new Object())).valueOf() must return true'
);

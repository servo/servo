// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Boolean.prototype.valueOf() returns this boolean value
esid: sec-boolean.prototype.valueof
description: calling with argument
---*/
assert.sameValue(Boolean.prototype.valueOf(true), false, 'Boolean.prototype.valueOf(true) must return false');
assert.sameValue((new Boolean()).valueOf(true), false, '(new Boolean()).valueOf(true) must return false');
assert.sameValue((new Boolean(0)).valueOf(true), false, '(new Boolean(0)).valueOf(true) must return false');
assert.sameValue((new Boolean(-1)).valueOf(false), true, '(new Boolean(-1)).valueOf(false) must return true');
assert.sameValue((new Boolean(1)).valueOf(false), true, '(new Boolean(1)).valueOf(false) must return true');

assert.sameValue(
  (new Boolean(new Object())).valueOf(false),
  true,
  '(new Boolean(new Object())).valueOf(false) must return true'
);

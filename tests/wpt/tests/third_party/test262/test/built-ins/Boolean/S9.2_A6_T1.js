// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of boolean conversion from object is true
esid: sec-toboolean
description: Different objects convert to Boolean by explicit transformation
---*/

assert.sameValue(Boolean(new Object()), true, 'Boolean(new Object()) must return true');
assert.sameValue(Boolean(new String("")), true, 'Boolean(new String("")) must return true');
assert.sameValue(Boolean(new String()), true, 'Boolean(new String()) must return true');
assert.sameValue(Boolean(new Boolean(true)), true, 'Boolean(new Boolean(true)) must return true');
assert.sameValue(Boolean(new Boolean(false)), true, 'Boolean(new Boolean(false)) must return true');
assert.sameValue(Boolean(new Boolean()), true, 'Boolean(new Boolean()) must return true');
assert.sameValue(Boolean(new Array()), true, 'Boolean(new Array()) must return true');
assert.sameValue(Boolean(new Number()), true, 'Boolean(new Number()) must return true');
assert.sameValue(Boolean(new Number(-0)), true, 'Boolean(new Number(-0)) must return true');
assert.sameValue(Boolean(new Number(0)), true, 'Boolean(new Number(0)) must return true');
assert.sameValue(Boolean(new Number()), true, 'Boolean(new Number()) must return true');
assert.sameValue(Boolean(new Number(Number.NaN)), true, 'Boolean(new Number(Number.NaN)) must return true');
assert.sameValue(Boolean(new Number(-1)), true, 'Boolean(new Number(-1)) must return true');
assert.sameValue(Boolean(new Number(1)), true, 'Boolean(new Number(1)) must return true');

assert.sameValue(
  Boolean(new Number(Number.POSITIVE_INFINITY)),
  true,
  'Boolean(new Number(Number.POSITIVE_INFINITY)) must return true'
);

assert.sameValue(
  Boolean(new Number(Number.NEGATIVE_INFINITY)),
  true,
  'Boolean(new Number(Number.NEGATIVE_INFINITY)) must return true'
);

assert.sameValue(Boolean(new Function()), true, 'Boolean(new Function()) must return true');
assert.sameValue(Boolean(new Date()), true, 'Boolean(new Date()) must return true');
assert.sameValue(Boolean(new Date(0)), true, 'Boolean(new Date(0)) must return true');

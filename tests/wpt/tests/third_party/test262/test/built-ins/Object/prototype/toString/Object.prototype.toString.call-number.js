// Copyright 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: has a [[NumberData]] internal slot, let builtinTag be "Number"
---*/
assert.sameValue(
  Object.prototype.toString.call(9),
  "[object Number]",
  "Object.prototype.toString.call(9) returns [object Number]"
);
assert.sameValue(
  Object.prototype.toString.call(Object(9)),
  "[object Number]",
  "Object.prototype.toString.call(Object(9)) returns [object Number]"
);
assert.sameValue(
  Object.prototype.toString.call(Number(9)),
  "[object Number]",
  "Object.prototype.toString.call(Number(9)) returns [object Number]"
);
assert.sameValue(
  Object.prototype.toString.call(Object(Number(9))),
  "[object Number]",
  "Object.prototype.toString.call(Object(Number(9))) returns [object Number]"
);
assert.sameValue(
  Object.prototype.toString.call(new Number(9)),
  "[object Number]",
  "Object.prototype.toString.call(new Number(9)) returns [object Number]"
);
assert.sameValue(
  Object.prototype.toString.call(Object(new Number(9))),
  "[object Number]",
  "Object.prototype.toString.call(Object(new Number(9))) returns [object Number]"
);

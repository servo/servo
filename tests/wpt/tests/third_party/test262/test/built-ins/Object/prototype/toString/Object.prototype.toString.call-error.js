// Copyright 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: has an [[ErrorData]] internal slot, let builtinTag be "Error".
---*/
assert.sameValue(
  Object.prototype.toString.call(Error()),
  "[object Error]",
  "Object.prototype.toString.call(Error()) returns [object Error]"
);
assert.sameValue(
  Object.prototype.toString.call(Object(Error())),
  "[object Error]",
  "Object.prototype.toString.call(Object(Error())) returns [object Error]"
);
assert.sameValue(
  Object.prototype.toString.call(new Error()),
  "[object Error]",
  "Object.prototype.toString.call(new Error()) returns [object Error]"
);
assert.sameValue(
  Object.prototype.toString.call(Object(new Error())),
  "[object Error]",
  "Object.prototype.toString.call(Object(new Error())) returns [object Error]"
);

// Copyright 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.tostring
description: is a String exotic object, let builtinTag be "String".
---*/
assert.sameValue(
  Object.prototype.toString.call(""),
  "[object String]",
  "Object.prototype.toString.call(\"\") returns [object String]"
);
assert.sameValue(
  Object.prototype.toString.call(Object("")),
  "[object String]",
  "Object.prototype.toString.call(Object(\"\")) returns [object String]"
);
assert.sameValue(
  Object.prototype.toString.call(String("")),
  "[object String]",
  "Object.prototype.toString.call(String(\"\")) returns [object String]"
);
assert.sameValue(
  Object.prototype.toString.call(Object(String(""))),
  "[object String]",
  "Object.prototype.toString.call(Object(String(\"\"))) returns [object String]"
);
assert.sameValue(
  Object.prototype.toString.call(new String("")),
  "[object String]",
  "Object.prototype.toString.call(new String(\"\")) returns [object String]"
);
assert.sameValue(
  Object.prototype.toString.call(Object(new String(""))),
  "[object String]",
  "Object.prototype.toString.call(Object(new String(\"\"))) returns [object String]"
);

// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Boolean.prototype has the attribute DontDelete
esid: sec-boolean.prototype
description: Checking if deleting the Boolean.prototype property fails
includes: [propertyHelper.js]
flags: [onlyStrict]
---*/

// CHECK#1
verifyNotConfigurable(Boolean, "prototype");

assert.throws(TypeError, () => {
  delete Boolean.prototype;
});

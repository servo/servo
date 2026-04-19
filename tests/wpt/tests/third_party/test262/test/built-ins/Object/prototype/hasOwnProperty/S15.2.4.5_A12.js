// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.4.5_A12
description: >
    Let O be the result of calling ToObject passing the this value as
    the argument.
---*/

assert.throws(TypeError, function() {
  Object.prototype.hasOwnProperty.call(undefined, 'foo');
}, 'Object.prototype.hasOwnProperty.call(undefined, "foo") throws a TypeError exception');

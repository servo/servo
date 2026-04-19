// Copyright 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.hasown
description: >
    Let O be the result of calling ToObject passing the this value as
    the argument.
author: Jamie Kyle
features: [Object.hasOwn]
---*/

assert.throws(TypeError, function() {
  Object.hasOwn(null, 'foo');
});

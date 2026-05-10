// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 19.5.6.3
description: >
  ReferenceError.prototype is not an error object instance.
info: |
  Each NativeError prototype object is an ordinary object. It is not an
  Error instance and does not have an [[ErrorData]] internal slot.
---*/

assert.sameValue(Object.prototype.toString.call(ReferenceError.prototype), "[object Object]");

// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.5.6.1
description: Super need to be called to initialize internals
info: |
  19.5.6.1  NativeError Constructors

  ...
  Each NativeError constructor is designed to be subclassable. It may be used as
  the value of an extends clause of a class definition. Subclass constructors
  that intend to inherit the specified NativeError behaviour must include a
  super call to the NativeError constructor to create and initialize subclass
  instances with a [[ErrorData]] internal slot.
---*/

class CustomError extends SyntaxError {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new CustomError();
});

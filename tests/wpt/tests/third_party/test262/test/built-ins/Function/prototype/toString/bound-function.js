// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// Copyright (C) 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.tostring
description: >
  toString bound function does not throw (bound Function Expression)
info: |
  ...
  If func is a Bound Function exotic object or a built-in Function object,
  then return an implementation-dependent String source code representation
  of func. The representation must have the syntax of a NativeFunction
  ...
includes: [nativeFunctionMatcher.js]
---*/

assertNativeFunction(function() {}.bind({}));

// Copyright (C) 2018 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.tostring
description: Function.prototype.toString on symbol-named built-ins
includes: [nativeFunctionMatcher.js]
---*/

assertNativeFunction(RegExp.prototype[Symbol.match]);
assertNativeFunction(Object.getOwnPropertyDescriptor(RegExp, Symbol.species).get);

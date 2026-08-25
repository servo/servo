// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [nativeFunctionMatcher.js]
description: |
  pending
esid: pending
---*/

// Bound functions are considered built-ins.
assertNativeFunction(function(){}.bind());
assertNativeFunction(function fn(){}.bind());

// Built-ins which are well-known intrinsic objects.
assertNativeFunction(Array);
assertNativeFunction(Object.prototype.toString);
assertNativeFunction(decodeURI);

// Other built-in functions.
assertNativeFunction(Math.asin);
assertNativeFunction(String.prototype.blink);
assertNativeFunction(RegExp.prototype[Symbol.split]);

// Built-in getter functions.
assertNativeFunction(Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get);
assertNativeFunction(Object.getOwnPropertyDescriptor(Object.prototype, "__proto__").get);

// Built-in setter functions.
assertNativeFunction(Object.getOwnPropertyDescriptor(Object.prototype, "__proto__").set);

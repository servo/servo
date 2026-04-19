// Copyright 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.tostring
description: >
  Function.prototype.toString on an async generator created with the
  AsyncGenerator constructor.
features: [async-iteration]
includes: [nativeFunctionMatcher.js]
---*/

async function* f() {}
var AsyncGenerator = f.constructor;

var g = /* before */AsyncGenerator("a", " /* a */ b, c /* b */ //", "/* c */ ; /* d */ //")/* after */;
assertToStringOrNativeFunction(g, "async function* anonymous(a, /* a */ b, c /* b */ //\n) {\n/* c */ ; /* d */ //\n}");

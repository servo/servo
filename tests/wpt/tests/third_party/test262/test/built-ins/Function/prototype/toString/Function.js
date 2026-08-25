// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdynamicfunction
description: Function.prototype.toString on a function created with the Function constructor
includes: [nativeFunctionMatcher.js]
---*/

let f = /* before */Function("a", " /* a */ b, c /* b */ //", "/* c */ ; /* d */ //")/* after */;

assertToStringOrNativeFunction(f, "function anonymous(a, /* a */ b, c /* b */ //\n) {\n/* c */ ; /* d */ //\n}");

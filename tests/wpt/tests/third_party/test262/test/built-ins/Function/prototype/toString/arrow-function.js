// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arrow-function-definitions-runtime-semantics-evaluation
description: Function.prototype.toString on an arrow function
includes: [nativeFunctionMatcher.js]
---*/

let f = /* before */( /* a */ a /* b */ , /* c */ b /* d */ ) /* e */ => /* f */ { /* g */ ; /* h */ }/* after */;
let g = /* before */( /* a */ ) /* b */ => /* c */ 0/* after */;
let h = /* before */a /* a */ => /* b */ 0/* after */;

assertToStringOrNativeFunction(f, "( /* a */ a /* b */ , /* c */ b /* d */ ) /* e */ => /* f */ { /* g */ ; /* h */ }");
assertToStringOrNativeFunction(g, "( /* a */ ) /* b */ => /* c */ 0");
assertToStringOrNativeFunction(h, "a /* a */ => /* b */ 0");

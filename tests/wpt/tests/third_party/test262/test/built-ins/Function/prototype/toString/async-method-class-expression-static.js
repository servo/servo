// Copyright 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.tostring
description: Function.prototype.toString on an async method
features: [async-functions]
includes: [nativeFunctionMatcher.js]
---*/

let x = "h";
let f = class { static /* before */async f /* a */ ( /* b */ ) /* c */ { /* d */ }/* after */ }.f;
let g = class { static /* before */async /* a */ [ /* b */ "g" /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }/* after */ }.g;
let h = class { static /* before */async /* a */ [ /* b */ x /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }/* after */ }.h;

assertToStringOrNativeFunction(f, "async f /* a */ ( /* b */ ) /* c */ { /* d */ }");
assertToStringOrNativeFunction(g, "async /* a */ [ /* b */ \"g\" /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }");
assertToStringOrNativeFunction(h, "async /* a */ [ /* b */ x /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }");

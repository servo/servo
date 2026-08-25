// Copyright 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.tostring
description: Function.prototype.toString on an async generator method
features: [async-iteration]
includes: [nativeFunctionMatcher.js]
---*/

let x = "h";
let f = class { static /* before */async /* a */ * /* b */ f /* c */ ( /* d */ ) /* e */ { /* f */ }/* after */ }.f;
let g = class { static /* before */async /* a */ * /* b */ [ /* c */ "g" /* d */ ] /* e */ ( /* f */ ) /* g */ { /* h */ }/* after */ }.g;
let h = class { static /* before */async /* a */ * /* b */ [ /* c */ x /* d */ ] /* e */ ( /* f */ ) /* g */ { /* h */ }/* after */ }.h;

assertToStringOrNativeFunction(f, "async /* a */ * /* b */ f /* c */ ( /* d */ ) /* e */ { /* f */ }");
assertToStringOrNativeFunction(g, "async /* a */ * /* b */ [ /* c */ \"g\" /* d */ ] /* e */ ( /* f */ ) /* g */ { /* h */ }");
assertToStringOrNativeFunction(h, "async /* a */ * /* b */ [ /* c */ x /* d */ ] /* e */ ( /* f */ ) /* g */ { /* h */ }");

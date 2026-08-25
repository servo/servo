// Copyright 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.tostring
description: Function.prototype.toString on an async generator method
features: [async-iteration]
includes: [nativeFunctionMatcher.js]
---*/

let x = "h";
class F { static /* before */async /* a */ * /* b */ f /* c */ ( /* d */ ) /* e */ { /* f */ }/* after */ }
class G { static /* before */async /* a */ * /* b */ [ /* c */ "g" /* d */ ] /* e */ ( /* f */ ) /* g */ { /* h */ }/* after */ }
class H { static /* before */async /* a */ * /* b */ [ /* c */ x /* d */ ] /* e */ ( /* f */ ) /* g */ { /* h */ }/* after */ }

let f = F.f;
let g = G.g;
let h = H.h;

assertToStringOrNativeFunction(f, "async /* a */ * /* b */ f /* c */ ( /* d */ ) /* e */ { /* f */ }");
assertToStringOrNativeFunction(g, "async /* a */ * /* b */ [ /* c */ \"g\" /* d */ ] /* e */ ( /* f */ ) /* g */ { /* h */ }");
assertToStringOrNativeFunction(h, "async /* a */ * /* b */ [ /* c */ x /* d */ ] /* e */ ( /* f */ ) /* g */ { /* h */ }");

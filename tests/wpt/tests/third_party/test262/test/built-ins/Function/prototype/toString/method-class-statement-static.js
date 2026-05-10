// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-definemethod
description: Function.prototype.toString on a method (class; static)
includes: [nativeFunctionMatcher.js]
---*/

let x = "h";
class F { static /* before */f /* a */ ( /* b */ ) /* c */ { /* d */ }/* after */ }
class G { static /* before */[ /* a */ "g" /* b */ ] /* c */ ( /* d */ ) /* e */ { /* f */ }/* after */ }
class H { static /* before */[ /* a */ x /* b */ ] /* c */ ( /* d */ ) /* e */ { /* f */ }/* after */ }

let f = F.f;
let g = G.g;
let h = H.h;

assertToStringOrNativeFunction(f, "f /* a */ ( /* b */ ) /* c */ { /* d */ }");
assertToStringOrNativeFunction(g, "[ /* a */ \"g\" /* b */ ] /* c */ ( /* d */ ) /* e */ { /* f */ }");
assertToStringOrNativeFunction(h, "[ /* a */ x /* b */ ] /* c */ ( /* d */ ) /* e */ { /* f */ }");

// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-definemethod
description: Function.prototype.toString on a method (class)
includes: [nativeFunctionMatcher.js]
---*/

let x = "h";
let f = class { /* before */f /* a */ ( /* b */ ) /* c */ { /* d */ }/* after */ }.prototype.f;
let g = class { /* before */[ /* a */ "g" /* b */ ] /* c */ ( /* d */ ) /* e */ { /* f */ }/* after */ }.prototype.g;
let h = class { /* before */[ /* a */ x /* b */ ] /* c */ ( /* d */ ) /* e */ { /* f */ }/* after */ }.prototype.h;

assertToStringOrNativeFunction(f, "f /* a */ ( /* b */ ) /* c */ { /* d */ }");
assertToStringOrNativeFunction(g, "[ /* a */ \"g\" /* b */ ] /* c */ ( /* d */ ) /* e */ { /* f */ }");
assertToStringOrNativeFunction(h, "[ /* a */ x /* b */ ] /* c */ ( /* d */ ) /* e */ { /* f */ }");

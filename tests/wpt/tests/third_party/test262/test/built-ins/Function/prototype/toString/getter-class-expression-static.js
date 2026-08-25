// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-method-definitions-runtime-semantics-propertydefinitionevaluation
description: Function.prototype.toString on a getter (class; static)
includes: [nativeFunctionMatcher.js]
---*/

let x = "h";
let f = Object.getOwnPropertyDescriptor(class { static /* before */get /* a */ f /* b */ ( /* c */ ) /* d */ { /* e */ }/* after */ }, "f").get;
let g = Object.getOwnPropertyDescriptor(class { static /* before */get /* a */ [ /* b */ "g" /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }/* after */ }, "g").get;
let h = Object.getOwnPropertyDescriptor(class { static /* before */get /* a */ [ /* b */ x /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }/* after */ }, "h").get;

assertToStringOrNativeFunction(f, "get /* a */ f /* b */ ( /* c */ ) /* d */ { /* e */ }");
assertToStringOrNativeFunction(g, "get /* a */ [ /* b */ \"g\" /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }");
assertToStringOrNativeFunction(h, "get /* a */ [ /* b */ x /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }");

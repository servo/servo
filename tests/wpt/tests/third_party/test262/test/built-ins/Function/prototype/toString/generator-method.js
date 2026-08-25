// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generator-function-definitions-runtime-semantics-propertydefinitionevaluation
description: Function.prototype.toString on a generator method
includes: [nativeFunctionMatcher.js]
---*/

let x = "h";
let f = { /* before */* /* a */ f /* b */ ( /* c */ ) /* d */ { /* e */ }/* after */ }.f;
let g = { /* before */* /* a */ [ /* b */ "g" /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }/* after */ }.g;
let h = { /* before */* /* a */ [ /* b */ x /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }/* after */ }.h;

assertToStringOrNativeFunction(f, "* /* a */ f /* b */ ( /* c */ ) /* d */ { /* e */ }");
assertToStringOrNativeFunction(g, "* /* a */ [ /* b */ \"g\" /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }");
assertToStringOrNativeFunction(h, "* /* a */ [ /* b */ x /* c */ ] /* d */ ( /* e */ ) /* f */ { /* g */ }");

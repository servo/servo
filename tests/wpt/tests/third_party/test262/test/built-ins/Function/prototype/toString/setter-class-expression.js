// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-method-definitions-runtime-semantics-propertydefinitionevaluation
description: Function.prototype.toString on a setter (class)
includes: [nativeFunctionMatcher.js]
---*/

let x = "h";
let f = Object.getOwnPropertyDescriptor(class { /* before */set /* a */ f /* b */ ( /* c */ a /* d */ ) /* e */ { /* f */ }/* after */ }.prototype, "f").set;
let g = Object.getOwnPropertyDescriptor(class { /* before */set /* a */ [ /* b */ "g" /* c */ ] /* d */ ( /* e */ a /* f */ ) /* g */ { /* h */ }/* after */ }.prototype, "g").set;
let h = Object.getOwnPropertyDescriptor(class { /* before */set /* a */ [ /* b */ x /* c */ ] /* d */ ( /* e */ a /* f */ ) /* g */ { /* h */ }/* after */ }.prototype, "h").set;

assertToStringOrNativeFunction(f, "set /* a */ f /* b */ ( /* c */ a /* d */ ) /* e */ { /* f */ }");
assertToStringOrNativeFunction(g, "set /* a */ [ /* b */ \"g\" /* c */ ] /* d */ ( /* e */ a /* f */ ) /* g */ { /* h */ }");
assertToStringOrNativeFunction(h, "set /* a */ [ /* b */ x /* c */ ] /* d */ ( /* e */ a /* f */ ) /* g */ { /* h */ }");

// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function-definitions-runtime-semantics-evaluation
description: Function.prototype.toString on a function expression
includes: [nativeFunctionMatcher.js]
---*/

let f = /* before */function /* a */ F /* b */ ( /* c */ x /* d */ , /* e */ y /* f */ ) /* g */ { /* h */ ; /* i */ ; /* j */ }/* after */;
let g = /* before */function /* a */ ( /* b */ x /* c */ , /* d */ y /* e */ ) /* f */ { /* g */ ; /* h */ ; /* i */ }/* after */;

assertToStringOrNativeFunction(f, "function /* a */ F /* b */ ( /* c */ x /* d */ , /* e */ y /* f */ ) /* g */ { /* h */ ; /* i */ ; /* j */ }");
assertToStringOrNativeFunction(g, "function /* a */ ( /* b */ x /* c */ , /* d */ y /* e */ ) /* f */ { /* g */ ; /* h */ ; /* i */ }");

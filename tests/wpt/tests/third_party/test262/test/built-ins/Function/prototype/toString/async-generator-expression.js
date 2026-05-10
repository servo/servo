// Copyright 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.tostring
description: Function.prototype.toString on an async generator expression
features: [async-iteration]
includes: [nativeFunctionMatcher.js]
---*/

let f = /* before */async /* a */ function /* b */ * /* c */ F /* d */ ( /* e */ x /* f */ , /* g */ y /* h */ ) /* i */ { /* j */ ; /* k */ ; /* l */ }/* after */;
let g = /* before */async /* a */ function /* b */ * /* c */ ( /* d */ x /* e */ , /* f */ y /* g */ ) /* h */ { /* i */ ; /* j */ ; /* k */ }/* after */;

assertToStringOrNativeFunction(f, "async /* a */ function /* b */ * /* c */ F /* d */ ( /* e */ x /* f */ , /* g */ y /* h */ ) /* i */ { /* j */ ; /* k */ ; /* l */ }");
assertToStringOrNativeFunction(g, "async /* a */ function /* b */ * /* c */ ( /* d */ x /* e */ , /* f */ y /* g */ ) /* h */ { /* i */ ; /* j */ ; /* k */ }");

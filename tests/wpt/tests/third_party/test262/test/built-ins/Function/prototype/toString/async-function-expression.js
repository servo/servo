// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-function.prototype.tostring
description: Function.prototype.toString on an async function expression
features: [async-functions]
includes: [nativeFunctionMatcher.js]
---*/

let f = /* before */async function /* a */ F /* b */ ( /* c */ x /* d */ , /* e */ y /* f */ ) /* g */ { /* h */ ; /* i */ ; /* j */ }/* after */;
let g = /* before */async function /* a */ ( /* b */ x /* c */ , /* d */ y /* e */ ) /* f */ { /* g */ ; /* h */ ; /* i */ }/* after */;

assertToStringOrNativeFunction(f, "async function /* a */ F /* b */ ( /* c */ x /* d */ , /* e */ y /* f */ ) /* g */ { /* h */ ; /* i */ ; /* j */ }");
assertToStringOrNativeFunction(g, "async function /* a */ ( /* b */ x /* c */ , /* d */ y /* e */ ) /* f */ { /* g */ ; /* h */ ; /* i */ }");

// Copyright 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.tostring
description: Function.prototype.toString on an async generator declaration
features: [async-iteration]
includes: [nativeFunctionMatcher.js]
---*/

/* before */async /* a */ function /* b */ * /* c */ f /* d */ ( /* e */ x /* f */ , /* g */ y /* h */ ) /* i */ { /* j */ ; /* k */ ; /* l */ }/* after */

assertToStringOrNativeFunction(f, "async /* a */ function /* b */ * /* c */ f /* d */ ( /* e */ x /* f */ , /* g */ y /* h */ ) /* i */ { /* j */ ; /* k */ ; /* l */ }");

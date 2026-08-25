// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-function.prototype.tostring
description: Function.prototype.toString on an async function declaration
features: [async-functions]
includes: [nativeFunctionMatcher.js]
---*/

/* before */async function /* a */ f /* b */ ( /* c */ x /* d */ , /* e */ y /* f */ ) /* g */ { /* h */ ; /* i */ ; /* j */ }/* after */

assertToStringOrNativeFunction(f, "async function /* a */ f /* b */ ( /* c */ x /* d */ , /* e */ y /* f */ ) /* g */ { /* h */ ; /* i */ ; /* j */ }");

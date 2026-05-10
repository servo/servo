// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generator-function-definitions-runtime-semantics-instantiatefunctionobject
description: Function.prototype.toString on a generator function declaration
includes: [nativeFunctionMatcher.js]
---*/

/* before */function /* a */ * /* b */ g /* c */ ( /* d */ x /* e */ , /* f */ y /* g */ ) /* h */ { /* i */ ; /* j */ ; /* k */ }/* after */

assertToStringOrNativeFunction(g, "function /* a */ * /* b */ g /* c */ ( /* d */ x /* e */ , /* f */ y /* g */ ) /* h */ { /* i */ ; /* j */ ; /* k */ }");

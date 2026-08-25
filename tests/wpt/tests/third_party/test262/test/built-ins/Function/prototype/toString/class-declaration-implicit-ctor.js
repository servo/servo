// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-bindingclassdeclarationevaluation
description: Function.prototype.toString on a class declaration (implicit constructor)
includes: [nativeFunctionMatcher.js]
---*/

/* before */class /* a */ A /* b */ { /* c */ }/* after */
/* before */class /* a */ B /* b */ extends /* c */ A /* d */ { /* e */ }/* after */
/* before */class /* a */ C /* b */ extends /* c */ B /* d */ { /* e */ m /* f */ ( /* g */ ) /* h */ { /* i */ } /* j */ }/* after */

assertToStringOrNativeFunction(A, "class /* a */ A /* b */ { /* c */ }");
assertToStringOrNativeFunction(B, "class /* a */ B /* b */ extends /* c */ A /* d */ { /* e */ }");
assertToStringOrNativeFunction(C, "class /* a */ C /* b */ extends /* c */ B /* d */ { /* e */ m /* f */ ( /* g */ ) /* h */ { /* i */ } /* j */ }");

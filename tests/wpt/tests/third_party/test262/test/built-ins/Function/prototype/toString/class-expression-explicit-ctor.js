// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-class-definitions-runtime-semantics-evaluation
description: Function.prototype.toString on a class expression (explicit constructor)
includes: [nativeFunctionMatcher.js]
---*/

let A = /* before */class /* a */ A /* b */ extends /* c */ B /* d */ { /* e */ constructor /* f */ ( /* g */ ) /* h */ { /* i */ ; /* j */ } /* k */ m /* l */ ( /* m */ ) /* n */ { /* o */ } /* p */ }/* after */;

assertToStringOrNativeFunction(A, "class /* a */ A /* b */ extends /* c */ B /* d */ { /* e */ constructor /* f */ ( /* g */ ) /* h */ { /* i */ ; /* j */ } /* k */ m /* l */ ( /* m */ ) /* n */ { /* o */ } /* p */ }");

function B(){}

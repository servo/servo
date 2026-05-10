// Copyright (C) 2016 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-definemethod
description: Function.prototype.toString on a method (object)
includes: [nativeFunctionMatcher.js]
---*/

let f = { /* before */f /* a */ ( /* b */ ) /* c */ { /* d */ }/* after */ }.f;

assertToStringOrNativeFunction(f, "f /* a */ ( /* b */ ) /* c */ { /* d */ }");

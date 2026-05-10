// Copyright (C) 2019 Kubilay Kahveci (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Function.prototype.toString on a private method
features: [class-methods-private]
includes: [nativeFunctionMatcher.js]
---*/

let c = new (class {
  /* before */#f /* a */ ( /* b */ ) /* c */ { /* d */ }/* after */
  assert(expected) {
    assertToStringOrNativeFunction(this.#f, expected);
  }
});

c.assert("#f /* a */ ( /* b */ ) /* c */ { /* d */ }");

// Copyright 2020 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-unwrapdatetimeformat
description: >
    Tests that [[FallbackSymbol]]'s [[Description]] is "IntlLegacyConstructedSymbol" if normative optional is implemented.
author: Yusuke Suzuki
features: [intl-normative-optional]
---*/

let object = new Intl.DateTimeFormat();
let newObject = Intl.DateTimeFormat.call(object);
let symbol = null;
let error = null;
try {
    let proxy = new Proxy(newObject, {
        get(target, property) {
            symbol = property;
            return target[property];
        }
    });
    Intl.DateTimeFormat.prototype.resolvedOptions.call(proxy);
} catch (e) {
    // If normative optional is not implemented, an error will be thrown.
    error = e;
    assert(error instanceof TypeError);
}
if (error === null) {
      assert.sameValue(typeof symbol, "symbol");
      assert.sameValue(symbol.description, "IntlLegacyConstructedSymbol");
}

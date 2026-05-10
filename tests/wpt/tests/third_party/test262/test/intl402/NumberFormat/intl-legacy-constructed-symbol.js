// Copyright 2020 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: >
    Tests that [[FallbackSymbol]]'s [[Description]] is "IntlLegacyConstructedSymbol" if normative optional is implemented.
author: Yusuke Suzuki
features: [intl-normative-optional]
---*/

let object = new Intl.NumberFormat();
let newObject = Intl.NumberFormat.call(object);
let symbols = Object.getOwnPropertySymbols(newObject);
if (symbols.length !== 0) {
    assert.sameValue(symbols.length, 1);
    assert.sameValue(symbols[0].description, "IntlLegacyConstructedSymbol");
}

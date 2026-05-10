// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype-@@tostringtag
description: >
    Checks Object.prototype.toString with Intl.Locale objects.
info: |
    Intl.Locale.prototype[ @@toStringTag ]

    The initial value of the @@toStringTag property is the string value "Intl.Locale".
features: [Intl.Locale, Symbol.toStringTag]
---*/

assert.sameValue(Object.prototype.toString.call(Intl.Locale.prototype), "[object Intl.Locale]");
assert.sameValue(Object.prototype.toString.call(new Intl.Locale("en")), "[object Intl.Locale]");

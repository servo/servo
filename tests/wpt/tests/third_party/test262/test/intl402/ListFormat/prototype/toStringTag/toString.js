// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.ListFormat.prototype-@@tostringtag
description: >
    Checks Object.prototype.toString with Intl.ListFormat objects.
info: |
    Intl.ListFormat.prototype[ @@toStringTag ]

    The initial value of the @@toStringTag property is the string value "Intl.ListFormat".
features: [Intl.ListFormat]
---*/

assert.sameValue(Object.prototype.toString.call(Intl.ListFormat.prototype), "[object Intl.ListFormat]");
assert.sameValue(Object.prototype.toString.call(new Intl.ListFormat()), "[object Intl.ListFormat]");

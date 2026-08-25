// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.RelativeTimeFormat.prototype-@@tostringtag
description: >
    Checks Object.prototype.toString with Intl.RelativeTimeFormat objects.
info: |
    Intl.RelativeTimeFormat.prototype[ @@toStringTag ]

    The initial value of the @@toStringTag property is the string value "Intl.RelativeTimeFormat".
features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(Object.prototype.toString.call(Intl.RelativeTimeFormat.prototype), "[object Intl.RelativeTimeFormat]");
assert.sameValue(Object.prototype.toString.call(new Intl.RelativeTimeFormat("en")), "[object Intl.RelativeTimeFormat]");

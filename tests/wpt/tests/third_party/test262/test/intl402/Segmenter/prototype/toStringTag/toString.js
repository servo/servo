// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.Segmenter.prototype-@@tostringtag
description: >
    Checks Object.prototype.toString with Intl.Segmenter objects.
info: |
    Intl.Segmenter.prototype[ @@toStringTag ]

    The initial value of the @@toStringTag property is the string value "Intl.Segmenter".
features: [Intl.Segmenter]
---*/

assert.sameValue(Object.prototype.toString.call(Intl.Segmenter.prototype), "[object Intl.Segmenter]");
assert.sameValue(Object.prototype.toString.call(new Intl.Segmenter()), "[object Intl.Segmenter]");

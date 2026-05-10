// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Checks handling of a null options argument to the DurationFormat constructor.
info: |
    Intl.DurationFormat ( [ locales [ , options ] ] )
    (...)
    4. Let options be GetOptionsObject(options).
features: [Intl.DurationFormat]
---*/

assert.sameValue(typeof Intl.DurationFormat, "function");

assert.throws(TypeError, function() { new Intl.DurationFormat([], null) })

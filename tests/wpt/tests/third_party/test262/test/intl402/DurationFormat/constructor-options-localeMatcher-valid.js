// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Tests that the option localeMatcher is processed correctly.
info: |
    Intl.DurationFormat ( [ locales [ , options ] ] )
    (...)
    5. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
features: [Intl.DurationFormat]
includes: [testIntl.js]
---*/

testOption(Intl.DurationFormat, "localeMatcher", "string", ["lookup", "best fit"], "best fit", {noReturn: true});

// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: >
    Checks error cases for the options argument to the DurationFormat constructor.
info: |
    Intl.DurationFormat ( [ locales [ , options ] ] )
    (...)
    6. Let numberingSystem be ? GetOption(options, "numberingSystem", "string", undefined, undefined).
    7. If numberingSystem does not match the Unicode Locale Identifier type nonterminal, throw a RangeError exception.
features: [Intl.DurationFormat]
---*/

const numberingSystems = Intl.supportedValuesOf("numberingSystem");

for (const numberingSystem of numberingSystems) {
  const obj = new Intl.DurationFormat("en", {numberingSystem});
  assert.sameValue(obj.resolvedOptions().numberingSystem, numberingSystem, `${numberingSystem} is supported by DurationFormat`);
}

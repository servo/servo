// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Checks the order of operations on the options argument to the DurationFormat constructor.
info: |
    Intl.DurationFormat ( [ locales [ , options ] ] )
    (...)
    5. Let matcher be ? GetOption(options, "localeMatcher", "string", « "lookup", "best fit" », "best fit").
    6. Let numberingSystem be ? GetOption(options, "numberingSystem", "string", undefined, undefined).
    13. Let style be ? GetOption(options, "style", "string", « "long", "short", "narrow", "digital" », "long").
includes: [compareArray.js]
features: [Intl.DurationFormat]
---*/

var actual = [];

const options = {
  get localeMatcher() {
    actual.push("localeMatcher");
    return undefined;
  },
  get numberingSystem() {
    actual.push("numberingSystem");
    return undefined;
  },
  get style() {
    actual.push("style");
    return undefined;
  },
};

const expected = [
  "localeMatcher",
  "numberingSystem",
  "style"
];

let nf = new Intl.DurationFormat(undefined, options);
assert.compareArray(actual, expected);

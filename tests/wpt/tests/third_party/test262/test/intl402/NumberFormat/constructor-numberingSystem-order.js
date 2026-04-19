// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
  Checks the order of getting "numberingSystem" option in the
  NumberFormat is between "localeMatcher" and "style" options.
info: |
  InitializeNumberFormat ( _numberFormat_, _locales_, _options_ )

  5. Let _matcher_ be ? GetOption(_options_, `"localeMatcher"`, `"string"`, &laquo; `"lookup"`, `"best fit"` &raquo;, `"best fit"`).
  ...
  7. Let _numberingSystem_ be ? GetOption(_options_, `"numberingSystem"`, `"string"`, *undefined*, *undefined*).
  ...
  17. Let _style_ be ? GetOption(_options_, `"style"`, `"string"`, &laquo; `"decimal"`, `"percent"`, `"currency"` &raquo;, `"decimal"`).
includes: [compareArray.js]
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

let nf = new Intl.NumberFormat(undefined, options);
assert.compareArray(actual, expected);

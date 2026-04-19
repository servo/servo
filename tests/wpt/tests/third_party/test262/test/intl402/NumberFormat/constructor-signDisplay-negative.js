// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializenumberformat
description: Checks handling of the compactDisplay option to the NumberFormat constructor.
info: |
  InitializeNumberFormat ( numberFormat, locales, options )

  32. Let signDisplay be ? GetOption(options, "signDisplay", "string", « "auto", "never", "always", "exceptZero", "negative" », "auto").
  33. Set numberFormat.[[SignDisplay]] to signDisplay.
includes: [propertyHelper.js]
features: [Intl.NumberFormat-v3]
---*/

const nf = new Intl.NumberFormat([], {
  signDisplay: 'negative',
});
const resolvedOptions = nf.resolvedOptions();

verifyProperty(resolvedOptions, 'signDisplay', {
  value: 'negative',
  writable: true,
  enumerable: true,
  configurable: true
});

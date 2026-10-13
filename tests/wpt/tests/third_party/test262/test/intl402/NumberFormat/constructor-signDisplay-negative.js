// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat
description: Checks handling of the compactDisplay option to the NumberFormat constructor.
info: |
  Intl.NumberFormat ( [ locales [ , options ] ] )

  1. Let _signDisplay_ be ? GetOption(_options_, *"signDisplay"*, ~string~,
     « *"auto"*, *"never"*, *"always"*, *"exceptZero"*, *"negative"* »,
     *"auto"*).
  1. Set _numberFormat_.[[SignDisplay]] to _signDisplay_.
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

// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setnumberformatunitoptions
description: Checks handling of valid values for the numeric option to the NumberFormat constructor.
info: |
    SetNumberFormatUnitOptions ( intlObj, options )

    6. Let currencyDisplay be ? GetOption(options, "currencyDisplay", "string", « "code", "symbol", "narrowSymbol", "name" », "symbol").
    11. If style is "currency", then
        f. Set intlObj.[[CurrencyDisplay]] to currencyDisplay.

features: [Intl.NumberFormat-unified]
---*/

const validOptions = [
  [undefined, "symbol"],
  ["narrowSymbol", "narrowSymbol"],
  [{ toString() { return "narrowSymbol"; } }, "narrowSymbol"],
];

for (const [validOption, expected] of validOptions) {
  const nf = new Intl.NumberFormat([], {
    "style": "currency",
    "currency": "EUR",
    "currencyDisplay": validOption,
  });
  const resolvedOptions = nf.resolvedOptions();
  assert.sameValue(resolvedOptions.currencyDisplay, expected);
}

for (const [validOption] of validOptions) {
  const nf = new Intl.NumberFormat([], {
    "style": "percent",
    "currencyDisplay": validOption,
  });
  const resolvedOptions = nf.resolvedOptions();
  assert.sameValue(resolvedOptions.currencyDisplay, undefined);
}

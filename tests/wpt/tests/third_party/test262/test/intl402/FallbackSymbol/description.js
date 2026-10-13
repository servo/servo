// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: intl-object
description: "%Intl%.[[FallbackSymbol]] description"
info: |
  The Intl object:
  - has a [[FallbackSymbol]] internal slot, which is a new %Symbol% in the
    current realm with the [[Description]] *"IntlLegacyConstructedSymbol"*.
features: [intl-normative-optional]
---*/

const fallbackDTF = Intl.DateTimeFormat.call(Object.create(Intl.DateTimeFormat.prototype));
const symbolProps = Object.getOwnPropertySymbols(fallbackDTF);
const fallbackSymbol = symbolProps.find((sym) => sym.description === "IntlLegacyConstructedSymbol");

assert.notSameValue(
  fallbackSymbol,
  undefined,
  "%Intl%.[[FallbackSymbol]] with description IntlLegacyConstructedSymbol should exist"
);

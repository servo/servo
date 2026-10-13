// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: intl-object
description: "%Intl%.[[FallbackSymbol]] is per-realm"
info: |
  The Intl object:
  - has a [[FallbackSymbol]] internal slot, which is a new %Symbol% in the
    current realm with the [[Description]] *"IntlLegacyConstructedSymbol"*.
features: [intl-normative-optional]
---*/

const fallbackDTF = Intl.DateTimeFormat.call(Object.create(Intl.DateTimeFormat.prototype));
const symbolProps = Object.getOwnPropertySymbols(fallbackDTF);
const fallbackSymbol = symbolProps.find((sym) => sym.description === "IntlLegacyConstructedSymbol");

assert.notSameValue(fallbackSymbol, undefined, "%Intl%.[[FallbackSymbol]] should exist in original realm");

const other = $262.createRealm();
const otherFallbackSymbol = other.evalScript(`
  const fallbackDTF = Intl.DateTimeFormat.call(Object.create(Intl.DateTimeFormat.prototype));
  const symbolProps = Object.getOwnPropertySymbols(fallbackDTF);
  symbolProps.find((sym) => sym.description === "IntlLegacyConstructedSymbol");
`);

assert.notSameValue(otherFallbackSymbol, undefined, "%Intl%.[[FallbackSymbol]] should exist in new realm");

assert.notSameValue(fallbackSymbol, otherFallbackSymbol, "%Intl%.[[FallbackSymbol]] should be different per-realm");

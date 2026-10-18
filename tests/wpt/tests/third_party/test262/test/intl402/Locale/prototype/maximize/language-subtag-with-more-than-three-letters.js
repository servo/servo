// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.maximize
description: >
  Language subtags with more than three letters are supported.
info: |
  Intl.Locale.prototype.maximize ( )

  1. Let loc be the this value.
  2. Perform ? RequireInternalSlot(loc, [[InitializedLocale]]).
  3. Let maximal be the result of the Add Likely Subtags algorithm applied to loc.[[Locale]]. If an error is signaled, set maximal to loc.[[Locale]].
  4. Return ! Construct(%Intl.Locale%, « maximal »).
features: [Intl.Locale, Intl.Locale-info]
---*/

// "abcdefgh" is not a registered language, so it's not possible to infer its script
// through adding likely subtags.
assert.sameValue(
  new Intl.Locale("abcdefgh").maximize().script,
  undefined,
  `can't infer script for locale "abcdefgh" by adding likely subtags`
);

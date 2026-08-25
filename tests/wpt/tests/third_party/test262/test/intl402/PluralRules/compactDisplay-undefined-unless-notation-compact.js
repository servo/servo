// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.pluralrules
description: compactDisplay option is only set when notation is compact
info: |
  17.4 Properties of Intl.PluralRules Instances
  - [[CompactDisplay]] is one of *"short"*, *"long"*, or *undefined*, specifying
    whether to display compact notation affixes in short form ("5K") or long
    form ("5 thousand") if formatting with the *"compact"* notation, as this can
    in some cases influence plural form selection. It is only *undefined* when
    [[Notation]] has a value other than *"compact"*.
---*/

for (const compactDisplay of ["long", "short"]) {
  const compact = new Intl.PluralRules(undefined, { notation: "compact", compactDisplay });
  const compactResolved = compact.resolvedOptions();

  assert.sameValue(compactResolved.notation, "compact", `notation (compact, ${compactDisplay})`);
  assert.sameValue(compactResolved.compactDisplay, compactDisplay, `compactDisplay (compact, ${compactDisplay})`);

  const undefinedNotation = new Intl.PluralRules(undefined, { compactDisplay });
  const undefinedResolved = undefinedNotation.resolvedOptions();

  assert.sameValue(undefinedResolved.notation, "standard", "notation (standard is the default)");
  assert(!('compactDisplay' in undefinedResolved), `compactDisplay not defined for standard notation (undefined, ${compactDisplay})`);
}

for (const notation of ["standard", "scientific", "engineering"]) {
  for (const compactDisplay of ["long", "short", undefined]) {
    const pr = new Intl.PluralRules(undefined, { notation, compactDisplay });
    const resolved = pr.resolvedOptions();

    assert.sameValue(resolved.notation, notation, `notation (${notation}, ${compactDisplay})`);
    assert(!('compactDisplay' in resolved), `compactDisplay not defined (${notation}, ${compactDisplay})`);
  }
}

const compactUndefined = new Intl.PluralRules(undefined, { notation: "compact" });
const compactUndefinedResolved = compactUndefined.resolvedOptions();

assert.sameValue(compactUndefinedResolved.notation, "compact", `notation (compact, undefined)`);
assert.sameValue(compactUndefinedResolved.compactDisplay, "short", "compactDisplay (short is the default)");

const allUndefined = new Intl.PluralRules();
const allUndefinedResolved = allUndefined.resolvedOptions();

assert.sameValue(allUndefinedResolved.notation, "standard", "notation (standard is the default)");
assert(!('compactDisplay' in allUndefinedResolved), "compactDisplay not defined for standard notation");

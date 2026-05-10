// Copyright 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Do not replace $<> preceded with $
esid: sec-getsubstitution
features: [regexp-named-groups]
info: |
  Runtime Semantics: GetSubstitution( matched, str, position, captures, namedCaptures, replacement )

  12. These $ replacements are done left-to-right, and, once such a replacement is performed,
  the new replacement text is not subject to further replacements.

  Table: Replacement Text Symbol Substitutions

  Unicode Characters: $$
  Replacement text: $
---*/

let source = "(?<fst>.)";
for (let flags of ["", "u"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("$<fst>bc", "abc".replace(re, "$$<fst>"));
}

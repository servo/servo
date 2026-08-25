// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Named substitutions are found by getting the property from the groups object
esid: sec-getsubstitution
features: [regexp-named-groups]
info: |
  Runtime Semantics: GetSubstitution( matched, str, position, captures, namedCaptures, replacement )

  Table: Replacement Text Symbol Substitutions

  Unicode Characters: $<
  Replacement text:
    2. Otherwise,
      c. Let capture be ? Get(namedCaptures, groupName).
      d. If capture is undefined, replace the text through > with the empty string.
      e. Otherwise, replace the text through this following > with ? ToString(capture).
---*/

let source = "(?<fst>.)(?<snd>.)|(?<thd>x)";
for (let flags of ["g", "gu"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("badc", "abcd".replace(re, "$<snd>$<fst>"));
}
for (let flags of ["", "u"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("bacd", "abcd".replace(re, "$<snd>$<fst>"));
}

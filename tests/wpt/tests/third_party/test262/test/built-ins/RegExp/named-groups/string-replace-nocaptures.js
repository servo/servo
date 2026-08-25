// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If there are no named captures, don't replace $<>
esid: sec-getsubstitution
features: [regexp-named-groups]
info: |
  Runtime Semantics: GetSubstitution( matched, str, position, captures, namedCaptures, replacement )

  Table: Replacement Text Symbol Substitutions

  Unicode Characters: $<
  Replacement text:
    1. If namedCaptures is undefined, the replacement text is the literal string $<.
---*/

// @@replace with a string replacement argument (no named captures).

let source = "(.)(.)|(x)";
for (let flags of ["", "u"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("$<snd>$<fst>cd", "abcd".replace(re, "$<snd>$<fst>"));
  assert.sameValue("bacd", "abcd".replace(re, "$2$1"));
  assert.sameValue("cd", "abcd".replace(re, "$3"));
  assert.sameValue("$<sndcd", "abcd".replace(re, "$<snd"));
  assert.sameValue("$<sndacd", "abcd".replace(re, "$<snd$1"));
  assert.sameValue("$<42a>cd", "abcd".replace(re, "$<42$1>"));
  assert.sameValue("$<fth>cd", "abcd".replace(re, "$<fth>"));
  assert.sameValue("$<a>cd", "abcd".replace(re, "$<$1>"));
}


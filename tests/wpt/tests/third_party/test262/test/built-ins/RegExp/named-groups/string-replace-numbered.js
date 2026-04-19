// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Named groups may be accessed in their replacement string by number
esid: sec-getsubstitution
features: [regexp-named-groups]
info: |
  Runtime Semantics: GetSubstitution( matched, str, position, captures, namedCaptures, replacement )

  Table: Replacement Text Symbol Substitutions

  Unicode Characters: $n
  Replacement text:
    The nth element of captures, where n is a single digit in the range 1 to 9. If
    n≤m and the nth element of captures is undefined, use the empty String instead.
    If n>m, the result is implementation-defined.
---*/

let source = "(?<fst>.)(?<snd>.)|(?<thd>x)";
for (let flags of ["g", "gu"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("badc", "abcd".replace(re, "$2$1"));
}
for (let flags of ["", "u"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("bacd", "abcd".replace(re, "$2$1"));
}


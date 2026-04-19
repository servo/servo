// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Function argument to String.prototype.replace gets groups as the last argument
esid: sec-regexp.prototype-@@replace
features: [regexp-named-groups]
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )
    14. Repeat, for each result in results,
      j. Let namedCaptures be ? Get(result, "groups").
      k. If functionalReplace is true, then
        iv. If namedCaptures is not undefined,
          1. Append namedCaptures as the last element of replacerArgs.
---*/

let source = "(?<fst>.)(?<snd>.)";
let alternateSource = "(?<fst>.)|(?<snd>.)";

for (let flags of ["", "u"]) {
  let i = 0;
  let re = new RegExp(source, flags);
  let result = "abcd".replace(re,
      (match, fst, snd, offset, str, groups) => {
    assert.sameValue(i++, 0);
    assert.sameValue("ab", match);
    assert.sameValue("a", groups.fst);
    assert.sameValue("b", groups.snd);
    assert.sameValue("a", fst);
    assert.sameValue("b", snd);
    assert.sameValue(0, offset);
    assert.sameValue("abcd", str);
    return `${groups.snd}${groups.fst}`;
  });
  assert.sameValue("bacd", result);
  assert.sameValue(i, 1);

  let re2 = new RegExp(alternateSource, flags);
  assert.sameValue("undefinedbcd",
      "abcd".replace(re2,
            (match, fst, snd, offset, str, groups) => groups.snd));
}

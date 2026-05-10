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

for (let flags of ["g", "gu"]) {
  let i = 0;
  let re = new RegExp(source, flags);
  let result = "abcd".replace(re,
      (match, fst, snd, offset, str, groups) => {
    if (i == 0) {
      assert.sameValue("ab", match);
      assert.sameValue("a", groups.fst);
      assert.sameValue("b", groups.snd);
      assert.sameValue("a", fst);
      assert.sameValue("b", snd);
      assert.sameValue(0, offset);
      assert.sameValue("abcd", str);
    } else if (i == 1) {
      assert.sameValue("cd", match);
      assert.sameValue("c", groups.fst);
      assert.sameValue("d", groups.snd);
      assert.sameValue("c", fst);
      assert.sameValue("d", snd);
      assert.sameValue(2, offset);
      assert.sameValue("abcd", str);
    } else {
      assertUnreachable();
    }
    i++;
    return `${groups.snd}${groups.fst}`;
  });
  assert.sameValue("badc", result);
  assert.sameValue(i, 2);

  let re2 = new RegExp(alternateSource, flags);
  assert.sameValue("undefinedundefinedundefinedundefined",
      "abcd".replace(re2,
            (match, fst, snd, offset, str, groups) => groups.snd));
}


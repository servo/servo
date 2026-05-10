// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  sticky flag should not break assertion behavior.
info: bugzilla.mozilla.org/show_bug.cgi?id=773687
esid: pending
---*/

function test(re, text, expectations) {
  // Sanity check for test data itself.
  assert.sameValue(expectations.length, text.length + 1);

  for (var i = 0; i < expectations.length; i++) {
    var result = expectations[i];

    re.lastIndex = i;
    var match = re.exec(text);
    if (result === null) {
      assert.sameValue(re.lastIndex, 0);
      assert.sameValue(match, null);
    } else {
      assert.sameValue(re.lastIndex, result.lastIndex);
      assert.sameValue(match !== null, true);
      assert.sameValue(match.length, result.matches.length);
      for (var j = 0; j < result.matches.length; j++)
        assert.sameValue(match[j], result.matches[j]);
      assert.sameValue(match.index, result.index);
    }
  }
}

// simple text
test(/bc/y, "abcabd", [
  null,
  { lastIndex: 3, matches: ["bc"], index: 1 },
  null,
  null,
  null,
  null,
  null,
]);

// complex pattern
test(/bc|c|d/y, "abcabd", [
  null,
  { lastIndex: 3, matches: ["bc"], index: 1 },
  { lastIndex: 3, matches: ["c"], index: 2 },
  null,
  null,
  { lastIndex: 6, matches: ["d"], index: 5 },
  null,
]);

test(/.*(bc|c|d)/y, "abcabd", [
  { lastIndex: 6, matches: ["abcabd", "d"], index: 0 },
  { lastIndex: 6, matches: ["bcabd", "d"], index: 1 },
  { lastIndex: 6, matches: ["cabd", "d"], index: 2 },
  { lastIndex: 6, matches: ["abd", "d"], index: 3 },
  { lastIndex: 6, matches: ["bd", "d"], index: 4 },
  { lastIndex: 6, matches: ["d", "d"], index: 5 },
  null,
]);

test(/.*?(bc|c|d)/y, "abcabd", [
  { lastIndex: 3, matches: ["abc", "bc"], index: 0 },
  { lastIndex: 3, matches: ["bc", "bc"], index: 1 },
  { lastIndex: 3, matches: ["c", "c"], index: 2 },
  { lastIndex: 6, matches: ["abd", "d"], index: 3 },
  { lastIndex: 6, matches: ["bd", "d"], index: 4 },
  { lastIndex: 6, matches: ["d", "d"], index: 5 },
  null,
]);

test(/(bc|.*c|d)/y, "abcabd", [
  { lastIndex: 3, matches: ["abc", "abc"], index: 0 },
  { lastIndex: 3, matches: ["bc", "bc"], index: 1 },
  { lastIndex: 3, matches: ["c", "c"], index: 2 },
  null,
  null,
  { lastIndex: 6, matches: ["d", "d"], index: 5 },
  null,
]);

// ^ assertions
test(/^/y, "abcabc", [
  { lastIndex: 0, matches: [""], index: 0 },
  null,
  null,
  null,
  null,
  null,
  null,
]);

test(/^a/my, "abc\nabc", [
  { lastIndex: 1, matches: ["a"], index: 0 },
  null,
  null,
  null,
  { lastIndex: 5, matches: ["a"], index: 4 },
  null,
  null,
  null,
]);

// \b assertions
test(/\b/y, "abc bc", [
  { lastIndex: 0, matches: [""], index: 0 },
  null,
  null,
  { lastIndex: 3, matches: [""], index: 3 },
  { lastIndex: 4, matches: [""], index: 4 },
  null,
  { lastIndex: 6, matches: [""], index: 6 },
]);

// \B assertions
test(/\B/y, "abc bc", [
  null,
  { lastIndex: 1, matches: [""], index: 1 },
  { lastIndex: 2, matches: [""], index: 2 },
  null,
  null,
  { lastIndex: 5, matches: [""], index: 5 },
  null,
]);

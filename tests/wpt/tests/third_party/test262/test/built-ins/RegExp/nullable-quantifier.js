// Copyright (C) 2024 Aurèle Barrière. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-repeatmatcher-abstract-operation
description: JavaScript nullable quantifiers have special semantics, optional iterations are not allowed to match the empty string. Point 2.b below shows that after each optional (min=0) iteration of a quantifier, if the iteration matched the empty string (y.[[EndIndex]] = x.[[EndIndex]]), then the iteration is discarded. In particular, for (a?b??)* on "ab", it is possible to do two iterations of the star, one matching "a" and the other matching "b".
info: |
    RepeatMatcher ( m, min, max, greedy, x, c, parenIndex, parenCount )

    2. Let d be a new MatcherContinuation with parameters (y) that captures m, min, max, greedy, x, c, parenIndex, and parenCount and performs the following steps when called:

      b. If min = 0 and y.[[EndIndex]] = x.[[EndIndex]], return failure.
author: Aurèle Barrière
---*/

let input = "ab";
let regex = /(a?b??)*/;
let match = regex.exec(input);
let expected = "ab";

assert.sameValue(match[0], expected, "The regex is expected to match the whole string");

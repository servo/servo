/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

let v = "global-v";

function f(v, global)
{
  with (global)
    return v;
}

assert.sameValue(f("argument-v", this), "argument-v",
         "let-var shouldn't appear in global for |with| purposes");

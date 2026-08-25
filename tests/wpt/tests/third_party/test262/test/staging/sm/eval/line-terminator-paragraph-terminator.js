/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  eval via the JSON parser should parse strings containing U+2028/U+2029 (as of <https://tc39.github.io/proposal-json-superset/>, that is)
info: bugzilla.mozilla.org/show_bug.cgi?id=657367
esid: pending
---*/

assert.sameValue(eval('("\u2028")'), "\u2028");
assert.sameValue(eval('("\u2029")'), "\u2029");

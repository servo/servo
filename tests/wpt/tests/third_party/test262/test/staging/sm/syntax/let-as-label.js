/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  let can't be used as a label in strict mode code
info: bugzilla.mozilla.org/show_bug.cgi?id=1288459
esid: pending
---*/

Function("let: 42");
Function("l\\u0065t: 42");
assert.throws(SyntaxError, () => Function(" 'use strict'; let: 42"));
assert.throws(SyntaxError, () => Function(" 'use strict' \n let: 42"));
assert.throws(SyntaxError, () => Function(" 'use strict'; l\\u0065t: 42"));
assert.throws(SyntaxError, () => Function(" 'use strict' \n l\\u0065t: 42"));

eval("let: 42");
eval("l\\u0065t: 42");
assert.throws(SyntaxError, () => eval(" 'use strict'; let: 42"));
assert.throws(SyntaxError, () => eval(" 'use strict' \n let: 42;"));
assert.throws(SyntaxError, () => eval(" 'use strict'; l\\u0065t: 42"));
assert.throws(SyntaxError, () => eval(" 'use strict' \n l\\u0065t: 42;"));

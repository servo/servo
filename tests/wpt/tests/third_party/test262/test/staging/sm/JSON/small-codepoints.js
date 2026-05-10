/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-JSON-shell.js]
description: |
  JSON.parse should reject U+0000 through U+001F
esid: pending
---*/

for (var i = 0; i <= 0x1F; i++)
  testJSONSyntaxError('["a' + String.fromCharCode(i) + 'c"]');

/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  JSON.parse handling of omitted arguments
info: bugzilla.mozilla.org/show_bug.cgi?id=653847
esid: pending
---*/

assert.throws(SyntaxError, function() {
  JSON.parse();
});

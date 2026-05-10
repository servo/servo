/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

function throws(code) {
  assert.throws(SyntaxError, function() {
    eval(code);
  });
}

var s = '\\u0073';
throws('var thi' + s);
throws('switch (' + s + 'witch) {}')
throws('var ' + s + 'witch');

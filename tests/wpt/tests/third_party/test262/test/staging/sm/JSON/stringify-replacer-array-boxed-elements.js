/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Boxed-string/number objects in replacer arrays
info: bugzilla.mozilla.org/show_bug.cgi?id=648471
esid: pending
---*/

var S = new String(3);
var N = new Number(4);

assert.sameValue(JSON.stringify({ 3: 3, 4: 4 }, [S]),
         '{"3":3}');
assert.sameValue(JSON.stringify({ 3: 3, 4: 4 }, [N]),
         '{"4":4}');

Number.prototype.toString = function() { return 3; };
assert.sameValue(JSON.stringify({ 3: 3, 4: 4 }, [N]),
         '{"3":3}');

String.prototype.toString = function() { return 4; };
assert.sameValue(JSON.stringify({ 3: 3, 4: 4 }, [S]),
         '{"4":4}');

Number.prototype.toString = function() { return new String(42); };
assert.sameValue(JSON.stringify({ 3: 3, 4: 4 }, [N]),
         '{"4":4}');

String.prototype.toString = function() { return new Number(17); };
assert.sameValue(JSON.stringify({ 3: 3, 4: 4 }, [S]),
         '{"3":3}');

Number.prototype.toString = null;
assert.sameValue(JSON.stringify({ 3: 3, 4: 4 }, [N]),
         '{"4":4}');

String.prototype.toString = null;
assert.sameValue(JSON.stringify({ 3: 3, 4: 4 }, [S]),
         '{"3":3}');

Number.prototype.valueOf = function() { return 17; };
assert.sameValue(JSON.stringify({ 4: 4, 17: 17 }, [N]),
         '{"17":17}');

String.prototype.valueOf = function() { return 42; };
assert.sameValue(JSON.stringify({ 3: 3, 42: 42 }, [S]),
         '{"42":42}');

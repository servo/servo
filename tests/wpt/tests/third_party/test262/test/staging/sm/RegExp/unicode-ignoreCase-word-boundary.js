// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var BUGNUMBER = 1338373;
var summary = "Word boundary should match U+017F and U+212A in unicode+ignoreCase.";

assert.sameValue(/\b/iu.test('\u017F'), true);
assert.sameValue(/\b/i.test('\u017F'), false);
assert.sameValue(/\b/u.test('\u017F'), false);
assert.sameValue(/\b/.test('\u017F'), false);

assert.sameValue(/\b/iu.test('\u212A'), true);
assert.sameValue(/\b/i.test('\u212A'), false);
assert.sameValue(/\b/u.test('\u212A'), false);
assert.sameValue(/\b/.test('\u212A'), false);

assert.sameValue(/\B/iu.test('\u017F'), false);
assert.sameValue(/\B/i.test('\u017F'), true);
assert.sameValue(/\B/u.test('\u017F'), true);
assert.sameValue(/\B/.test('\u017F'), true);

assert.sameValue(/\B/iu.test('\u212A'), false);
assert.sameValue(/\B/i.test('\u212A'), true);
assert.sameValue(/\B/u.test('\u212A'), true);
assert.sameValue(/\B/.test('\u212A'), true);

// Bug 1338779 - More testcases.
assert.sameValue(/(i\B\u017F)/ui.test("is"), true);
assert.sameValue(/(i\B\u017F)/ui.test("it"), false);
assert.sameValue(/(i\B\u017F)+/ui.test("is"), true);
assert.sameValue(/(i\B\u017F)+/ui.test("it"), false);

assert.sameValue(/(\u017F\Bi)/ui.test("si"), true);
assert.sameValue(/(\u017F\Bi)/ui.test("ti"), false);
assert.sameValue(/(\u017F\Bi)+/ui.test("si"), true);
assert.sameValue(/(\u017F\Bi)+/ui.test("ti"), false);

assert.sameValue(/(i\B\u212A)/ui.test("ik"), true);
assert.sameValue(/(i\B\u212A)/ui.test("it"), false);
assert.sameValue(/(i\B\u212A)+/ui.test("ik"), true);
assert.sameValue(/(i\B\u212A)+/ui.test("it"), false);

assert.sameValue(/(\u212A\Bi)/ui.test("ki"), true);
assert.sameValue(/(\u212A\Bi)/ui.test("ti"), false);
assert.sameValue(/(\u212A\Bi)+/ui.test("ki"), true);
assert.sameValue(/(\u212A\Bi)+/ui.test("ti"), false);


// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var BUGNUMBER = 1338779;
var summary = "Non-Latin1 to Latin1 mapping in ignoreCase.";

assert.sameValue(/(\u039C)/.test("\xB5"), false);
assert.sameValue(/(\u039C)+/.test("\xB5"), false);
assert.sameValue(/(\u039C)/i.test("\xB5"), true);
assert.sameValue(/(\u039C)+/i.test("\xB5"), true);
assert.sameValue(/(\u039C)/u.test("\xB5"), false);
assert.sameValue(/(\u039C)+/u.test("\xB5"), false);
assert.sameValue(/(\u039C)/ui.test("\xB5"), true);
assert.sameValue(/(\u039C)+/ui.test("\xB5"), true);

assert.sameValue(/(\xB5)/.test("\u039C"), false);
assert.sameValue(/(\xB5)+/.test("\u039C"), false);
assert.sameValue(/(\xB5)/i.test("\u039C"), true);
assert.sameValue(/(\xB5)+/i.test("\u039C"), true);
assert.sameValue(/(\xB5)/u.test("\u039C"), false);
assert.sameValue(/(\xB5)+/u.test("\u039C"), false);
assert.sameValue(/(\xB5)/ui.test("\u039C"), true);
assert.sameValue(/(\xB5)+/ui.test("\u039C"), true);


assert.sameValue(/(\u0178)/.test("\xFF"), false);
assert.sameValue(/(\u0178)+/.test("\xFF"), false);
assert.sameValue(/(\u0178)/i.test("\xFF"), true);
assert.sameValue(/(\u0178)+/i.test("\xFF"), true);
assert.sameValue(/(\u0178)/u.test("\xFF"), false);
assert.sameValue(/(\u0178)+/u.test("\xFF"), false);
assert.sameValue(/(\u0178)/ui.test("\xFF"), true);
assert.sameValue(/(\u0178)+/ui.test("\xFF"), true);

assert.sameValue(/(\xFF)/.test("\u0178"), false);
assert.sameValue(/(\xFF)+/.test("\u0178"), false);
assert.sameValue(/(\xFF)/i.test("\u0178"), true);
assert.sameValue(/(\xFF)+/i.test("\u0178"), true);
assert.sameValue(/(\xFF)/u.test("\u0178"), false);
assert.sameValue(/(\xFF)+/u.test("\u0178"), false);
assert.sameValue(/(\xFF)/ui.test("\u0178"), true);
assert.sameValue(/(\xFF)+/ui.test("\u0178"), true);


assert.sameValue(/(\u017F)/.test("\x73"), false);
assert.sameValue(/(\u017F)+/.test("\x73"), false);
assert.sameValue(/(\u017F)/i.test("\x73"), false);
assert.sameValue(/(\u017F)+/i.test("\x73"), false);
assert.sameValue(/(\u017F)/u.test("\x73"), false);
assert.sameValue(/(\u017F)+/u.test("\x73"), false);
assert.sameValue(/(\u017F)/iu.test("\x73"), true);
assert.sameValue(/(\u017F)+/iu.test("\x73"), true);

assert.sameValue(/(\x73)/.test("\u017F"), false);
assert.sameValue(/(\x73)+/.test("\u017F"), false);
assert.sameValue(/(\x73)/i.test("\u017F"), false);
assert.sameValue(/(\x73)+/i.test("\u017F"), false);
assert.sameValue(/(\x73)/u.test("\u017F"), false);
assert.sameValue(/(\x73)+/u.test("\u017F"), false);
assert.sameValue(/(\x73)/iu.test("\u017F"), true);
assert.sameValue(/(\x73)+/iu.test("\u017F"), true);


assert.sameValue(/(\u1E9E)/.test("\xDF"), false);
assert.sameValue(/(\u1E9E)+/.test("\xDF"), false);
assert.sameValue(/(\u1E9E)/i.test("\xDF"), false);
assert.sameValue(/(\u1E9E)+/i.test("\xDF"), false);
assert.sameValue(/(\u1E9E)/u.test("\xDF"), false);
assert.sameValue(/(\u1E9E)+/u.test("\xDF"), false);
assert.sameValue(/(\u1E9E)/iu.test("\xDF"), true);
assert.sameValue(/(\u1E9E)+/iu.test("\xDF"), true);

assert.sameValue(/(\xDF)/.test("\u1E9E"), false);
assert.sameValue(/(\xDF)+/.test("\u1E9E"), false);
assert.sameValue(/(\xDF)/i.test("\u1E9E"), false);
assert.sameValue(/(\xDF)+/i.test("\u1E9E"), false);
assert.sameValue(/(\xDF)/u.test("\u1E9E"), false);
assert.sameValue(/(\xDF)+/u.test("\u1E9E"), false);
assert.sameValue(/(\xDF)/iu.test("\u1E9E"), true);
assert.sameValue(/(\xDF)+/iu.test("\u1E9E"), true);


assert.sameValue(/(\u212A)/.test("\x6B"), false);
assert.sameValue(/(\u212A)+/.test("\x6B"), false);
assert.sameValue(/(\u212A)/i.test("\x6B"), false);
assert.sameValue(/(\u212A)+/i.test("\x6B"), false);
assert.sameValue(/(\u212A)/u.test("\x6B"), false);
assert.sameValue(/(\u212A)+/u.test("\x6B"), false);
assert.sameValue(/(\u212A)/iu.test("\x6B"), true);
assert.sameValue(/(\u212A)+/iu.test("\x6B"), true);

assert.sameValue(/(\x6B)/.test("\u212A"), false);
assert.sameValue(/(\x6B)+/.test("\u212A"), false);
assert.sameValue(/(\x6B)/i.test("\u212A"), false);
assert.sameValue(/(\x6B)+/i.test("\u212A"), false);
assert.sameValue(/(\x6B)/u.test("\u212A"), false);
assert.sameValue(/(\x6B)+/u.test("\u212A"), false);
assert.sameValue(/(\x6B)/iu.test("\u212A"), true);
assert.sameValue(/(\x6B)+/iu.test("\u212A"), true);


assert.sameValue(/(\u212B)/.test("\xE5"), false);
assert.sameValue(/(\u212B)+/.test("\xE5"), false);
assert.sameValue(/(\u212B)/i.test("\xE5"), false);
assert.sameValue(/(\u212B)+/i.test("\xE5"), false);
assert.sameValue(/(\u212B)/u.test("\xE5"), false);
assert.sameValue(/(\u212B)+/u.test("\xE5"), false);
assert.sameValue(/(\u212B)/iu.test("\xE5"), true);
assert.sameValue(/(\u212B)+/iu.test("\xE5"), true);

assert.sameValue(/(\xE5)/.test("\u212B"), false);
assert.sameValue(/(\xE5)+/.test("\u212B"), false);
assert.sameValue(/(\xE5)/i.test("\u212B"), false);
assert.sameValue(/(\xE5)+/i.test("\u212B"), false);
assert.sameValue(/(\xE5)/u.test("\u212B"), false);
assert.sameValue(/(\xE5)+/u.test("\u212B"), false);
assert.sameValue(/(\xE5)/iu.test("\u212B"), true);
assert.sameValue(/(\xE5)+/iu.test("\u212B"), true);


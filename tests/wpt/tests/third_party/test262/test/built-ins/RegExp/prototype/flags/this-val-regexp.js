// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: >
  RegExp.prototype.flags returns RegExp flags as a string
info: |
  3. Let result be the empty String.
  4. Let global be ToBoolean(? Get(R, "global")).
  5. If global is true, append "g" as the last code unit of result.
  6. Let ignoreCase be ToBoolean(? Get(R, "ignoreCase")).
  7. If ignoreCase is true, append "i" as the last code unit of result.
  8. Let multiline be ToBoolean(? Get(R, "multiline")).
  9. If multiline is true, append "m" as the last code unit of result.
  10. Let dotAll be ToBoolean(? Get(R, "dotAll")).
  11. If dotAll is true, append "s" as the last code unit of result.
  12. Let unicode be ToBoolean(? Get(R, "unicode")).
  13. If unicode is true, append "u" as the last code unit of result.
  14. Let unicodeSets be ! ToBoolean(? Get(R, "unicodeSets")).
  15. If unicodeSets is true, append "v" as the last code unit of result.
  16. Let sticky be ToBoolean(? Get(R, "sticky")).
  17. If sticky is true, append "y" as the last code unit of result.
  18. Return result.
features: [regexp-dotall, regexp-match-indices, regexp-v-flag]
---*/

assert.sameValue(/./.flags, '', 'no flags');
assert.sameValue(/./d.flags, 'd', 'hasIndices');
assert.sameValue(/./g.flags, 'g', 'global');
assert.sameValue(/./i.flags, 'i', 'ignoreCase');
assert.sameValue(/./m.flags, 'm', 'multiline');
assert.sameValue(/./s.flags, 's', 'dotAll');
assert.sameValue(/./u.flags, 'u', 'unicode');
assert.sameValue(/./v.flags, 'v', 'unicodeSets');
assert.sameValue(/./y.flags, 'y', 'sticky');

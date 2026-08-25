// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: >
  RegExp.prototype.flags come in a single order, independent of source order
info: |
  4. Let global be ToBoolean(? Get(R, "global")).
  5. If global is true, append "g" as the last code unit of result.
  6. Let global be ToBoolean(? Get(R, "global")).
  7. If global is true, append "g" as the last code unit of result.
  8. Let ignoreCase be ToBoolean(? Get(R, "ignoreCase")).
  9. If ignoreCase is true, append "i" as the last code unit of result.
  10. Let multiline be ToBoolean(? Get(R, "multiline")).
  11. If multiline is true, append "m" as the last code unit of result.
  12. Let dotAll be ToBoolean(? Get(R, "dotAll")).
  13. If dotAll is true, append "s" as the last code unit of result.
  14. Let unicode be ToBoolean(? Get(R, "unicode")).
  15. If unicode is true, append "u" as the last code unit of result.
  16. Let sticky be ToBoolean(? Get(R, "sticky")).
  17. If sticky is true, append "y" as the last code unit of result.
features: [regexp-dotall, regexp-match-indices]
---*/

assert.sameValue(new RegExp("", "dgimsuy").flags, "dgimsuy", "dgimsuy => dgimsuy");
assert.sameValue(new RegExp("", "yusmigd").flags, "dgimsuy", "yusmigd => dgimsuy");

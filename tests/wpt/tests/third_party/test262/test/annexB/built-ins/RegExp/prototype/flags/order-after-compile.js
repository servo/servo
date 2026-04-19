// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: >
  The flags come in the same order in a new instance produced by RegExp.prototype.compile
info: |
  B.2.5.1 RegExp.prototype.compile ( pattern, flags )

  ...
  5. Return ? RegExpInitialize(O, P, F).

  21.2.5.3 get RegExp.prototype.flags

  ...
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
  14. Let sticky be ToBoolean(? Get(R, "sticky")).
  15. If sticky is true, append "y" as the last code unit of result.
  14. Return result.
features: [regexp-dotall]
---*/

let re = /(?:)/;
re.compile("(?:)", "imsuyg");
assert.sameValue(re.flags, "gimsuy");

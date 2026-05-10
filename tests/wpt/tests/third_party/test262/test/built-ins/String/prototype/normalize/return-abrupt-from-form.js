// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.12
description: >
  Returns abrupt from ToString(form)
info: |
  21.1.3.12 String.prototype.normalize ( [ form ] )

  ...
  4. If form is not provided or form is undefined, let form be "NFC".
  5. Let f be ToString(form).
  6. ReturnIfAbrupt(f).
---*/

var o = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  'foo'.normalize(o);
});

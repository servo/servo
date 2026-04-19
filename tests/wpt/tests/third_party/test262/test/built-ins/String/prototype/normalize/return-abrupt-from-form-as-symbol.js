// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.12
description: >
  Returns abrupt from ToString(form) as a Symbol.
info: |
  21.1.3.12 String.prototype.normalize ( [ form ] )

  ...
  4. If form is not provided or form is undefined, let form be "NFC".
  5. Let f be ToString(form).
  6. ReturnIfAbrupt(f).
features: [Symbol]
---*/

var s = Symbol('foo');

assert.throws(TypeError, function() {
  'foo'.normalize(s);
});

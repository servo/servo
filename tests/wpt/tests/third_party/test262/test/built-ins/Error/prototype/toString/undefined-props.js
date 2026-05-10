// Copyright (C) 2019 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.prototype.tostring
description: >
  Error.prototype.toString handles this.name and this.message being undefined.
info: |
  Error.prototype.toString ( )
  ...
  3. Let name be ? Get(O, "name").
  4. If name is undefined, set name to "Error"; otherwise set name to ? ToString(name).
  5. Let msg be ? Get(O, "message").
  6. If msg is undefined, set msg to the empty String; otherwise set msg to ? ToString(msg).
---*/

assert.sameValue(Error.prototype.toString.call({}), 'Error');
assert.sameValue(Error.prototype.toString.call({ message: '42' }), 'Error: 42');
assert.sameValue(Error.prototype.toString.call({ name: '24' }), '24');
assert.sameValue(Error.prototype.toString.call({ name: '24', message: '42' }), '24: 42');

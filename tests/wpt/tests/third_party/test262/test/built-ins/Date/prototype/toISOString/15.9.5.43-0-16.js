// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.toisostring
description: >
    Date.prototype.toISOString - when this is a String object that
    value format is 'YYYY-MM-DDTHH:mm:ss.sssZ'
    Date.prototype.toISOString throw the TypeError
---*/

var date = new String("1970-01-00000:00:00.000Z");
assert.throws(TypeError, function() {
  Date.prototype.toISOString.call(date);
});

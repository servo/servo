/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [detachArrayBuffer.js]
description: |
  Don't assert assigning into memory detached while converting the value to assign into a number
info: bugzilla.mozilla.org/show_bug.cgi?id=1001547
esid: pending
---*/

var ab = new ArrayBuffer(64);
var ta = new Uint32Array(ab);
ta[4] = { valueOf() { $DETACHBUFFER(ab); return 5; } };

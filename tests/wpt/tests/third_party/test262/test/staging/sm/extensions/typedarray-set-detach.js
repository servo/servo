/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [detachArrayBuffer.js]
description: |
  Uint8Array.prototype.set issues when this array changes during setting
info: bugzilla.mozilla.org/show_bug.cgi?id=983344
esid: pending
features: [host-gc-required]
---*/

var ab = new ArrayBuffer(200);
var a = new Uint8Array(ab);
var a_2 = new Uint8Array(10);

var src = [ 10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40,
            10, 20, 30, 40,
            ];
Object.defineProperty(src, 4, {
  get: function () {
    $DETACHBUFFER(ab);
    $262.gc();
    return 200;
  }
});

a.set(src);

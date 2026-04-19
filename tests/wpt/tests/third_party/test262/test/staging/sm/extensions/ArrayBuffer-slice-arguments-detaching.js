/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [detachArrayBuffer.js]
description: |
  ArrayBuffer.prototype.slice shouldn't misbehave horribly if index-argument conversion detaches the ArrayBuffer being sliced
info: bugzilla.mozilla.org/show_bug.cgi?id=991981
esid: pending
features: [host-gc-required]
---*/

function testStart()
{
  var ab = new ArrayBuffer(0x1000);

  var start =
    {
      valueOf: function()
      {
        $DETACHBUFFER(ab);
        $262.gc();
        return 0x800;
      }
    };

  assert.throws(TypeError, function() {
    ab.slice(start);
  }, "start weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for start weirdness");
}
testStart();

function testEnd()
{
  var ab = new ArrayBuffer(0x1000);

  var end =
    {
      valueOf: function()
      {
        $DETACHBUFFER(ab);
        $262.gc();
        return 0x1000;
      }
    };

  assert.throws(TypeError, function() {
    ab.slice(0x800, end);
  }, "byteLength weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for byteLength weirdness");
}
testEnd();

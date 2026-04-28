/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [detachArrayBuffer.js]
description: |
  new DataView(...) shouldn't misbehave horribly if index-argument conversion detaches the ArrayBuffer to be viewed
info: bugzilla.mozilla.org/show_bug.cgi?id=991981
esid: pending
features: [host-gc-required]
---*/

function testByteOffset()
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
    new DataView(ab, start);
  }, "byteOffset weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for byteOffset weirdness");
}
testByteOffset();

function testByteLength()
{
  var ab = new ArrayBuffer(0x1000);

  var len =
    {
      valueOf: function()
      {
        $DETACHBUFFER(ab);
        $262.gc();
        return 0x800;
      }
    };

  assert.throws(TypeError, function() {
    new DataView(ab, 0x800, len);
  }, "byteLength weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for byteLength weirdness");
}
testByteLength();

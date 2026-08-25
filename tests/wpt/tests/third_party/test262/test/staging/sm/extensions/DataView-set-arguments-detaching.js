/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [detachArrayBuffer.js]
description: |
  DataView.prototype.set* methods shouldn't misbehave horribly if index-argument conversion detaches the ArrayBuffer being modified
info: bugzilla.mozilla.org/show_bug.cgi?id=991981
esid: pending
features: [host-gc-required]
---*/

function testIndex()
{
  var ab = new ArrayBuffer(0x1000);

  var dv = new DataView(ab);

  var start =
    {
      valueOf: function()
      {
        $DETACHBUFFER(ab);
        $262.gc();
        return 0xFFF;
      }
    };

  assert.throws(TypeError, function() {
    dv.setUint8(start, 0x42);
  });
  assert.sameValue(ab.byteLength, 0, "should have been detached correctly");
}
testIndex();

function testValue()
{
  var ab = new ArrayBuffer(0x100000);

  var dv = new DataView(ab);

  var value =
    {
      valueOf: function()
      {
        $DETACHBUFFER(ab);
        $262.gc();
        return 0x42;
      }
    };

  assert.throws(TypeError, function() {
    dv.setUint8(0xFFFFF, value);
  });
  assert.sameValue(ab.byteLength, 0, "should have been detached correctly");
}
testValue();

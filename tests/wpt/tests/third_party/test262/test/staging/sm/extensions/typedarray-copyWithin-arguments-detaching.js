/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [detachArrayBuffer.js]
description: |
  %TypedArray.prototype.copyWithin shouldn't misbehave horribly if index-argument conversion detaches the underlying ArrayBuffer
info: bugzilla.mozilla.org/show_bug.cgi?id=991981
esid: pending
---*/

function testBegin()
{
  var ab = new ArrayBuffer(0x1000);

  var begin =
    {
      valueOf: function()
      {
        $DETACHBUFFER(ab);
        return 0x800;
      }
    };

  var ta = new Uint8Array(ab);

  assert.throws(TypeError, function() {
    ta.copyWithin(0, begin, 0x1000);
  }, "begin weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for begin weirdness");
}
testBegin();

function testEnd()
{
  var ab = new ArrayBuffer(0x1000);

  var end =
    {
      valueOf: function()
      {
        $DETACHBUFFER(ab);
        return 0x1000;
      }
    };

  var ta = new Uint8Array(ab);

  assert.throws(TypeError, function() {
    ta.copyWithin(0, 0x800, end);
  }, "end weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for end weirdness");
}
testEnd();

function testDest()
{
  var ab = new ArrayBuffer(0x1000);

  var dest =
    {
      valueOf: function()
      {
        $DETACHBUFFER(ab);
        return 0;
      }
    };

  var ta = new Uint8Array(ab);

  assert.throws(TypeError, function() {
    ta.copyWithin(dest, 0x800, 0x1000);
  }, "dest weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for dest weirdness");
}
testDest();

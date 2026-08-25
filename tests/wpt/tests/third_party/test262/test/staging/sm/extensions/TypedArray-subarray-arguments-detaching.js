/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [detachArrayBuffer.js]
description: |
  %TypedArray.prototype.subarray shouldn't misbehave horribly if index-argument conversion detaches the underlying ArrayBuffer
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
    ta.subarray(begin)
  }, "start weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for start weirdness");
}
testBegin();

function testBeginWithEnd()
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
    ta.subarray(begin, 0x1000);
  }, "start weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for start weirdness");
}
testBeginWithEnd();

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
    ta.subarray(0x800, end);
  }, "end weirdness should have thrown");
  assert.sameValue(ab.byteLength, 0, "detaching should work for end weirdness");
}
testEnd();

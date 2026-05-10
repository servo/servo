/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Implement ES6 binary literals
info: bugzilla.mozilla.org/show_bug.cgi?id=894026
esid: pending
---*/

var chars = ['b', 'B'];

for (var i = 0; i < 2; i++)
{
  if (i === 2)
  {
    chars.forEach(function(v)
    {
      assert.throws(SyntaxError, function() {
        eval('0' + v + i);
      }, "no syntax error evaluating 0" + v + i);
    });
    continue;
  }

  for (var j = 0; j < 2; j++)
  {
    if (j === 2)
    {
      chars.forEach(function(v)
      {
        assert.throws(SyntaxError, function() {
          eval('0' + v + i + j);
        }, "no syntax error evaluating 0" + v + i + j);
      });
      continue;
    }

    for (var k = 0; k < 2; k++)
    {
      if (k === 2)
      {
        chars.forEach(function(v)
        {
          assert.throws(SyntaxError, function() {
            eval('0' + v + i + j + k);
          }, "no syntax error evaluating 0" + v + i + j + k);
        });
        continue;
      }

      chars.forEach(function(v)
      {
        assert.sameValue(eval('0' + v + i + j + k), i * 4 + j * 2 + k);
      });
    }
  }
}

chars.forEach(function(v)
{
  assert.throws(SyntaxError, function() {
    eval('0' + v);
  }, "no syntax error evaluating 0" + v);
});

// Off-by-one check: '/' immediately precedes '0'.
assert.sameValue(0b110/1, 6);
assert.sameValue(0B10110/1, 22);

function strict()
{
  "use strict";
  return 0b11010101;
}
assert.sameValue(strict(), 128 + 64 + 16 + 4 + 1);

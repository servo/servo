/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Implement ES6 octal literals
info: bugzilla.mozilla.org/show_bug.cgi?id=894026
esid: pending
---*/

var chars = ['o', 'O'];

for (var i = 0; i < 8; i++)
{
  if (i === 8)
  {
    chars.forEach(function(v)
    {
      assert.throws(SyntaxError, function() {
        eval('0' + v + i);
      }, "syntax error evaluating 0" + v + i);
    });
    continue;
  }

  for (var j = 0; j < 8; j++)
  {
    if (j === 8)
    {
      chars.forEach(function(v)
      {
        assert.throws(SyntaxError, function() {
          eval('0' + v + i + j);
        }, "syntax error evaluating 0" + v + i + j);
      });
      continue;
    }

    for (var k = 0; k < 8; k++)
    {
      if (k === 8)
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
        assert.sameValue(eval('0' + v + i + j + k), i * 64 + j * 8 + k);
      });
    }
  }
}

// Off-by-one check: '/' immediately precedes '0'.
assert.sameValue(0o110/2, 36);
assert.sameValue(0O644/2, 210);

function strict()
{
  "use strict";
  return 0o755;
}
assert.sameValue(strict(), 7 * 64 + 5 * 8 + 5);

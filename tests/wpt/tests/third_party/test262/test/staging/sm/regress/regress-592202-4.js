/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
function p() { }

function test()
{
  var counter = 0;

  function f(x) {
      try
      { 
        throw 42;
      }
      catch(e)
      { 
        assert.sameValue(counter, 0);
        p(function(){x;});
        counter = 1;
      }
  }

  f(2);
  assert.sameValue(counter, 1);
}

test();


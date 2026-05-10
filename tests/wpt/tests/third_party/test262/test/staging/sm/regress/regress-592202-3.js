/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
test();

function test()
{
  var counter = 0;
  function f(x,y) {
      try
      { 
        throw 42;
      }
      catch(e2)
      { 
        foo(function(){ return x; }| "9.2" && 5 || counter && e);
        ++counter;
      }
  }

  f(2, 1);
}

function foo(bar) { return ""+bar; }


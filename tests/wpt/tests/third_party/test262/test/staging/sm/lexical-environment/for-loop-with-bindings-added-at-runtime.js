/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't assert when freshening the scope chain for a for-loop whose head contains a lexical declaration, where the loop body might add more bindings at runtime
info: bugzilla.mozilla.org/show_bug.cgi?id=1149797
esid: pending
---*/

for (let x = 0; x < 9; ++x)
  eval("var y");

{
  for (let x = 0; x < 9; ++x)
    eval("var y");
}

function f1()
{
  for (let x = 0; x < 9; ++x)
    eval("var y");
}
f1();

function f2()
{
  {
    for (let x = 0; x < 9; ++x)
      eval("var y");
  }
}
f2();

for (let x = 0; x < 9; ++x)
{
  // deliberately inside a block statement
  eval("var y");
}

{
  for (let x = 0; x < 9; ++x)
  {
    // deliberately inside a block statement
    eval("var y");
  }
}

function g1()
{
  for (let x = 0; x < 9; ++x)
  {
    // deliberately inside a block statement
    eval("var y");
  }
}
g1();

function g2()
{
  {
    for (let x = 0; x < 9; ++x)
    {
      // deliberately inside a block statement
      eval("var y");
    }
  }
}
g2();

for (let x = 0; x < 9; ++x) {
  (function() {
      eval("var y");
  })();
}

{
  for (let x = 0; x < 9; ++x)
  {
    // deliberately inside a block statement
    (function() {
        eval("var y");
    })();
  }
}

function h1()
{
  for (let x = 0; x < 9; ++x)
  {
    // deliberately inside a block statement
    (function() {
        eval("var y");
    })();
  }
}
h1();

function h2()
{
  {
    for (let x = 0; x < 9; ++x)
    {
      // deliberately inside a block statement
      (function() { eval("var y"); })();
    }
  }
}
h2();

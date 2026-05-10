/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var v="global";
function f(a) {
  // This eval could extend f's call object. However, the call object has
  // not yet been marked as a delegate at this point, so no scope chain
  // purge takes place when it is extended.
  eval(a);
  {
    let b=3;
    // This eval causes the cloned block object to be added to the
    // scope chain. The block needs a unique shape: its parent call
    // could acquire bindings for anything without affecting the global
    // object's shape, so it's up to the block's shape to mismatch all
    // property cache entries for prior blocks.
    eval("");
    return v;
  };
}

// Call the function once, to cache a reference to the global v from within
// f's lexical block.
assert.sameValue("global", f(""));

// Call the function again, adding a binding to the call, and ensure that
// we do not see any property cache entry created by the previous reference
// that would direct us to the global definition.
assert.sameValue("local", f("var v='local'"));

// Similarly,but with a doubly-nested block; make sure everyone gets marked.
function f2(a) {
  eval(a);
  {
    let b=3;
    {
      let c=4;
      eval("");
      return v;
    };
  };
}

assert.sameValue("global", f2(""));
assert.sameValue("local",  f2("var v='local'"));


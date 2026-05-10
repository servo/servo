/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Do not assert: JSSTRING_IS_FLAT(str_)
info: bugzilla.mozilla.org/show_bug.cgi?id=449666
esid: pending
---*/

var actual = '';
var expect = '';

test();

function test()
{
  var global;


  if (typeof window == 'undefined') {
    global = this;
  }
  else {
    global = window;
  }

  if (!global['g']) {
    global['g'] = {};
  }

  if (!global['g']['l']) {
    global['g']['l'] = {};
    (function() {
      function k(a,b){
        var c=a.split(/\./);
        var d=global;
        for(var e=0;e<c.length-1;e++){
          if(!d[c[e]]){
            d[c[e]]={};
          }
          d=d[c[e]];
        }
        d[c[c.length-1]]=b;
      }

      function T(a){return "hmm"}
      k("g.l.loaded",T);
    })();

  }


  assert.sameValue(expect, actual);
}

/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
/*
 * NB: this test hardcodes the value of MAX_PROPERTY_TREE_HEIGHT.
 */
var i = 0;
function add0to64(o) {
  o.p00 = ++i;o.p01 = ++i;o.p02 = ++i;o.p03 = ++i;o.p04 = ++i;o.p05 = ++i;o.p06 = ++i;o.p07 = ++i;
  o.p10 = ++i;o.p11 = ++i;o.p12 = ++i;o.p13 = ++i;o.p14 = ++i;o.p15 = ++i;o.p16 = ++i;o.p17 = ++i;
  o.p20 = ++i;o.p21 = ++i;o.p22 = ++i;o.p23 = ++i;o.p24 = ++i;o.p25 = ++i;o.p26 = ++i;o.p27 = ++i;
  o.p30 = ++i;o.p31 = ++i;o.p32 = ++i;o.p33 = ++i;o.p34 = ++i;o.p35 = ++i;o.p36 = ++i;o.p37 = ++i;
  o.p40 = ++i;o.p41 = ++i;o.p42 = ++i;o.p43 = ++i;o.p44 = ++i;o.p45 = ++i;o.p46 = ++i;o.p47 = ++i;
  o.p50 = ++i;o.p51 = ++i;o.p52 = ++i;o.p53 = ++i;o.p54 = ++i;o.p55 = ++i;o.p56 = ++i;o.p57 = ++i;
  o.p60 = ++i;o.p61 = ++i;o.p62 = ++i;o.p63 = ++i;o.p64 = ++i;o.p65 = ++i;o.p66 = ++i;o.p67 = ++i;
  o.p70 = ++i;o.p71 = ++i;o.p72 = ++i;o.p73 = ++i;o.p74 = ++i;o.p75 = ++i;o.p76 = ++i;o.p77 = ++i;
  o.p100 = ++i;
  return o;
}
function add65th(o) {
  o.p101 = ++i;
}
var o = add0to64({});
add65th(o);
delete o.p101;
add65th(o);

assert.sameValue(true, true, "don't crash");

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
 * For the sake of cross compatibility with other implementations we
 * implement date parsing heuristics which support single and double
 * digit years. See bug: 1265136
 */

for (let year of Array(100).keys()) {
    for (let month of Array(12).keys()) {
        for (let day of Array(31).keys()) {
            let fullYear = year >= 50 ? year + 1900 : year + 2000;
            let fullDate = new Date(`${month + 1}/${day + 1}/${fullYear}`);

            // mm/dd/yy
            let d1 = new Date(`${month + 1}/${day + 1}/${year}`);
            assert.sameValue(d1.getTime(), fullDate.getTime())

            // yy/mm/dd
            let d2 = new Date(`${year}/${month + 1}/${day + 1}`);
            if (year > 31) {
                assert.sameValue(d2.getTime(), fullDate.getTime())
            } else if (year > 12) {
                assert.sameValue(d2.getTime(), new Date(NaN).getTime())
            }
        }
    }
}

assert.sameValue(new Date("99/1/99").getTime(), new Date(NaN).getTime());
assert.sameValue(new Date("13/13/13").getTime(), new Date(NaN).getTime());
assert.sameValue(new Date("0/10/0").getTime(), new Date(NaN).getTime());

// Written months.
for (let year of Array(1000).keys()) {
    let fullDate = new Date(`5/1/${year}`);
    let d1 = new Date(`may 1 ${year}`);
    let d2 = new Date(`1 may ${year}`);
    let d3 = new Date(`1 ${year} may`);

    assert.sameValue(d1.getTime(), fullDate.getTime())
    assert.sameValue(d2.getTime(), fullDate.getTime())
    assert.sameValue(d3.getTime(), fullDate.getTime())

    if (year > 31) {
      let d4 = new Date(`may ${year} 1`);
      let d5 = new Date(`${year} may 1`);
      let d6 = new Date(`${year} 1 may`);

      assert.sameValue(d4.getTime(), fullDate.getTime())
      assert.sameValue(d5.getTime(), fullDate.getTime())
      assert.sameValue(d6.getTime(), fullDate.getTime())
    }
}

assert.sameValue(new Date("may 1999 1999").getTime(), new Date(NaN).getTime());
assert.sameValue(new Date("may 0 0").getTime(), new Date(NaN).getTime());

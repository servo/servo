// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Map/Set/WeakMap/WeakSet constructor should close iterator on error
info: bugzilla.mozilla.org/show_bug.cgi?id=1180306
esid: pending
---*/

function test(ctors, { nextVal=undefined,
                       nextThrowVal=undefined,
                       modifier=undefined,
                       exceptionVal=undefined,
                       exceptionType=undefined,
                       closed=true }) {
    function getIterable() {
        let iterable = {
            closed: false,
            [Symbol.iterator]() {
                let iterator = {
                    first: true,
                    next() {
                        if (this.first) {
                            this.first = false;
                            if (nextThrowVal)
                                throw nextThrowVal;
                            return nextVal;
                        }
                        return { value: undefined, done: true };
                    },
                    return() {
                        iterable.closed = true;
                        return {};
                    }
                };
                if (modifier)
                    modifier(iterator, iterable);

                return iterator;
            }
        };
        return iterable;
    }

    for (let ctor of ctors) {
        let iterable = getIterable();
        if (exceptionVal) {
            let caught = false;
            try {
                new ctor(iterable);
            } catch (e) {
                assert.sameValue(e, exceptionVal);
                caught = true;
            }
            assert.sameValue(caught, true);
        } else if (exceptionType) {
            assert.throws(exceptionType, () => new ctor(iterable));
        } else {
            new ctor(iterable);
        }
        assert.sameValue(iterable.closed, closed);
    }
}

// == Error cases with close ==

// ES 2017 draft 23.1.1.1 Map step 8.d.ii.
// ES 2017 draft 23.3.1.1 WeakMap step 8.d.ii.
test([Map, WeakMap], {
    nextVal: { value: "non object", done: false },
    exceptionType: TypeError,
    closed: true,
});

// ES 2017 draft 23.1.1.1 Map step 8.f.
// ES 2017 draft 23.3.1.1 WeakMap step 8.f.
test([Map, WeakMap], {
    nextVal: { value: { get 0() { throw "0 getter throws"; } }, done: false },
    exceptionVal: "0 getter throws",
    closed: true,
});

// ES 2017 draft 23.1.1.1 Map step 8.h.
// ES 2017 draft 23.3.1.1 WeakMap step 8.h.
test([Map, WeakMap], {
    nextVal: { value: { 0: {}, get 1() { throw "1 getter throws"; } }, done: false },
    exceptionVal: "1 getter throws",
    closed: true,
});

// ES 2017 draft 23.1.1.1 Map step 8.j.
// ES 2017 draft 23.3.1.1 WeakMap step 8.j.
class MyMap extends Map {
    set(k, v) {
        throw "setter throws";
    }
}
class MyWeakMap extends WeakMap {
    set(k, v) {
        throw "setter throws";
    }
}
test([MyMap, MyWeakMap], {
    nextVal: { value: [{}, {}], done: false },
    exceptionVal: "setter throws",
    closed: true,
});

// ES 2017 draft 23.2.1.1 Set step 8.e.
// ES 2017 draft 23.4.1.1 WeakSet step 8.e.
class MySet extends Set {
    add(v) {
        throw "adder throws";
    }
}
class MyWeakSet extends WeakSet {
    add(v) {
        throw "adder throws";
    }
}
test([MySet, MyWeakSet], {
    nextVal: { value: {}, done: false },
    exceptionVal: "adder throws",
    closed: true,
});

// ES 2021 draft 7.4.6 step 5.
// if GetMethod fails, the thrown value should be ignored.
test([MyMap, MyWeakMap], {
    nextVal: { value: [{}, {}], done: false },
    modifier: (iterator, iterable) => {
        Object.defineProperty(iterator, "return", {
            get: function() {
                iterable.closed = true;
                throw "return getter throws";
            }
        });
    },
    exceptionVal: "setter throws",
    closed: true,
});
test([MySet, MyWeakSet], {
  nextVal: { value: [{}, {}], done: false },
  modifier: (iterator, iterable) => {
      Object.defineProperty(iterator, "return", {
          get: function() {
              iterable.closed = true;
              throw "return getter throws";
          }
      });
  },
  exceptionVal: "adder throws",
  closed: true,
});
test([MyMap, MyWeakMap], {
    nextVal: { value: [{}, {}], done: false },
    modifier: (iterator, iterable) => {
        Object.defineProperty(iterator, "return", {
            get: function() {
                iterable.closed = true;
                return "non object";
            }
        });
    },
    exceptionVal: "setter throws",
    closed: true,
});
test([MySet, MyWeakSet], {
  nextVal: { value: [{}, {}], done: false },
  modifier: (iterator, iterable) => {
      Object.defineProperty(iterator, "return", {
          get: function() {
              iterable.closed = true;
              return "non object";
          }
      });
  },
  exceptionVal: "adder throws",
  closed: true,
});
test([MyMap, MyWeakMap], {
    nextVal: { value: [{}, {}], done: false },
    modifier: (iterator, iterable) => {
        Object.defineProperty(iterator, "return", {
            get: function() {
                iterable.closed = true;
                // Non callable.
                return {};
            }
        });
    },
    exceptionVal: "setter throws",
    closed: true,
});
test([MySet, MyWeakSet], {
  nextVal: { value: [{}, {}], done: false },
  modifier: (iterator, iterable) => {
      Object.defineProperty(iterator, "return", {
          get: function() {
              iterable.closed = true;
              // Non callable.
              return {};
          }
      });
  },
  exceptionVal: "adder throws",
  closed: true,
});

// ES 2017 draft 7.4.6 steps 6.
// if return method throws, the thrown value should be ignored.
test([MyMap, MyWeakMap], {
    nextVal: { value: [{}, {}], done: false },
    modifier: (iterator, iterable) => {
        iterator.return = function() {
            iterable.closed = true;
            throw "return throws";
        };
    },
    exceptionVal: "setter throws",
    closed: true,
});
test([MySet, MyWeakSet], {
    nextVal: { value: [{}, {}], done: false },
    modifier: (iterator, iterable) => {
        iterator.return = function() {
            iterable.closed = true;
            throw "return throws";
        };
    },
    exceptionVal: "adder throws",
    closed: true,
});

test([MyMap, MyWeakMap], {
    nextVal: { value: [{}, {}], done: false },
    modifier: (iterator, iterable) => {
        iterator.return = function() {
            iterable.closed = true;
            return "non object";
        };
    },
    exceptionVal: "setter throws",
    closed: true,
});
test([MySet, MyWeakSet], {
    nextVal: { value: [{}, {}], done: false },
    modifier: (iterator, iterable) => {
        iterator.return = function() {
            iterable.closed = true;
            return "non object";
        };
    },
    exceptionVal: "adder throws",
    closed: true,
});

// == Error cases without close ==

// ES 2017 draft 23.1.1.1 Map step 8.a.
// ES 2017 draft 23.3.1.1 WeakMap step 8.a.
// ES 2017 draft 23.2.1.1 Set step 8.a.
// ES 2017 draft 23.4.1.1 WeakSet step 8.a.
test([Map, WeakMap, Set, WeakSet], {
    nextThrowVal: "next throws",
    exceptionVal: "next throws",
    closed: false,
});
test([Map, WeakMap, Set, WeakSet], {
    nextVal: { value: {}, get done() { throw "done getter throws"; } },
    exceptionVal: "done getter throws",
    closed: false,
});

// ES 2017 draft 23.1.1.1 Map step 8.c.
// ES 2017 draft 23.3.1.1 WeakMap step 8.c.
// ES 2017 draft 23.2.1.1 Set step 8.c.
// ES 2017 draft 23.4.1.1 WeakSet step 8.c.
test([Map, WeakMap, Set, WeakSet], {
    nextVal: { get value() { throw "value getter throws"; }, done: false },
    exceptionVal: "value getter throws",
    closed: false,
});

// == Successful cases ==

test([Map, WeakMap], {
    nextVal: { value: [{}, {}], done: false },
    closed: false,
});
test([Set, WeakSet], {
    nextVal: { value: {}, done: false },
    closed: false,
});

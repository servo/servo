// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Destructuring should evaluate lhs reference before rhs in super property
info: bugzilla.mozilla.org/show_bug.cgi?id=1204028
esid: pending
---*/

let logs = [];
function log(x) {
  logs.push(x);
}

let unwrapMap = new Map();
function unwrap(maybeWrapped) {
  if (unwrapMap.has(maybeWrapped))
    return unwrapMap.get(maybeWrapped);
  return maybeWrapped;
}
function ToString(name) {
  if (name == Symbol.iterator)
    return "@@iterator";
  return String(name);
}
function logger(obj, prefix=[]) {
  let wrapped = new Proxy(obj, {
    get(that, name) {
      let names = prefix.concat(ToString(name));
      log("rhs get " + names.join("::"));
      let v = obj[name];
      if (typeof v === "object" || typeof v === "function")
        return logger(v, names);
      return v;
    },
    apply(that, thisArg, args) {
      let names = prefix.slice();
      log("rhs call " + names.join("::"));
      let v = obj.apply(unwrap(thisArg), args);
      if (typeof v === "object" || typeof v === "function") {
        names[names.length - 1] += "()";
        return logger(v, names);
      }
      return v;
    }
  });
  unwrapMap.set(wrapped, obj);
  return wrapped;
}

class C1 {
  constructor() {
    this.clear();
  }
  clear() {
    this.values = {};
  }
}
for (let name of [
  "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m",
  "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
  "0", "1", "length"
]) {
  Object.defineProperty(C1.prototype, name, {
    set: function(value) {
      log("lhs set " + name);
      this.values[name] = value;
    }
  });
}
class C2 extends C1 {
  constructor() {
    super();

    let clear = () => {
      logs = [];
      this.clear();
    };

    // Array.

    clear();
    [
      super.a
    ] = logger(["A"]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "lhs set a",

               "rhs get @@iterator()::return",
             ].join(","));
    assert.sameValue(this.values.a, "A");

    clear();
    [
      super[ (log("lhs before name a"), "a") ]
    ] = logger(["A"]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "lhs before name a",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "lhs set a",

               "rhs get @@iterator()::return",
             ].join(","));
    assert.sameValue(this.values.a, "A");

    // Array rest.

    clear();
    [
      ...super.a
    ] = logger(["A", "B", "C"]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "lhs set a",
             ].join(","));
    assert.sameValue(this.values.a.join(","), "A,B,C");

    clear();
    [
      ...super[ (log("lhs before name a"), "a") ]
    ] = logger(["A", "B", "C"]);;
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "lhs before name a",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "lhs set a",
             ].join(","));
    assert.sameValue(this.values.a.join(","), "A,B,C");

    // Array combined.

    clear();
    [
      super.a,
      super[ (log("lhs before name b"), "b") ],
      ...super[ (log("lhs before name c"), "c") ]
    ] = logger(["A", "B", "C"]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "lhs set a",

               "lhs before name b",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "lhs set b",

               "lhs before name c",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "lhs set c",
             ].join(","));
    assert.sameValue(this.values.a, "A");
    assert.sameValue(this.values.b, "B");
    assert.sameValue(this.values.c.join(","), "C");

    // Object.

    clear();
    ({
      a: super.a
    } = logger({a: "A"}));
    assert.sameValue(logs.join(","),
             [
               "rhs get a",
               "lhs set a",
             ].join(","));
    assert.sameValue(this.values.a, "A");

    clear();
    ({
      a: super[ (log("lhs before name a"), "a") ]
    } = logger({a: "A"}));
    assert.sameValue(logs.join(","),
             [
               "lhs before name a",
               "rhs get a",
               "lhs set a",
             ].join(","));
    assert.sameValue(this.values.a, "A");

    // Object combined.

    clear();
    ({
      a: super.a,
      b: super[ (log("lhs before name b"), "b") ]
    } = logger({a: "A", b: "B"}));
    assert.sameValue(logs.join(","),
             [
               "rhs get a",
               "lhs set a",

               "lhs before name b",
               "rhs get b",
               "lhs set b",
             ].join(","));
    assert.sameValue(this.values.a, "A");
    assert.sameValue(this.values.b, "B");

    // == Nested ==

    // Array -> Array

    clear();
    [
      [
        super[ (log("lhs before name a"), "a") ],
        ...super[ (log("lhs before name b"), "b") ]
      ]
    ] = logger([["A", "B"]]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs get @@iterator()::next()::value::@@iterator",
               "rhs call @@iterator()::next()::value::@@iterator",
               "rhs get @@iterator()::next()::value::@@iterator()::next",

               "lhs before name a",
               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
               "lhs set a",

               "lhs before name b",
               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "lhs set b",

               "rhs get @@iterator()::return",
             ].join(","));
    assert.sameValue(this.values.a, "A");
    assert.sameValue(this.values.b.length, 1);
    assert.sameValue(this.values.b[0], "B");

    // Array rest -> Array

    clear();
    [
      ...[
        super[ (log("lhs before name a"), "a") ],
        ...super[ (log("lhs before name b"), "b") ]
      ]
    ] = logger(["A", "B"]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",

               "lhs before name a",
               "lhs set a",

               "lhs before name b",
               "lhs set b",
             ].join(","));
    assert.sameValue(this.values.a, "A");
    assert.sameValue(this.values.b.join(","), "B");

    // Array -> Object
    clear();
    [
      {
        a: super[ (log("lhs before name a"), "a") ]
      }
    ] = logger([{a: "A"}]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",

               "lhs before name a",
               "rhs get @@iterator()::next()::value::a",
               "lhs set a",

               "rhs get @@iterator()::return",
             ].join(","));
    assert.sameValue(this.values.a, "A");

    // Array rest -> Object
    clear();
    [
      ...{
        0: super[ (log("lhs before name 0"), "0") ],
        1: super[ (log("lhs before name 1"), "1") ],
        length: super[ (log("lhs before name length"), "length") ],
      }
    ] = logger(["A", "B"]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",

               "lhs before name 0",
               "lhs set 0",

               "lhs before name 1",
               "lhs set 1",

               "lhs before name length",
               "lhs set length",
             ].join(","));
    assert.sameValue(this.values["0"], "A");
    assert.sameValue(this.values["1"], "B");
    assert.sameValue(this.values.length, 2);

    // Object -> Array
    clear();
    ({
      a: [
        super[ (log("lhs before name b"), "b") ]
      ]
    } = logger({a: ["B"]}));
    assert.sameValue(logs.join(","),
             [
               "rhs get a",
               "rhs get a::@@iterator",
               "rhs call a::@@iterator",
               "rhs get a::@@iterator()::next",

               "lhs before name b",
               "rhs call a::@@iterator()::next",
               "rhs get a::@@iterator()::next()::done",
               "rhs get a::@@iterator()::next()::value",
               "lhs set b",

               "rhs get a::@@iterator()::return",
             ].join(","));
    assert.sameValue(this.values.b, "B");

    // Object -> Object
    clear();
    ({
      a: {
        b: super[ (log("lhs before name b"), "b") ]
      }
    } = logger({a: {b: "B"}}));
    assert.sameValue(logs.join(","),
             [
               "rhs get a",
               "lhs before name b",
               "rhs get a::b",
               "lhs set b",
             ].join(","));
    assert.sameValue(this.values.b, "B");

    // All combined

    clear();
    [
      super[ (log("lhs before name a"), "a") ],
      [
        super[ (log("lhs before name b"), "b") ],
        {
          c: super[ (log("lhs before name c"), "c") ],
          d: {
            e: super[ (log("lhs before name e"), "e") ],
            f: [
              super[ (log("lhs before name g"), "g") ]
            ]
          }
        }
      ],
      {
        h: super[ (log("lhs before name h"), "h") ],
        i: [
          super[ (log("lhs before name j"), "j") ],
          {
            k: [
              super[ (log("lhs before name l"), "l") ]
            ]
          }
        ]
      },
      ...[
        super[ (log("lhs before name m"), "m") ],
        [
          super[ (log("lhs before name n"), "n") ],
          {
            o: super[ (log("lhs before name o"), "o") ],
            p: {
              q: super[ (log("lhs before name q"), "q") ],
              r: [
                super[ (log("lhs before name s"), "s") ]
              ]
            }
          }
        ],
        ...{
          0: super[ (log("lhs before name t"), "t") ],
          1: [
            super[ (log("lhs before name u"), "u") ],
            {
              v: super[ (log("lhs before name v"), "v") ],
              w: {
                x: super[ (log("lhs before name x"), "x") ],
                y: [
                  super[ (log("lhs before name z"), "z") ]
                ]
              }
            }
          ],
          length: super[ (log("lhs before name length"), "length") ],
        }
      ]
    ] = logger(["A",
                ["B", {c: "C", d: {e: "E", f: ["G"]}}],
                {h: "H", i: ["J", {k: ["L"]}]},
                "M",
                ["N", {o: "O", p: {q: "Q", r: ["S"]}}],
                "T", ["U", {v: "V", w: {x: "X", y: ["Z"]}}]]);
    assert.sameValue(logs.join(","),
             [
               "rhs get @@iterator",
               "rhs call @@iterator",
               "rhs get @@iterator()::next",

               "lhs before name a",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "lhs set a",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs get @@iterator()::next()::value::@@iterator",
               "rhs call @@iterator()::next()::value::@@iterator",
               "rhs get @@iterator()::next()::value::@@iterator()::next",

               "lhs before name b",
               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
               "lhs set b",

               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value",

               "lhs before name c",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::c",
               "lhs set c",

               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d",

               "lhs before name e",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::e",
               "lhs set e",

               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator",
               "rhs call @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator()::next",

               "lhs before name g",
               "rhs call @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator()::next()::value",
               "lhs set g",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator()::return",
               "rhs get @@iterator()::next()::value::@@iterator()::return",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",

               "lhs before name h",
               "rhs get @@iterator()::next()::value::h",
               "lhs set h",

               "rhs get @@iterator()::next()::value::i",
               "rhs get @@iterator()::next()::value::i::@@iterator",
               "rhs call @@iterator()::next()::value::i::@@iterator",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next",

               "lhs before name j",
               "rhs call @@iterator()::next()::value::i::@@iterator()::next",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::value",
               "lhs set j",

               "rhs call @@iterator()::next()::value::i::@@iterator()::next",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::value",

               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::value::k",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::value::k::@@iterator",
               "rhs call @@iterator()::next()::value::i::@@iterator()::next()::value::k::@@iterator",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::value::k::@@iterator()::next",

               "lhs before name l",
               "rhs call @@iterator()::next()::value::i::@@iterator()::next()::value::k::@@iterator()::next",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::value::k::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::value::k::@@iterator()::next()::value",
               "lhs set l",
               "rhs get @@iterator()::next()::value::i::@@iterator()::next()::value::k::@@iterator()::return",
               "rhs get @@iterator()::next()::value::i::@@iterator()::return",

               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",
               "rhs get @@iterator()::next()::value",
               "rhs call @@iterator()::next",
               "rhs get @@iterator()::next()::done",

               "lhs before name m",
               "lhs set m",

               "rhs get @@iterator()::next()::value::@@iterator",
               "rhs call @@iterator()::next()::value::@@iterator",
               "rhs get @@iterator()::next()::value::@@iterator()::next",

               "lhs before name n",
               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
               "lhs set n",

               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value",

               "lhs before name o",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::o",
               "lhs set o",

               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p",

               "lhs before name q",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::q",
               "lhs set q",

               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator",
               "rhs call @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::next",

               "lhs before name s",
               "rhs call @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::next()::value",
               "lhs set s",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::return",
               "rhs get @@iterator()::next()::value::@@iterator()::return",

               "lhs before name t",
               "lhs set t",

               "rhs get @@iterator()::next()::value::@@iterator",
               "rhs call @@iterator()::next()::value::@@iterator",
               "rhs get @@iterator()::next()::value::@@iterator()::next",

               "lhs before name u",
               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
               "lhs set u",

               "rhs call @@iterator()::next()::value::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value",

               "lhs before name v",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::v",
               "lhs set v",

               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w",

               "lhs before name x",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::x",
               "lhs set x",

               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator",
               "rhs call @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::next",

               "lhs before name z",
               "rhs call @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::next",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::next()::done",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::next()::value",
               "lhs set z",
               "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::return",
               "rhs get @@iterator()::next()::value::@@iterator()::return",

               "lhs before name length",
               "lhs set length",
             ].join(","));
    assert.sameValue(this.values.a, "A");
    assert.sameValue(this.values.b, "B");
    assert.sameValue(this.values.c, "C");
    assert.sameValue(this.values.e, "E");
    assert.sameValue(this.values.g, "G");
    assert.sameValue(this.values.h, "H");
    assert.sameValue(this.values.j, "J");
    assert.sameValue(this.values.l, "L");
    assert.sameValue(this.values.m, "M");
    assert.sameValue(this.values.n, "N");
    assert.sameValue(this.values.o, "O");
    assert.sameValue(this.values.q, "Q");
    assert.sameValue(this.values.s, "S");
    assert.sameValue(this.values.t, "T");
    assert.sameValue(this.values.u, "U");
    assert.sameValue(this.values.v, "V");
    assert.sameValue(this.values.x, "X");
    assert.sameValue(this.values.z, "Z");
    assert.sameValue(this.values.length, 2);
  }
}

new C2();

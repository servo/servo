// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Destructuring should evaluate lhs reference before rhs
info: bugzilla.mozilla.org/show_bug.cgi?id=1204028
esid: pending
---*/

let storage = {
  clear() {
    this.values = {};
  }
};
storage.clear();
let obj = new Proxy(storage, {
  set(that, name, value) {
    log("lhs set " + name);
    storage.values[name] = value;
    return true;
  }
});

let logs = [];
function log(x) {
  logs.push(x);
}

function clear() {
  logs = [];
  storage.clear();
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

// Array.

clear();
[
  ( log("lhs before obj a"), obj ).a
] = logger(["A"]);
assert.sameValue(logs.join(","),
         [
           "rhs get @@iterator",
           "rhs call @@iterator",
           "rhs get @@iterator()::next",

           "lhs before obj a",
           "rhs call @@iterator()::next",
           "rhs get @@iterator()::next()::done",
           "rhs get @@iterator()::next()::value",
           "lhs set a",

           "rhs get @@iterator()::return",
         ].join(","));
assert.sameValue(storage.values.a, "A");

clear();
[
  ( log("lhs before obj a"), obj )[ (log("lhs before name a"), "a") ]
] = logger(["A"]);
assert.sameValue(logs.join(","),
         [
           "rhs get @@iterator",
           "rhs call @@iterator",
           "rhs get @@iterator()::next",

           "lhs before obj a",
           "lhs before name a",
           "rhs call @@iterator()::next",
           "rhs get @@iterator()::next()::done",
           "rhs get @@iterator()::next()::value",
           "lhs set a",

           "rhs get @@iterator()::return",
         ].join(","));
assert.sameValue(storage.values.a, "A");

// Array rest.

clear();
[
  ...( log("lhs before obj a"), obj ).a
] = logger(["A", "B", "C"]);
assert.sameValue(logs.join(","),
         [
           "rhs get @@iterator",
           "rhs call @@iterator",
           "rhs get @@iterator()::next",

           "lhs before obj a",
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
assert.sameValue(storage.values.a.join(","), "A,B,C");

clear();
[
  ...( log("lhs before obj a"), obj )[ (log("lhs before name a"), "a") ]
] = logger(["A", "B", "C"]);;
assert.sameValue(logs.join(","),
         [
           "rhs get @@iterator",
           "rhs call @@iterator",
           "rhs get @@iterator()::next",

           "lhs before obj a",
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
assert.sameValue(storage.values.a.join(","), "A,B,C");

// Array combined.

clear();
[
  ( log("lhs before obj a"), obj ).a,
  ( log("lhs before obj b"), obj )[ (log("lhs before name b"), "b") ],
  ...( log("lhs before obj c"), obj )[ (log("lhs before name c"), "c") ]
] = logger(["A", "B", "C"]);
assert.sameValue(logs.join(","),
         [
           "rhs get @@iterator",
           "rhs call @@iterator",
           "rhs get @@iterator()::next",

           "lhs before obj a",
           "rhs call @@iterator()::next",
           "rhs get @@iterator()::next()::done",
           "rhs get @@iterator()::next()::value",
           "lhs set a",

           "lhs before obj b",
           "lhs before name b",
           "rhs call @@iterator()::next",
           "rhs get @@iterator()::next()::done",
           "rhs get @@iterator()::next()::value",
           "lhs set b",

           "lhs before obj c",
           "lhs before name c",
           "rhs call @@iterator()::next",
           "rhs get @@iterator()::next()::done",
           "rhs get @@iterator()::next()::value",
           "rhs call @@iterator()::next",
           "rhs get @@iterator()::next()::done",
           "lhs set c",
         ].join(","));
assert.sameValue(storage.values.a, "A");
assert.sameValue(storage.values.b, "B");
assert.sameValue(storage.values.c.join(","), "C");

// Object.

clear();
({
  a: ( log("lhs before obj a"), obj ).a
} = logger({a: "A"}));
assert.sameValue(logs.join(","),
         [
           "lhs before obj a",
           "rhs get a",
           "lhs set a",
         ].join(","));
assert.sameValue(storage.values.a, "A");

clear();
({
  a: ( log("lhs before obj a"), obj )[ (log("lhs before name a"), "a") ]
} = logger({a: "A"}));
assert.sameValue(logs.join(","),
         [
           "lhs before obj a",
           "lhs before name a",
           "rhs get a",
           "lhs set a",
         ].join(","));
assert.sameValue(storage.values.a, "A");

// Object combined.

clear();
({
  a: ( log("lhs before obj a"), obj ).a,
  b: ( log("lhs before obj b"), obj )[ (log("lhs before name b"), "b") ]
} = logger({a: "A", b: "B"}));
assert.sameValue(logs.join(","),
         [
           "lhs before obj a",
           "rhs get a",
           "lhs set a",

           "lhs before obj b",
           "lhs before name b",
           "rhs get b",
           "lhs set b",
         ].join(","));
assert.sameValue(storage.values.a, "A");
assert.sameValue(storage.values.b, "B");

// == Nested ==

// Array -> Array

clear();
[
  [
    ( log("lhs before obj a"), obj )[ (log("lhs before name a"), "a") ],
    ...( log("lhs before obj b"), obj )[ (log("lhs before name b"), "b") ]
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

           "lhs before obj a",
           "lhs before name a",
           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
           "lhs set a",

           "lhs before obj b",
           "lhs before name b",
           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "lhs set b",

           "rhs get @@iterator()::return",
         ].join(","));
assert.sameValue(storage.values.a, "A");
assert.sameValue(storage.values.b.length, 1);
assert.sameValue(storage.values.b[0], "B");

// Array rest -> Array

clear();
[
  ...[
    ( log("lhs before obj a"), obj )[ (log("lhs before name a"), "a") ],
    ...( log("lhs before obj b"), obj )[ (log("lhs before name b"), "b") ]
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

           "lhs before obj a",
           "lhs before name a",
           "lhs set a",

           "lhs before obj b",
           "lhs before name b",
           "lhs set b",
         ].join(","));
assert.sameValue(storage.values.a, "A");
assert.sameValue(storage.values.b.join(","), "B");

// Array -> Object
clear();
[
  {
    a: ( log("lhs before obj a"), obj )[ (log("lhs before name a"), "a") ]
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

           "lhs before obj a",
           "lhs before name a",
           "rhs get @@iterator()::next()::value::a",
           "lhs set a",

           "rhs get @@iterator()::return",
         ].join(","));
assert.sameValue(storage.values.a, "A");

// Array rest -> Object
clear();
[
  ...{
    0: ( log("lhs before obj 0"), obj )[ (log("lhs before name 0"), "0") ],
    1: ( log("lhs before obj 1"), obj )[ (log("lhs before name 1"), "1") ],
    length: ( log("lhs before obj length"), obj )[ (log("lhs before name length"), "length") ],
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

           "lhs before obj 0",
           "lhs before name 0",
           "lhs set 0",

           "lhs before obj 1",
           "lhs before name 1",
           "lhs set 1",

           "lhs before obj length",
           "lhs before name length",
           "lhs set length",
         ].join(","));
assert.sameValue(storage.values["0"], "A");
assert.sameValue(storage.values["1"], "B");
assert.sameValue(storage.values.length, 2);

// Object -> Array
clear();
({
  a: [
    ( log("lhs before obj b"), obj )[ (log("lhs before name b"), "b") ]
  ]
} = logger({a: ["B"]}));
assert.sameValue(logs.join(","),
         [
           "rhs get a",
           "rhs get a::@@iterator",
           "rhs call a::@@iterator",
           "rhs get a::@@iterator()::next",

           "lhs before obj b",
           "lhs before name b",
           "rhs call a::@@iterator()::next",
           "rhs get a::@@iterator()::next()::done",
           "rhs get a::@@iterator()::next()::value",
           "lhs set b",

           "rhs get a::@@iterator()::return",
         ].join(","));
assert.sameValue(storage.values.b, "B");

// Object -> Object
clear();
({
  a: {
    b: ( log("lhs before obj b"), obj )[ (log("lhs before name b"), "b") ]
  }
} = logger({a: {b: "B"}}));
assert.sameValue(logs.join(","),
         [
           "rhs get a",
           "lhs before obj b",
           "lhs before name b",
           "rhs get a::b",
           "lhs set b",
         ].join(","));
assert.sameValue(storage.values.b, "B");

// All combined

clear();
[
  ( log("lhs before obj a"), obj )[ (log("lhs before name a"), "a") ],
  [
    ( log("lhs before obj b"), obj )[ (log("lhs before name b"), "b") ],
    {
      c: ( log("lhs before obj c"), obj )[ (log("lhs before name c"), "c") ],
      d: {
        e: ( log("lhs before obj e"), obj )[ (log("lhs before name e"), "e") ],
        f: [
          ( log("lhs before obj g"), obj )[ (log("lhs before name g"), "g") ]
        ]
      }
    }
  ],
  {
    h: ( log("lhs before obj h"), obj )[ (log("lhs before name h"), "h") ],
    i: [
      ( log("lhs before obj j"), obj )[ (log("lhs before name j"), "j") ],
      {
        k: [
          ( log("lhs before obj l"), obj )[ (log("lhs before name l"), "l") ]
        ]
      }
    ]
  },
  ...[
    ( log("lhs before obj m"), obj )[ (log("lhs before name m"), "m") ],
    [
      ( log("lhs before obj n"), obj )[ (log("lhs before name n"), "n") ],
      {
        o: ( log("lhs before obj o"), obj )[ (log("lhs before name o"), "o") ],
        p: {
          q: ( log("lhs before obj q"), obj )[ (log("lhs before name q"), "q") ],
          r: [
            ( log("lhs before obj s"), obj )[ (log("lhs before name s"), "s") ]
          ]
        }
      }
    ],
    ...{
      0: ( log("lhs before obj t"), obj )[ (log("lhs before name t"), "t") ],
      1: [
        ( log("lhs before obj u"), obj )[ (log("lhs before name u"), "u") ],
        {
          v: ( log("lhs before obj v"), obj )[ (log("lhs before name v"), "v") ],
          w: {
            x: ( log("lhs before obj x"), obj )[ (log("lhs before name x"), "x") ],
            y: [
              ( log("lhs before obj z"), obj )[ (log("lhs before name z"), "z") ]
            ]
          }
        }
      ],
      length: ( log("lhs before obj length"), obj )[ (log("lhs before name length"), "length") ],
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

           "lhs before obj a",
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

           "lhs before obj b",
           "lhs before name b",
           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
           "lhs set b",

           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value",

           "lhs before obj c",
           "lhs before name c",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::c",
           "lhs set c",

           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d",

           "lhs before obj e",
           "lhs before name e",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::e",
           "lhs set e",

           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator",
           "rhs call @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::d::f::@@iterator()::next",

           "lhs before obj g",
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

           "lhs before obj h",
           "lhs before name h",
           "rhs get @@iterator()::next()::value::h",
           "lhs set h",

           "rhs get @@iterator()::next()::value::i",
           "rhs get @@iterator()::next()::value::i::@@iterator",
           "rhs call @@iterator()::next()::value::i::@@iterator",
           "rhs get @@iterator()::next()::value::i::@@iterator()::next",

           "lhs before obj j",
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

           "lhs before obj l",
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

           "lhs before obj m",
           "lhs before name m",
           "lhs set m",

           "rhs get @@iterator()::next()::value::@@iterator",
           "rhs call @@iterator()::next()::value::@@iterator",
           "rhs get @@iterator()::next()::value::@@iterator()::next",

           "lhs before obj n",
           "lhs before name n",
           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
           "lhs set n",

           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value",

           "lhs before obj o",
           "lhs before name o",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::o",
           "lhs set o",

           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p",

           "lhs before obj q",
           "lhs before name q",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::q",
           "lhs set q",

           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator",
           "rhs call @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::next",

           "lhs before obj s",
           "lhs before name s",
           "rhs call @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::next()::value",
           "lhs set s",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::p::r::@@iterator()::return",
           "rhs get @@iterator()::next()::value::@@iterator()::return",

           "lhs before obj t",
           "lhs before name t",
           "lhs set t",

           "rhs get @@iterator()::next()::value::@@iterator",
           "rhs call @@iterator()::next()::value::@@iterator",
           "rhs get @@iterator()::next()::value::@@iterator()::next",

           "lhs before obj u",
           "lhs before name u",
           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value",
           "lhs set u",

           "rhs call @@iterator()::next()::value::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value",

           "lhs before obj v",
           "lhs before name v",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::v",
           "lhs set v",

           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w",

           "lhs before obj x",
           "lhs before name x",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::x",
           "lhs set x",

           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator",
           "rhs call @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::next",

           "lhs before obj z",
           "lhs before name z",
           "rhs call @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::next",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::next()::done",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::next()::value",
           "lhs set z",
           "rhs get @@iterator()::next()::value::@@iterator()::next()::value::w::y::@@iterator()::return",
           "rhs get @@iterator()::next()::value::@@iterator()::return",

           "lhs before obj length",
           "lhs before name length",
           "lhs set length",
         ].join(","));
assert.sameValue(storage.values.a, "A");
assert.sameValue(storage.values.b, "B");
assert.sameValue(storage.values.c, "C");
assert.sameValue(storage.values.e, "E");
assert.sameValue(storage.values.g, "G");
assert.sameValue(storage.values.h, "H");
assert.sameValue(storage.values.j, "J");
assert.sameValue(storage.values.l, "L");
assert.sameValue(storage.values.m, "M");
assert.sameValue(storage.values.n, "N");
assert.sameValue(storage.values.o, "O");
assert.sameValue(storage.values.q, "Q");
assert.sameValue(storage.values.s, "S");
assert.sameValue(storage.values.t, "T");
assert.sameValue(storage.values.u, "U");
assert.sameValue(storage.values.v, "V");
assert.sameValue(storage.values.x, "X");
assert.sameValue(storage.values.z, "Z");
assert.sameValue(storage.values.length, 2);

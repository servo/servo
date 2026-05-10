// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Object operations are performed in the right order, as observed by proxies.

let log = [];
function LoggingProxyHandlerWrapper(name, handler={}) {
    return new Proxy(handler, {
        get(t, id) {
            let method = handler[id];
            return function (...args) {
                log.push([name + "." + id, ...args.filter(v => typeof v !== "object")]);
                if (method === undefined)
                    return Reflect[id].apply(null, args);
                return method.apply(this, args);
            };
        }
    });
}

function LoggingProxy(name, target) {
    return new Proxy(target, new LoggingProxyHandlerWrapper(name));
}

let proto = {x: 44};
let proto_proxy = new LoggingProxy("proto", proto);
let unscopables = {x: true};
let unscopables_proxy = new LoggingProxy("unscopables", {x: true});
let env = Object.create(proto_proxy, {
    [Symbol.unscopables]: { value: unscopables_proxy }
});
let env_proxy = new LoggingProxy("env", env);

let x = 11;
function f() {
    with (env_proxy)
        return x;
}

assert.sameValue(f(), 11);

assert.deepEqual(log, [
    ["env.has", "x"],
    ["proto.has", "x"],
    ["env.get", Symbol.unscopables],
    ["unscopables.get", "x"]
]);


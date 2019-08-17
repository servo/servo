/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

let _Symbol$iterator;

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

export function pcombine(params) {
  return new PCombine(params);
}

function merge(a, b) {
  for (const key of Object.keys(a)) {
    if (b.hasOwnProperty(key)) {
      throw new Error('Duplicate key: ' + key);
    }
  }

  return { ...a,
    ...b
  };
}

function* cartesian(iters) {
  if (iters.length === 0) {
    return;
  }

  if (iters.length === 1) {
    yield* iters[0];
    return;
  }

  const [as, ...rest] = iters;

  for (const a of as) {
    for (const b of cartesian(rest)) {
      yield merge(a, b);
    }
  }
}

_Symbol$iterator = Symbol.iterator;

class PCombine {
  constructor(params) {
    _defineProperty(this, "params", void 0);

    this.params = params;
  }

  [_Symbol$iterator]() {
    return cartesian(this.params);
  }

}
//# sourceMappingURL=combine.js.map
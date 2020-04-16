/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

let _Symbol$iterator, _Symbol$iterator2, _Symbol$iterator3, _Symbol$iterator4;

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { paramsEquals } from './params_utils.js';
import { assert } from './util/util.js';
export function poptions(name, values) {
  return new POptions(name, values);
}
export function pbool(name) {
  return new POptions(name, [false, true]);
}
export function pexclude(params, exclude) {
  return new PExclude(params, exclude);
}
export function pfilter(cases, pred) {
  return new PFilter(cases, pred);
}
export function pcombine(...params) {
  return new PCombine(params);
}
_Symbol$iterator = Symbol.iterator;

class POptions {
  constructor(name, values) {
    _defineProperty(this, "name", void 0);

    _defineProperty(this, "values", void 0);

    this.name = name;
    this.values = values;
  }

  *[_Symbol$iterator]() {
    for (const value of this.values) {
      yield {
        [this.name]: value
      };
    }
  }

}

_Symbol$iterator2 = Symbol.iterator;

class PExclude {
  constructor(cases, exclude) {
    _defineProperty(this, "cases", void 0);

    _defineProperty(this, "exclude", void 0);

    this.cases = cases;
    this.exclude = Array.from(exclude);
  }

  *[_Symbol$iterator2]() {
    for (const p of this.cases) {
      if (this.exclude.every(e => !paramsEquals(p, e))) {
        yield p;
      }
    }
  }

}

_Symbol$iterator3 = Symbol.iterator;

class PFilter {
  constructor(cases, pred) {
    _defineProperty(this, "cases", void 0);

    _defineProperty(this, "pred", void 0);

    this.cases = cases;
    this.pred = pred;
  }

  *[_Symbol$iterator3]() {
    for (const p of this.cases) {
      if (this.pred(p)) {
        yield p;
      }
    }
  }

}

_Symbol$iterator4 = Symbol.iterator;

class PCombine {
  constructor(params) {
    _defineProperty(this, "params", void 0);

    this.params = params;
  }

  [_Symbol$iterator4]() {
    return PCombine.cartesian(this.params);
  }

  static merge(a, b) {
    for (const key of Object.keys(a)) {
      assert(!b.hasOwnProperty(key), 'Duplicate key: ' + key);
    }

    return { ...a,
      ...b
    };
  }

  static *cartesian(iters) {
    if (iters.length === 0) {
      return;
    }

    if (iters.length === 1) {
      yield* iters[0];
      return;
    }

    const [as, ...rest] = iters;

    for (const a of as) {
      for (const b of PCombine.cartesian(rest)) {
        yield PCombine.merge(a, b);
      }
    }
  }

}
//# sourceMappingURL=params.js.map
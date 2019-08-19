/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

let _Symbol$iterator;

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { paramsEquals } from './index.js';
export function pexclude(params, exclude) {
  return new PExclude(params, exclude);
}
_Symbol$iterator = Symbol.iterator;

class PExclude {
  constructor(cases, exclude) {
    _defineProperty(this, "cases", void 0);

    _defineProperty(this, "exclude", void 0);

    this.cases = cases;
    this.exclude = Array.from(exclude);
  }

  *[_Symbol$iterator]() {
    for (const p of this.cases) {
      if (this.exclude.every(e => !paramsEquals(p, e))) {
        yield p;
      }
    }
  }

}
//# sourceMappingURL=exclude.js.map
/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

let _Symbol$iterator;

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

export function pfilter(cases, pred) {
  return new PFilter(cases, pred);
}
_Symbol$iterator = Symbol.iterator;

class PFilter {
  constructor(cases, pred) {
    _defineProperty(this, "cases", void 0);

    _defineProperty(this, "pred", void 0);

    this.cases = cases;
    this.pred = pred;
  }

  *[_Symbol$iterator]() {
    for (const p of this.cases) {
      if (this.pred(p)) {
        yield p;
      }
    }
  }

}
//# sourceMappingURL=filter.js.map
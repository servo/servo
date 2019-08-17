/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

let _Symbol$iterator;

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

export function poptions(name, values) {
  return new POptions(name, values);
}
export function pbool(name) {
  return new POptions(name, [false, true]);
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
//# sourceMappingURL=options.js.map
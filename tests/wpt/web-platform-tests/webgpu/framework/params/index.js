/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export * from './combine.js';
export * from './filter.js';
export * from './options.js';
export * from './exclude.js';
export function paramsEquals(x, y) {
  if (x === y) {
    return true;
  }

  if (x === null || y === null) {
    return false;
  }

  for (const xk of Object.keys(x)) {
    if (!y.hasOwnProperty(xk)) {
      return false;
    }

    if (x[xk] !== y[xk]) {
      return false;
    }
  }

  for (const yk of Object.keys(y)) {
    if (!x.hasOwnProperty(yk)) {
      return false;
    }
  }

  return true;
}
export function paramsSupersets(sup, sub) {
  if (sub === null) {
    return true;
  }

  if (sup === null) {
    // && sub !== undefined
    return false;
  }

  for (const k of Object.keys(sub)) {
    if (!sup.hasOwnProperty(k) || sup[k] !== sub[k]) {
      return false;
    }
  }

  return true;
}
//# sourceMappingURL=index.js.map
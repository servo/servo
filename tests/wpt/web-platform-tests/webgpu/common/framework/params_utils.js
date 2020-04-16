/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { objectEquals } from './util/util.js';
export function extractPublicParams(params) {
  const publicParams = {};

  for (const k of Object.keys(params)) {
    if (!k.startsWith('_')) {
      publicParams[k] = params[k];
    }
  }

  return publicParams;
}
export function stringifyPublicParams(p) {
  if (p === null || paramsEquals(p, {})) {
    return '';
  }

  return JSON.stringify(extractPublicParams(p));
}
export function paramsEquals(x, y) {
  if (x === y) {
    return true;
  }

  if (x === null) {
    x = {};
  }

  if (y === null) {
    y = {};
  }

  for (const xk of Object.keys(x)) {
    if (x[xk] !== undefined && !y.hasOwnProperty(xk)) {
      return false;
    }

    if (!objectEquals(x[xk], y[xk])) {
      return false;
    }
  }

  for (const yk of Object.keys(y)) {
    if (y[yk] !== undefined && !x.hasOwnProperty(yk)) {
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
    sup = {};
  }

  for (const k of Object.keys(sub)) {
    if (!sup.hasOwnProperty(k) || sup[k] !== sub[k]) {
      return false;
    }
  }

  return true;
}
//# sourceMappingURL=params_utils.js.map
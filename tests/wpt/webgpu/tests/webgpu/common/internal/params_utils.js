/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../util/util.js';

import { comparePublicParamsPaths, Ordering } from './query/compare.js';
import { kWildcard, kParamSeparator, kParamKVSeparator } from './query/separators.js';















export function paramKeyIsPublic(key) {
  return !key.startsWith('_');
}

export function extractPublicParams(params) {
  const publicParams = {};
  for (const k of Object.keys(params)) {
    if (paramKeyIsPublic(k)) {
      publicParams[k] = params[k];
    }
  }
  return publicParams;
}

/** Used to escape reserved characters in URIs */
const kPercent = '%';

export const badParamValueChars = new RegExp(
  '[' + kParamKVSeparator + kParamSeparator + kWildcard + kPercent + ']'
);

export function publicParamsEquals(x, y) {
  return comparePublicParamsPaths(x, y) === Ordering.Equal;
}





/**
 * Flatten a union of interfaces into a single interface encoding the same type.
 *
 * Flattens a union in such a way that:
 * `{ a: number, b?: undefined } | { b: string, a?: undefined }`
 * (which is the value type of `[{ a: 1 }, { b: 1 }]`)
 * becomes `{ a: number | undefined, b: string | undefined }`.
 *
 * And also works for `{ a: number } | { b: string }` which maps to the same.
 */











function typeAssert() {}
{






















  {
    typeAssert();
    typeAssert();
    typeAssert();
    typeAssert();
    typeAssert();

    typeAssert();

    typeAssert();
    typeAssert();
    typeAssert();
    typeAssert();
    typeAssert();

    // Unexpected test results - hopefully okay to ignore these
    typeAssert();
    typeAssert();
  }
}






/** Merges two objects into one `{ ...a, ...b }` and return it with a flattened type. */
export function mergeParams(a, b) {
  return { ...a, ...b };
}

/**
 * Merges two objects into one `{ ...a, ...b }` and asserts they had no overlapping keys.
 * This is slower than {@link mergeParams}.
 */
export function mergeParamsChecked(a, b) {
  const merged = mergeParams(a, b);
  assert(
    Object.keys(merged).length === Object.keys(a).length + Object.keys(b).length,
    () => `Duplicate key between ${JSON.stringify(a)} and ${JSON.stringify(b)}`
  );
  return merged;
}
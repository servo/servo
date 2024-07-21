/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { Float16Array } from '../../external/petamoriken/float16/float16.js';import { SkipTestCase } from '../framework/fixture.js';import { globalTestConfig } from '../framework/test_config.js';

import { keysOf } from './data_tables.js';
import { timeout } from './timeout.js';

/**
 * Error with arbitrary `extra` data attached, for debugging.
 * The extra data is omitted if not running the test in debug mode (`?debug=1`).
 */
export class ErrorWithExtra extends Error {


  /**
   * `extra` function is only called if in debug mode.
   * If an `ErrorWithExtra` is passed, its message is used and its extras are passed through.
   */


  constructor(baseOrMessage, newExtra) {
    const message = typeof baseOrMessage === 'string' ? baseOrMessage : baseOrMessage.message;
    super(message);

    const oldExtras = baseOrMessage instanceof ErrorWithExtra ? baseOrMessage.extra : {};
    this.extra = globalTestConfig.enableDebugLogs ?
    { ...oldExtras, ...newExtra() } :
    { omitted: 'pass ?debug=1' };
  }
}

/**
 * Asserts `condition` is true. Otherwise, throws an `Error` with the provided message.
 */
export function assert(condition, msg) {
  if (!condition) {
    throw new Error(msg && (typeof msg === 'string' ? msg : msg()));
  }
}

/** If the argument is an Error, throw it. Otherwise, pass it back. */
export function assertOK(value) {
  if (value instanceof Error) {
    throw value;
  }
  return value;
}

/** Options for assertReject, shouldReject, and friends. */


/**
 * Resolves if the provided promise rejects; rejects if it does not.
 */
export async function assertReject(
expectedName,
p,
{ allowMissingStack = false, message } = {})
{
  try {
    await p;
    unreachable(message);
  } catch (ex) {
    // Asserted as expected
    if (!allowMissingStack) {
      const m = message ? ` (${message})` : '';
      assert(
        ex instanceof Error && typeof ex.stack === 'string',
        'threw as expected, but missing stack' + m
      );
    }
  }
}

/**
 * Assert this code is unreachable. Unconditionally throws an `Error`.
 */
export function unreachable(msg) {
  throw new Error(msg);
}

/**
 * Throw a `SkipTestCase` exception, which skips the test case.
 */
export function skipTestCase(msg) {
  throw new SkipTestCase(msg);
}

/**
 * The `performance` interface.
 * It is available in all browsers, but it is not in scope by default in Node.
 */

const perf = typeof performance !== 'undefined' ? performance : require('perf_hooks').performance;

/**
 * Calls the appropriate `performance.now()` depending on whether running in a browser or Node.
 */
export function now() {
  return perf.now();
}

/**
 * Returns a promise which resolves after the specified time.
 */
export function resolveOnTimeout(ms) {
  return new Promise((resolve) => {
    timeout(() => {
      resolve();
    }, ms);
  });
}

export class PromiseTimeoutError extends Error {}

/**
 * Returns a promise which rejects after the specified time.
 */
export function rejectOnTimeout(ms, msg) {
  return new Promise((_resolve, reject) => {
    timeout(() => {
      reject(new PromiseTimeoutError(msg));
    }, ms);
  });
}

/**
 * Takes a promise `p`, and returns a new one which rejects if `p` takes too long,
 * and otherwise passes the result through.
 */
export function raceWithRejectOnTimeout(p, ms, msg) {
  if (globalTestConfig.noRaceWithRejectOnTimeout) {
    return p;
  }
  // Setup a promise that will reject after `ms` milliseconds. We cancel this timeout when
  // `p` is finalized, so the JavaScript VM doesn't hang around waiting for the timer to
  // complete, once the test runner has finished executing the tests.
  const timeoutPromise = new Promise((_resolve, reject) => {
    const handle = timeout(() => {
      reject(new PromiseTimeoutError(msg));
    }, ms);
    p = p.finally(() => clearTimeout(handle));
  });
  return Promise.race([p, timeoutPromise]);
}

/**
 * Takes a promise `p` and returns a new one which rejects if `p` resolves or rejects,
 * and otherwise resolves after the specified time.
 */
export function assertNotSettledWithinTime(
p,
ms,
msg)
{
  // Rejects regardless of whether p resolves or rejects.
  const rejectWhenSettled = p.then(() => Promise.reject(new Error(msg)));
  // Resolves after `ms` milliseconds.
  const timeoutPromise = new Promise((resolve) => {
    const handle = timeout(() => {
      resolve(undefined);
    }, ms);
    void p.finally(() => clearTimeout(handle));
  });
  return Promise.race([rejectWhenSettled, timeoutPromise]);
}

/**
 * Returns a `Promise.reject()`, but also registers a dummy `.catch()` handler so it doesn't count
 * as an uncaught promise rejection in the runtime.
 */
export function rejectWithoutUncaught(err) {
  const p = Promise.reject(err);
  // Suppress uncaught promise rejection.
  p.catch(() => {});
  return p;
}

/**
 * Returns true if v is a plain JavaScript object.
 */
export function isPlainObject(v) {
  return !!v && Object.getPrototypeOf(v).constructor === Object.prototype.constructor;
}

/**
 * Makes a copy of a JS `object`, with the keys reordered into sorted order.
 */
export function sortObjectByKey(v) {
  const sortedObject = {};
  for (const k of Object.keys(v).sort()) {
    sortedObject[k] = v[k];
  }
  return sortedObject;
}

/**
 * Determines whether two JS values are equal, recursing into objects and arrays.
 * NaN is treated specially, such that `objectEquals(NaN, NaN)`. +/-0.0 are treated as equal
 * by default, but can be opted to be distinguished.
 * @param x the first JS values that get compared
 * @param y the second JS values that get compared
 * @param distinguishSignedZero if set to true, treat 0.0 and -0.0 as unequal. Default to false.
 */
export function objectEquals(
x,
y,
distinguishSignedZero = false)
{
  if (typeof x !== 'object' || typeof y !== 'object') {
    if (typeof x === 'number' && typeof y === 'number' && Number.isNaN(x) && Number.isNaN(y)) {
      return true;
    }
    // Object.is(0.0, -0.0) is false while (0.0 === -0.0) is true. Other than +/-0.0 and NaN cases,
    // Object.is works in the same way as ===.
    return distinguishSignedZero ? Object.is(x, y) : x === y;
  }
  if (x === null || y === null) return x === y;
  if (x.constructor !== y.constructor) return false;
  if (x instanceof Function) return x === y;
  if (x instanceof RegExp) return x === y;
  if (x === y || x.valueOf() === y.valueOf()) return true;
  if (Array.isArray(x) && Array.isArray(y) && x.length !== y.length) return false;
  if (x instanceof Date) return false;
  if (!(x instanceof Object)) return false;
  if (!(y instanceof Object)) return false;

  const x1 = x;
  const y1 = y;
  const p = Object.keys(x);
  return Object.keys(y).every((i) => p.indexOf(i) !== -1) && p.every((i) => objectEquals(x1[i], y1[i]));
}

/**
 * Generates a range of values `fn(0)..fn(n-1)`.
 */
export function range(n, fn) {
  return [...new Array(n)].map((_, i) => fn(i));
}

/**
 * Generates a range of values `fn(0)..fn(n-1)`.
 */
export function* iterRange(n, fn) {
  for (let i = 0; i < n; ++i) {
    yield fn(i);
  }
}

/** Creates a (reusable) iterable object that maps `f` over `xs`, lazily. */
export function mapLazy(xs, f) {
  return {
    *[Symbol.iterator]() {
      for (const x of xs) {
        yield f(x);
      }
    }
  };
}

const ReorderOrders = {
  forward: true,
  backward: true,
  shiftByHalf: true
};

export const kReorderOrderKeys = keysOf(ReorderOrders);

/**
 * Creates a new array from the given array with the first half
 * swapped with the last half.
 */
export function shiftByHalf(arr) {
  const len = arr.length;
  const half = len / 2 | 0;
  const firstHalf = arr.splice(0, half);
  return [...arr, ...firstHalf];
}

/**
 * Creates a reordered array from the input array based on the Order
 */
export function reorder(order, arr) {
  switch (order) {
    case 'forward':
      return arr.slice();
    case 'backward':
      return arr.slice().reverse();
    case 'shiftByHalf':{
        // should this be pseudo random?
        return shiftByHalf(arr);
      }
  }
}

const TypedArrayBufferViewInstances = [
new Uint8Array(),
new Uint8ClampedArray(),
new Uint16Array(),
new Uint32Array(),
new Int8Array(),
new Int16Array(),
new Int32Array(),
new Float16Array(),
new Float32Array(),
new Float64Array(),
new BigInt64Array(),
new BigUint64Array()];
























export const kTypedArrayBufferViews =

{
  ...(() => {

    const result = {};
    for (const v of TypedArrayBufferViewInstances) {
      result[v.constructor.name] = v.constructor;
    }
    return result;
  })()
};
export const kTypedArrayBufferViewKeys = keysOf(kTypedArrayBufferViews);
export const kTypedArrayBufferViewConstructors = Object.values(kTypedArrayBufferViews);




















/**
 * Creates a case parameter for a typedarray.
 *
 * You can't put typedarrays in case parameters directly so instead of
 *
 * ```
 * u.combine('data', [
 *   new Uint8Array([1, 2, 3]),
 *   new Float32Array([4, 5, 6]),
 * ])
 * ```
 *
 * You can use
 *
 * ```
 * u.combine('data', [
 *   typedArrayParam('Uint8Array' [1, 2, 3]),
 *   typedArrayParam('Float32Array' [4, 5, 6]),
 * ])
 * ```
 *
 * and then convert the params to typedarrays eg.
 *
 * ```
 *  .fn(t => {
 *    const data = t.params.data.map(v => typedArrayFromParam(v));
 *  })
 * ```
 */
export function typedArrayParam(
type,
data)
{
  return { type, data };
}

export function createTypedArray(
type,
data)
{
  return new kTypedArrayBufferViews[type](data);
}

/**
 * Converts a TypedArrayParam to a typedarray. See typedArrayParam
 */
export function typedArrayFromParam(
param)
{
  const { type, data } = param;
  return createTypedArray(type, data);
}

function subarrayAsU8(
buf,
{ start = 0, length })
{
  if (buf instanceof ArrayBuffer) {
    return new Uint8Array(buf, start, length);
  } else if (buf instanceof Uint8Array || buf instanceof Uint8ClampedArray) {
    // Don't wrap in new views if we don't need to.
    if (start === 0 && (length === undefined || length === buf.byteLength)) {
      return buf;
    }
  }
  const byteOffset = buf.byteOffset + start * buf.BYTES_PER_ELEMENT;
  const byteLength =
  length !== undefined ?
  length * buf.BYTES_PER_ELEMENT :
  buf.byteLength - (byteOffset - buf.byteOffset);
  return new Uint8Array(buf.buffer, byteOffset, byteLength);
}

/**
 * Copy a range of bytes from one ArrayBuffer or TypedArray to another.
 *
 * `start`/`length` are in elements (or in bytes, if ArrayBuffer).
 */
export function memcpy(
src,
dst)
{
  subarrayAsU8(dst.dst, dst).set(subarrayAsU8(src.src, src));
}

/**
 * Used to create a value that is specified by multiplying some runtime value
 * by a constant and then adding a constant to it.
 */





/**
 * Filters out SpecValues that are the same.
 */
export function filterUniqueValueTestVariants(valueTestVariants) {
  return new Map(
    valueTestVariants.map((v) => [`m:${v.mult},a:${v.add}`, v])
  ).values();
}

/**
 * Used to create a value that is specified by multiplied some runtime value
 * by a constant and then adding a constant to it. This happens often in test
 * with limits that can only be known at runtime and yet we need a way to
 * add parameters to a test and those parameters must be constants.
 */
export function makeValueTestVariant(base, variant) {
  return base * variant.mult + variant.add;
}
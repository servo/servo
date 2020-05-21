/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { timeout } from './timeout.js';
export function assert(condition, msg) {
  if (!condition) {
    throw new Error(msg && (typeof msg === 'string' ? msg : msg()));
  }
}
export async function assertReject(p, msg) {
  try {
    await p;
    unreachable(msg);
  } catch (ex) {// Assertion OK
  }
}
export function unreachable(msg) {
  throw new Error(msg);
} // performance.now() is available in all browsers, but not in scope by default in Node.

const perf = typeof performance !== 'undefined' ? performance : require('perf_hooks').performance;
export function now() {
  return perf.now();
}
export function resolveOnTimeout(ms) {
  return new Promise(resolve => {
    timeout(() => {
      resolve();
    }, ms);
  });
}
export class PromiseTimeoutError extends Error {}
export function rejectOnTimeout(ms, msg) {
  return new Promise((_resolve, reject) => {
    timeout(() => {
      reject(new PromiseTimeoutError(msg));
    }, ms);
  });
}
export function raceWithRejectOnTimeout(p, ms, msg) {
  return Promise.race([p, rejectOnTimeout(ms, msg)]);
}
export function objectEquals(x, y) {
  if (typeof x !== 'object' || typeof y !== 'object') return x === y;
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
  return Object.keys(y).every(i => p.indexOf(i) !== -1) && p.every(i => objectEquals(x1[i], y1[i]));
}
export function range(n, fn) {
  return [...new Array(n)].map((_, i) => fn(i));
}
//# sourceMappingURL=util.js.map
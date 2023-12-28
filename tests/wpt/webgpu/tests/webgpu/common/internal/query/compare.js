/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, objectEquals } from '../../util/util.js';import { paramKeyIsPublic } from '../params_utils.js';



export let Ordering = /*#__PURE__*/function (Ordering) {Ordering[Ordering["Unordered"] = 0] = "Unordered";Ordering[Ordering["StrictSuperset"] = 1] = "StrictSuperset";Ordering[Ordering["Equal"] = 2] = "Equal";Ordering[Ordering["StrictSubset"] = 3] = "StrictSubset";return Ordering;}({});






/**
 * Compares two queries for their ordering (which is used to build the tree).
 *
 * See src/unittests/query_compare.spec.ts for examples.
 */
export function compareQueries(a, b) {
  if (a.suite !== b.suite) {
    return Ordering.Unordered;
  }

  const filePathOrdering = comparePaths(a.filePathParts, b.filePathParts);
  if (filePathOrdering !== Ordering.Equal || a.isMultiFile || b.isMultiFile) {
    return compareOneLevel(filePathOrdering, a.isMultiFile, b.isMultiFile);
  }
  assert('testPathParts' in a && 'testPathParts' in b);

  const testPathOrdering = comparePaths(a.testPathParts, b.testPathParts);
  if (testPathOrdering !== Ordering.Equal || a.isMultiTest || b.isMultiTest) {
    return compareOneLevel(testPathOrdering, a.isMultiTest, b.isMultiTest);
  }
  assert('params' in a && 'params' in b);

  const paramsPathOrdering = comparePublicParamsPaths(a.params, b.params);
  if (paramsPathOrdering !== Ordering.Equal || a.isMultiCase || b.isMultiCase) {
    return compareOneLevel(paramsPathOrdering, a.isMultiCase, b.isMultiCase);
  }
  return Ordering.Equal;
}

/**
 * Compares a single level of a query.
 *
 * "IsBig" means the query is big relative to the level, e.g. for test-level:
 *   - Anything >= `suite:a,*` is big
 *   - Anything <= `suite:a:*` is small
 */
function compareOneLevel(ordering, aIsBig, bIsBig) {
  assert(ordering !== Ordering.Equal || aIsBig || bIsBig);
  if (ordering === Ordering.Unordered) return Ordering.Unordered;
  if (aIsBig && bIsBig) return ordering;
  if (!aIsBig && !bIsBig) return Ordering.Unordered; // Equal case is already handled
  // Exactly one of (a, b) is big.
  if (aIsBig && ordering !== Ordering.StrictSubset) return Ordering.StrictSuperset;
  if (bIsBig && ordering !== Ordering.StrictSuperset) return Ordering.StrictSubset;
  return Ordering.Unordered;
}

function comparePaths(a, b) {
  const shorter = Math.min(a.length, b.length);

  for (let i = 0; i < shorter; ++i) {
    if (a[i] !== b[i]) {
      return Ordering.Unordered;
    }
  }
  if (a.length === b.length) {
    return Ordering.Equal;
  } else if (a.length < b.length) {
    return Ordering.StrictSuperset;
  } else {
    return Ordering.StrictSubset;
  }
}

export function comparePublicParamsPaths(a, b) {
  const aKeys = Object.keys(a).filter((k) => paramKeyIsPublic(k));
  const commonKeys = new Set(aKeys.filter((k) => k in b));

  for (const k of commonKeys) {
    // Treat +/-0.0 as different query by distinguishing them in objectEquals
    if (!objectEquals(a[k], b[k], true)) {
      return Ordering.Unordered;
    }
  }
  const bKeys = Object.keys(b).filter((k) => paramKeyIsPublic(k));
  const aRemainingKeys = aKeys.length - commonKeys.size;
  const bRemainingKeys = bKeys.length - commonKeys.size;
  if (aRemainingKeys === 0 && bRemainingKeys === 0) return Ordering.Equal;
  if (aRemainingKeys === 0) return Ordering.StrictSuperset;
  if (bRemainingKeys === 0) return Ordering.StrictSubset;
  return Ordering.Unordered;
}
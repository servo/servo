/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { mergeParams } from '../internal/params_utils.js';
import { stringifyPublicParams } from '../internal/query/stringify_params.js';
import { assert, mapLazy } from '../util/util.js';

// ================================================================
// "Public" ParamsBuilder API / Documentation
// ================================================================

/**
 * Provides doc comments for the methods of CaseParamsBuilder and SubcaseParamsBuilder.
 * (Also enforces rough interface match between them.)
 */

/**
 * Base class for `CaseParamsBuilder` and `SubcaseParamsBuilder`.
 */
export class ParamsBuilderBase {
  constructor(cases) {
    this.cases = cases;
  }

  /**
   * Hidden from test files. Use `builderIterateCasesWithSubcases` to access this.
   */
}

/**
 * Calls the (normally hidden) `iterateCasesWithSubcases()` method.
 */
export function builderIterateCasesWithSubcases(builder) {
  return builder.iterateCasesWithSubcases();
}

/**
 * Builder for combinatorial test **case** parameters.
 *
 * CaseParamsBuilder is immutable. Each method call returns a new, immutable object,
 * modifying the list of cases according to the method called.
 *
 * This means, for example, that the `unit` passed into `TestBuilder.params()` can be reused.
 */
export class CaseParamsBuilder extends ParamsBuilderBase {
  *iterateCasesWithSubcases() {
    for (const a of this.cases()) {
      yield [a, undefined];
    }
  }

  [Symbol.iterator]() {
    return this.cases();
  }

  /** @inheritDoc */
  expandWithParams(expander) {
    const newGenerator = expanderGenerator(this.cases, expander);
    return new CaseParamsBuilder(() => newGenerator({}));
  }

  /** @inheritDoc */
  expand(key, expander) {
    return this.expandWithParams(function* (p) {
      for (const value of expander(p)) {
        yield { [key]: value };
      }
    });
  }

  /** @inheritDoc */
  combineWithParams(newParams) {
    assertNotGenerator(newParams);
    const seenValues = new Set();
    for (const params of newParams) {
      const paramsStr = stringifyPublicParams(params);
      assert(!seenValues.has(paramsStr), `Duplicate entry in combine[WithParams]: ${paramsStr}`);
      seenValues.add(paramsStr);
    }

    return this.expandWithParams(() => newParams);
  }

  /** @inheritDoc */
  combine(key, values) {
    assertNotGenerator(values);
    const mapped = mapLazy(values, v => ({ [key]: v }));
    return this.combineWithParams(mapped);
  }

  /** @inheritDoc */
  filter(pred) {
    const newGenerator = filterGenerator(this.cases, pred);
    return new CaseParamsBuilder(() => newGenerator({}));
  }

  /** @inheritDoc */
  unless(pred) {
    return this.filter(x => !pred(x));
  }

  /**
   * "Finalize" the list of cases and begin defining subcases.
   * Returns a new SubcaseParamsBuilder. Methods called on SubcaseParamsBuilder
   * generate new subcases instead of new cases.
   */
  beginSubcases() {
    return new SubcaseParamsBuilder(
      () => this.cases(),
      function* () {
        yield {};
      }
    );
  }
}

/**
 * The unit CaseParamsBuilder, representing a single case with no params: `[ {} ]`.
 *
 * `punit` is passed to every `.params()`/`.paramsSubcasesOnly()` call, so `kUnitCaseParamsBuilder`
 * is only explicitly needed if constructing a ParamsBuilder outside of a test builder.
 */
export const kUnitCaseParamsBuilder = new CaseParamsBuilder(function* () {
  yield {};
});

/**
 * Builder for combinatorial test _subcase_ parameters.
 *
 * SubcaseParamsBuilder is immutable. Each method call returns a new, immutable object,
 * modifying the list of subcases according to the method called.
 */
export class SubcaseParamsBuilder extends ParamsBuilderBase {
  constructor(cases, generator) {
    super(cases);
    this.subcases = generator;
  }

  *iterateCasesWithSubcases() {
    for (const caseP of this.cases()) {
      const subcases = Array.from(this.subcases(caseP));
      if (subcases.length) {
        yield [caseP, subcases];
      }
    }
  }

  /** @inheritDoc */
  expandWithParams(expander) {
    return new SubcaseParamsBuilder(this.cases, expanderGenerator(this.subcases, expander));
  }

  /** @inheritDoc */
  expand(key, expander) {
    return this.expandWithParams(function* (p) {
      for (const value of expander(p)) {
        // TypeScript doesn't know here that NewPKey is always a single literal string type.
        yield { [key]: value };
      }
    });
  }

  /** @inheritDoc */
  combineWithParams(newParams) {
    assertNotGenerator(newParams);
    return this.expandWithParams(() => newParams);
  }

  /** @inheritDoc */
  combine(key, values) {
    assertNotGenerator(values);
    return this.expand(key, () => values);
  }

  /** @inheritDoc */
  filter(pred) {
    return new SubcaseParamsBuilder(this.cases, filterGenerator(this.subcases, pred));
  }

  /** @inheritDoc */
  unless(pred) {
    return this.filter(x => !pred(x));
  }
}

function expanderGenerator(baseGenerator, expander) {
  return function* (base) {
    for (const a of baseGenerator(base)) {
      for (const b of expander(mergeParams(base, a))) {
        yield mergeParams(a, b);
      }
    }
  };
}

function filterGenerator(baseGenerator, pred) {
  return function* (base) {
    for (const a of baseGenerator(base)) {
      if (pred(mergeParams(base, a))) {
        yield a;
      }
    }
  };
}

/** Assert an object is not a Generator (a thing returned from a generator function). */
function assertNotGenerator(x) {
  if ('constructor' in x) {
    assert(
      x.constructor !== (function* () {})().constructor,
      'Argument must not be a generator, as generators are not reusable'
    );
  }
}

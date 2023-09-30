/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { dataCache } from '../../../../common/framework/data_cache.js';
import { unreachable } from '../../../../common/util/util.js';
import { deserializeComparator, serializeComparator } from '../../../util/compare.js';
import {
  Scalar,
  Vector,
  serializeValue,
  deserializeValue,
  Matrix,
} from '../../../util/conversion.js';
import {
  deserializeFPInterval,
  FPInterval,
  serializeFPInterval,
} from '../../../util/floating_point.js';
import { flatten2DArray, unflatten2DArray } from '../../../util/math.js';

import { isComparator } from './expression.js';

/**
 * SerializedExpectationValue holds the serialized form of an Expectation when
 * the Expectation is a Value
 * This form can be safely encoded to JSON.
 */

/** serializeExpectation() converts an Expectation to a SerializedExpectation */
export function serializeExpectation(e) {
  if (e instanceof Scalar || e instanceof Vector || e instanceof Matrix) {
    return { kind: 'value', value: serializeValue(e) };
  }
  if (e instanceof FPInterval) {
    return { kind: 'interval', value: serializeFPInterval(e) };
  }
  if (e instanceof Array) {
    if (e[0] instanceof Array) {
      e = e;
      const cols = e.length;
      const rows = e[0].length;
      return {
        kind: '2d-interval-array',
        cols,
        rows,
        value: flatten2DArray(e).map(serializeFPInterval),
      };
    } else {
      e = e;
      return { kind: 'intervals', value: e.map(serializeFPInterval) };
    }
  }
  if (isComparator(e)) {
    return { kind: 'comparator', value: serializeComparator(e) };
  }
  unreachable(`cannot serialize Expectation ${e}`);
}

/** deserializeExpectation() converts a SerializedExpectation to a Expectation */
export function deserializeExpectation(data) {
  switch (data.kind) {
    case 'value':
      return deserializeValue(data.value);
    case 'interval':
      return deserializeFPInterval(data.value);
    case 'intervals':
      return data.value.map(deserializeFPInterval);
    case '2d-interval-array':
      return unflatten2DArray(data.value.map(deserializeFPInterval), data.cols, data.rows);
    case 'comparator':
      return deserializeComparator(data.value);
  }
}

/**
 * SerializedCase holds the serialized form of a Case.
 * This form can be safely encoded to JSON.
 */

/** serializeCase() converts an Case to a SerializedCase */
export function serializeCase(c) {
  return {
    input: c.input instanceof Array ? c.input.map(v => serializeValue(v)) : serializeValue(c.input),
    expected: serializeExpectation(c.expected),
  };
}

/** serializeCase() converts an SerializedCase to a Case */
export function deserializeCase(data) {
  return {
    input:
      data.input instanceof Array
        ? data.input.map(v => deserializeValue(v))
        : deserializeValue(data.input),
    expected: deserializeExpectation(data.expected),
  };
}

/** CaseListBuilder is a function that builds a CaseList */

/**
 * CaseCache is a cache of CaseList.
 * CaseCache implements the Cacheable interface, so the cases can be pre-built
 * and stored in the data cache, reducing computation costs at CTS runtime.
 */
export class CaseCache {
  /**
   * Constructor
   * @param name the name of the cache. This must be globally unique.
   * @param builders a Record of case-list name to case-list builder.
   */
  constructor(name, builders) {
    this.path = `webgpu/shader/execution/case-cache/${name}.json`;
    this.builders = builders;
  }

  /** get() returns the list of cases with the given name */
  async get(name) {
    const data = await dataCache.fetch(this);
    return data[name];
  }

  /**
   * build() implements the Cacheable.build interface.
   * @returns the data.
   */
  build() {
    const built = {};
    for (const name in this.builders) {
      const cases = this.builders[name]();
      built[name] = cases;
    }
    return Promise.resolve(built);
  }

  /**
   * serialize() implements the Cacheable.serialize interface.
   * @returns the serialized data.
   */
  serialize(data) {
    const serialized = {};
    for (const name in data) {
      serialized[name] = data[name].map(c => serializeCase(c));
    }
    return JSON.stringify(serialized);
  }

  /**
   * deserialize() implements the Cacheable.deserialize interface.
   * @returns the deserialize data.
   */
  deserialize(serialized) {
    const data = JSON.parse(serialized);
    const casesByName = {};
    for (const name in data) {
      const cases = data[name].map(caseData => deserializeCase(caseData));
      casesByName[name] = cases;
    }
    return casesByName;
  }
}

export function makeCaseCache(name, builders) {
  return new CaseCache(name, builders);
}

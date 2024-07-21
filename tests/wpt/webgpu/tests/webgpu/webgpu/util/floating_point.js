/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, unreachable } from '../../common/util/util.js';import { Float16Array } from '../../external/petamoriken/float16/float16.js';




import { anyOf } from './compare.js';
import { kValue } from './constants.js';
import {
  abstractFloat,
  f16,
  f32,
  isFloatType,


  toMatrix,
  toVector,
  u32 } from
'./conversion.js';
import {
  calculatePermutations,
  cartesianProduct,
  correctlyRoundedF16,
  correctlyRoundedF32,
  correctlyRoundedF64,
  every2DArray,
  flatten2DArray,

  flushSubnormalNumberF16,
  flushSubnormalNumberF32,
  flushSubnormalNumberF64,
  isFiniteF16,
  isFiniteF32,
  isSubnormalNumberF16,
  isSubnormalNumberF32,
  isSubnormalNumberF64,
  map2DArray,
  oneULPF16,
  oneULPF32,
  quantizeToF16,
  quantizeToF32,
  scalarF16Range,
  scalarF32Range,
  scalarF64Range,
  sparseMatrixF16Range,
  sparseMatrixF32Range,
  sparseMatrixF64Range,
  sparseScalarF16Range,
  sparseScalarF32Range,
  sparseScalarF64Range,
  sparseVectorF16Range,
  sparseVectorF32Range,
  sparseVectorF64Range,
  unflatten2DArray,
  vectorF16Range,
  vectorF32Range,
  vectorF64Range } from
'./math.js';

/** Indicate the kind of WGSL floating point numbers being operated on */var


SerializedFPIntervalKind = /*#__PURE__*/function (SerializedFPIntervalKind) {SerializedFPIntervalKind[SerializedFPIntervalKind["Abstract"] = 0] = "Abstract";SerializedFPIntervalKind[SerializedFPIntervalKind["F32"] = 1] = "F32";SerializedFPIntervalKind[SerializedFPIntervalKind["F16"] = 2] = "F16";return SerializedFPIntervalKind;}(SerializedFPIntervalKind || {});





/** serializeFPKind() serializes a FPKind to a BinaryStream */
export function serializeFPKind(s, value) {
  switch (value) {
    case 'abstract':
      s.writeU8(SerializedFPIntervalKind.Abstract);
      break;
    case 'f16':
      s.writeU8(SerializedFPIntervalKind.F16);
      break;
    case 'f32':
      s.writeU8(SerializedFPIntervalKind.F32);
      break;
  }
}

/** deserializeFPKind() deserializes a FPKind from a BinaryStream */
export function deserializeFPKind(s) {
  const kind = s.readU8();
  switch (kind) {
    case SerializedFPIntervalKind.Abstract:
      return 'abstract';
    case SerializedFPIntervalKind.F16:
      return 'f16';
    case SerializedFPIntervalKind.F32:
      return 'f32';
    default:
      unreachable(`invalid deserialized FPKind: ${kind}`);
  }
}
// Containers

/**
 * Representation of endpoints for an interval as an array with either one or
 * two elements. Single element indicates that the interval is a single point.
 * For two elements, the first is the lower edges of the interval and the
 * second is the upper edge, i.e. e[0] <= e[1], where e is an IntervalEndpoints
 */


/** Represents a closed interval of floating point numbers */
export class FPInterval {




  /**
   * Constructor
   *
   * `FPTraits.toInterval` is the preferred way to create FPIntervals
   *
   * @param kind the floating point number type this is an interval for
   * @param endpoints beginning and end of the interval
   */
  constructor(kind, ...endpoints) {
    this.kind = kind;

    const begin = endpoints[0];
    const end = endpoints.length === 2 ? endpoints[1] : endpoints[0];
    assert(!Number.isNaN(begin) && !Number.isNaN(end), `endpoints need to be non-NaN`);
    assert(
      begin <= end,
      `endpoints[0] (${begin}) must be less than or equal to endpoints[1]  (${end})`
    );

    this.begin = begin;
    this.end = end;
  }

  /** @returns the floating point traits for this interval */
  traits() {
    return FP[this.kind];
  }

  /** @returns begin and end if non-point interval, otherwise just begin */
  endpoints() {
    return this.isPoint() ? [this.begin] : [this.begin, this.end];
  }

  /** @returns if a point or interval is completely contained by this interval */
  contains(n) {
    if (Number.isNaN(n)) {
      // Being the 'any' interval indicates that accuracy is not defined for this
      // test, so the test is just checking that this input doesn't cause the
      // implementation to misbehave, so NaN is accepted.
      return this.begin === Number.NEGATIVE_INFINITY && this.end === Number.POSITIVE_INFINITY;
    }

    if (n instanceof FPInterval) {
      return this.begin <= n.begin && this.end >= n.end;
    }
    return this.begin <= n && this.end >= n;
  }

  /** @returns if any values in the interval may be flushed to zero, this
   *           includes any subnormals and zero itself.
   */
  containsZeroOrSubnormals() {
    return !(
    this.end < this.traits().constants().negative.subnormal.min ||
    this.begin > this.traits().constants().positive.subnormal.max);

  }

  /** @returns if this interval contains a single point */
  isPoint() {
    return this.begin === this.end;
  }

  /** @returns if this interval only contains finite values */
  isFinite() {
    return this.traits().isFinite(this.begin) && this.traits().isFinite(this.end);
  }

  /** @returns a string representation for logging purposes */
  toString() {
    return `{ '${this.kind}', [${this.endpoints().map(this.traits().scalarBuilder)}] }`;
  }
}

/** serializeFPInterval() serializes a FPInterval to a BinaryStream */
export function serializeFPInterval(s, i) {
  serializeFPKind(s, i.kind);
  const traits = FP[i.kind];
  s.writeCond(i !== traits.constants().unboundedInterval, {
    if_true: () => {
      // Bounded
      switch (i.kind) {
        case 'abstract':
          s.writeF64(i.begin);
          s.writeF64(i.end);
          break;
        case 'f32':
          s.writeF32(i.begin);
          s.writeF32(i.end);
          break;
        case 'f16':
          s.writeF16(i.begin);
          s.writeF16(i.end);
          break;
        default:
          unreachable(`Unable to serialize FPInterval ${i}`);
          break;
      }
    },
    if_false: () => {

      // Unbounded
    } });
}

/** deserializeFPInterval() deserializes a FPInterval from a BinaryStream */
export function deserializeFPInterval(s) {
  const kind = deserializeFPKind(s);
  const traits = FP[kind];
  return s.readCond({
    if_true: () => {
      // Bounded
      switch (kind) {
        case 'abstract':
          return new FPInterval(traits.kind, s.readF64(), s.readF64());
        case 'f32':
          return new FPInterval(traits.kind, s.readF32(), s.readF32());
        case 'f16':
          return new FPInterval(traits.kind, s.readF16(), s.readF16());
      }
      unreachable(`Unable to deserialize FPInterval with kind ${kind}`);
    },
    if_false: () => {
      // Unbounded
      return traits.constants().unboundedInterval;
    }
  });
}

/**
 * Representation of a vec2/3/4 of floating point intervals as an array of
 * FPIntervals.
 */





/** Shorthand for an Array of Arrays that contains a column-major matrix */


/**
 * Representation of a matCxR of floating point intervals as an array of arrays
 * of FPIntervals. This maps onto the WGSL concept of matrix. Internally
 */












































// Utilities

/** @returns input with an appended 0, if inputs contains non-zero subnormals */
// When f16 traits is defined, this can be replaced with something like
// `FP.f16..addFlushIfNeeded`
function addFlushedIfNeededF16(values) {
  return values.some((v) => v !== 0 && isSubnormalNumberF16(v)) ? values.concat(0) : values;
}

// Operations

/**
 * A function that converts a point to an acceptance interval.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/** Operation used to implement a ScalarToInterval */






























/**
 * A function that converts a pair of points to an acceptance interval.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/** Domain for a ScalarPairToInterval implementation */






/** Operation used to implement a ScalarPairToInterval */




























/**
 * A function that converts a triplet of points to an acceptance interval.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/** Operation used to implement a ScalarTripleToInterval */






// Currently ScalarToVector is not integrated with the rest of the floating point
// framework, because the only builtins that use it are actually
// u32 -> [f32, f32, f32, f32] functions, so the whole rounding and interval
// process doesn't get applied to the inputs.
// They do use the framework internally by invoking divisionInterval on segments
// of the input.
/**
 * A function that converts a point to a vector of acceptance intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/**
 * A function that converts a vector to an acceptance interval.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/** Operation used to implement a VectorToInterval */






/**
 * A function that converts a pair of vectors to an acceptance interval.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/** Operation used to implement a VectorPairToInterval */






/**
 * A function that converts a vector to a vector of acceptance intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/** Operation used to implement a VectorToVector */






/**
 * A function that converts a pair of vectors to a vector of acceptance
 * intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/** Operation used to implement a VectorPairToVector */






/**
 * A function that converts a vector and a scalar to a vector of acceptance
 * intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/**
 * A function that converts a scalar and a vector  to a vector of acceptance
 * intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/**
 * A function that converts a matrix to an acceptance interval.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/** Operation used to implement a MatrixToMatrix */






/**
 * A function that converts a matrix to a matrix of acceptance intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/**
 * A function that converts a pair of matrices to a matrix of acceptance
 * intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/**
 * A function that converts a matrix and a scalar to a matrix of acceptance
 * intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/**
 * A function that converts a scalar and a matrix to a matrix of acceptance
 * intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/**
 * A function that converts a matrix and a vector to a vector of acceptance
 * intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




/**
 * A function that converts a vector and a matrix to a vector of acceptance
 * intervals.
 * This is the public facing API for builtin implementations that is called
 * from tests.
 */




// Traits

/**
 * Typed structure containing all the constants defined for each
 * WGSL floating point kind
 */











































































/** A representation of an FPInterval for a case param */





/** Abstract base class for all floating-point traits */
export class FPTraits {

  constructor(k) {
    this.kind = k;
  }



  // Utilities - Implemented

  /** @returns an interval containing the point or the original interval */
  toInterval(n) {
    if (n instanceof FPInterval) {
      if (n.kind === this.kind) {
        return n;
      }

      // Preserve if the original interval was unbounded or bounded
      if (!n.isFinite()) {
        return this.constants().unboundedInterval;
      }

      return new FPInterval(this.kind, ...n.endpoints());
    }

    if (n instanceof Array) {
      return new FPInterval(this.kind, ...n);
    }

    return new FPInterval(this.kind, n, n);
  }

  /**
   * Makes a param that can be turned into an interval
   */
  toParam(n) {
    return {
      kind: this.kind,
      interval: n
    };
  }

  /**
   * Converts p into an FPInterval if it is an FPIntervalPAram
   */
  fromParam(
  p)
  {
    const param = p;
    if (param.interval && param.kind) {
      assert(param.kind === this.kind);
      return this.toInterval(param.interval);
    }
    return p;
  }

  /**
   * @returns an interval with the tightest endpoints that includes all provided
   *          intervals
   */
  spanIntervals(...intervals) {
    assert(intervals.length > 0, `span of an empty list of FPIntervals is not allowed`);
    assert(
      intervals.every((i) => i.kind === this.kind),
      `span is only defined for intervals with the same kind`
    );
    let begin = Number.POSITIVE_INFINITY;
    let end = Number.NEGATIVE_INFINITY;
    intervals.forEach((i) => {
      begin = Math.min(i.begin, begin);
      end = Math.max(i.end, end);
    });
    return this.toInterval([begin, end]);
  }

  /** Narrow an array of values to FPVector if possible */
  isVector(v) {
    if (v.every((e) => e instanceof FPInterval && e.kind === this.kind)) {
      return v.length === 2 || v.length === 3 || v.length === 4;
    }
    return false;
  }

  /** @returns an FPVector representation of an array of values if possible */
  toVector(v) {
    if (this.isVector(v) && v.every((e) => e.kind === this.kind)) {
      return v;
    }

    const f = v.map((e) => this.toInterval(e));
    // The return of the map above is a readonly FPInterval[], which needs to be narrowed
    // to FPVector, since FPVector is defined as fixed length tuples.
    if (this.isVector(f)) {
      return f;
    }
    unreachable(`Cannot convert [${v}] to FPVector`);
  }

  /**
   * @returns a FPVector where each element is the span for corresponding
   *          elements at the same index in the input vectors
   */
  spanVectors(...vectors) {
    assert(
      vectors.every((e) => this.isVector(e)),
      'Vector span is not defined for vectors of differing floating point kinds'
    );

    const vector_length = vectors[0].length;
    assert(
      vectors.every((e) => e.length === vector_length),
      `Vector span is not defined for vectors of differing lengths`
    );

    const result = new Array(vector_length);

    for (let i = 0; i < vector_length; i++) {
      result[i] = this.spanIntervals(...vectors.map((v) => v[i]));
    }
    return this.toVector(result);
  }

  /** Narrow an array of an array of values to FPMatrix if possible */
  isMatrix(m) {
    if (!m.every((c) => c.every((e) => e instanceof FPInterval && e.kind === this.kind))) {
      return false;
    }
    // At this point m guaranteed to be a ROArrayArray<FPInterval>, but maybe typed as a
    // FPVector[].
    // Coercing the type since FPVector[] is functionally equivalent to
    // ROArrayArray<FPInterval> for .length and .every, but they are type compatible,
    // since tuples are not equivalent to arrays, so TS considers c in .every to
    // be unresolvable below, even though our usage is safe.
    m = m;

    if (m.length > 4 || m.length < 2) {
      return false;
    }

    const num_rows = m[0].length;
    if (num_rows > 4 || num_rows < 2) {
      return false;
    }

    return m.every((c) => c.length === num_rows);
  }

  /** @returns an FPMatrix representation of an array of an array of values if possible */
  toMatrix(m) {
    if (
    this.isMatrix(m) &&
    every2DArray(m, (e) => {
      return e.kind === this.kind;
    }))
    {
      return m;
    }

    const result = map2DArray(m, this.toInterval.bind(this));

    // The return of the map above is a ROArrayArray<FPInterval>, which needs to be
    // narrowed to FPMatrix, since FPMatrix is defined as fixed length tuples.
    if (this.isMatrix(result)) {
      return result;
    }
    unreachable(`Cannot convert ${m} to FPMatrix`);
  }

  /**
   * @returns a FPMatrix where each element is the span for corresponding
   *          elements at the same index in the input matrices
   */
  spanMatrices(...matrices) {
    // Coercing the type of matrices, since tuples are not generally compatible
    // with Arrays, but they are functionally equivalent for the usages in this
    // function.
    const ms = matrices;
    const num_cols = ms[0].length;
    const num_rows = ms[0][0].length;
    assert(
      ms.every((m) => m.length === num_cols && m.every((r) => r.length === num_rows)),
      `Matrix span is not defined for Matrices of differing dimensions`
    );

    const result = [...Array(num_cols)].map((_) => [...Array(num_rows)]);
    for (let i = 0; i < num_cols; i++) {
      for (let j = 0; j < num_rows; j++) {
        result[i][j] = this.spanIntervals(...ms.map((m) => m[i][j]));
      }
    }

    return this.toMatrix(result);
  }

  /** @returns input with an appended 0, if inputs contains non-zero subnormals */
  addFlushedIfNeeded(values) {
    const subnormals = values.filter(this.isSubnormal);
    const needs_zero = subnormals.length > 0 && subnormals.every((s) => s !== 0);
    return needs_zero ? values.concat(0) : values;
  }

  /** Stub for scalar to interval generator */
  unimplementedScalarToInterval(name, _x) {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for scalar pair to interval generator */
  unimplementedScalarPairToInterval(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is yet implemented for '${this.kind}'`);
  }

  /** Stub for scalar triple to interval generator */
  unimplementedScalarTripleToInterval(
  name,
  _x,
  _y,
  _z)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for scalar to vector generator */
  unimplementedScalarToVector(name, _x) {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for vector to interval generator */
  unimplementedVectorToInterval(name, _x) {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for vector pair to interval generator */
  unimplementedVectorPairToInterval(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for vector to vector generator */
  unimplementedVectorToVector(
  name,
  _x)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for vector pair to vector generator */
  unimplementedVectorPairToVector(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for vector-scalar to vector generator */
  unimplementedVectorScalarToVector(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for scalar-vector to vector generator */
  unimplementedScalarVectorToVector(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for matrix to interval generator */
  unimplementedMatrixToInterval(name, _x) {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for matrix to matirx generator */
  unimplementedMatrixToMatrix(name, _x) {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for matrix pair to matrix generator */
  unimplementedMatrixPairToMatrix(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for matrix-scalar to matrix generator  */
  unimplementedMatrixScalarToMatrix(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for scalar-matrix to matrix generator  */
  unimplementedScalarMatrixToMatrix(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for matrix-vector to vector generator  */
  unimplementedMatrixVectorToVector(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for vector-matrix to vector generator  */
  unimplementedVectorMatrixToVector(
  name,
  _x,
  _y)
  {
    unreachable(`'${name}' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for distance generator */
  unimplementedDistance(
  _x,
  _y)
  {
    unreachable(`'distance' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for faceForward */
  unimplementedFaceForward(
  _x,
  _y,
  _z)
  {
    unreachable(`'faceForward' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for length generator */
  unimplementedLength(
  _x)
  {
    unreachable(`'length' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for modf generator */
  unimplementedModf(_x) {
    unreachable(`'modf' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for refract generator */
  unimplementedRefract(
  _i,
  _s,
  _r)
  {
    unreachable(`'refract' is not yet implemented for '${this.kind}'`);
  }

  /** Stub for absolute errors */
  unimplementedAbsoluteErrorInterval(_n, _error_range) {
    unreachable(`Absolute Error is not implement for '${this.kind}'`);
  }

  /** Stub for ULP errors */
  unimplementedUlpInterval(_n, _numULP) {
    unreachable(`ULP Error is not implement for '${this.kind}'`);
  }

  // Utilities - Defined by subclass
  /**
   * @returns the nearest precise value to the input. Rounding should be IEEE
   *          'roundTiesToEven'.
   */

  /** @returns all valid roundings of input */

  /** @returns true if input is considered finite, otherwise false */

  /** @returns true if input is considered subnormal, otherwise false */

  /** @returns 0 if the provided number is subnormal, otherwise returns the proved number */

  /** @returns 1 * ULP: (number) */

  /** @returns a builder for converting numbers to ScalarsValues */

  /** @returns a range of scalars for testing */

  /** @returns a reduced range of scalars for testing */

  /** @returns a range of dim element vectors for testing */

  /** @returns a reduced range of dim element vectors for testing */

  /** @returns a reduced range of cols x rows matrices for testing
   *
   * A non-sparse version of this generator is intentionally not provided due to
   * runtime issues with more dense ranges.
   */


  // Framework - Cases

  /**
   * @returns a Case for the param and the interval generator provided.
   * The Case will use an interval comparator for matching results.
   * @param param the param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  makeScalarToIntervalCase(
  param,
  filter,
  ...ops)
  {
    param = this.quantize(param);

    const intervals = ops.map((o) => o(param));
    if (filter === 'finite' && intervals.some((i) => !i.isFinite())) {
      return undefined;
    }
    return { input: [this.scalarBuilder(param)], expected: anyOf(...intervals) };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param params array of inputs to try
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  generateScalarToIntervalCases(
  params,
  filter,
  ...ops)
  {
    return params.reduce((cases, e) => {
      const c = this.makeScalarToIntervalCase(e, filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the params and the interval generator provided.
   * The Case will use an interval comparator for matching results.
   * @param param0 the first param to pass in
   * @param param1 the second param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  makeScalarPairToIntervalCase(
  param0,
  param1,
  filter,
  ...ops)
  {
    param0 = this.quantize(param0);
    param1 = this.quantize(param1);

    const intervals = ops.map((o) => o(param0, param1));
    if (filter === 'finite' && intervals.some((i) => !i.isFinite())) {
      return undefined;
    }
    return {
      input: [this.scalarBuilder(param0), this.scalarBuilder(param1)],
      expected: anyOf(...intervals)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param param0s array of inputs to try for the first input
   * @param param1s array of inputs to try for the second input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  generateScalarPairToIntervalCases(
  param0s,
  param1s,
  filter,
  ...ops)
  {
    return cartesianProduct(param0s, param1s).reduce((cases, e) => {
      const c = this.makeScalarPairToIntervalCase(e[0], e[1], filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the params and the interval generator provided.
   * The Case will use an interval comparator for matching results.
   * @param param0 the first param to pass in
   * @param param1 the second param to pass in
   * @param param2 the third param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  makeScalarTripleToIntervalCase(
  param0,
  param1,
  param2,
  filter,
  ...ops)
  {
    param0 = this.quantize(param0);
    param1 = this.quantize(param1);
    param2 = this.quantize(param2);

    const intervals = ops.map((o) => o(param0, param1, param2));
    if (filter === 'finite' && intervals.some((i) => !i.isFinite())) {
      return undefined;
    }
    return {
      input: [this.scalarBuilder(param0), this.scalarBuilder(param1), this.scalarBuilder(param2)],
      expected: anyOf(...intervals)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param param0s array of inputs to try for the first input
   * @param param1s array of inputs to try for the second input
   * @param param2s array of inputs to try for the third input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  generateScalarTripleToIntervalCases(
  param0s,
  param1s,
  param2s,
  filter,
  ...ops)
  {
    return cartesianProduct(param0s, param1s, param2s).reduce((cases, e) => {
      const c = this.makeScalarTripleToIntervalCase(e[0], e[1], e[2], filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the params and the interval generator provided.
   * The Case will use an interval comparator for matching results.
   * @param param the param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  makeVectorToIntervalCase(
  param,
  filter,
  ...ops)
  {
    param = param.map(this.quantize);

    const intervals = ops.map((o) => o(param));
    if (filter === 'finite' && intervals.some((i) => !i.isFinite())) {
      return undefined;
    }
    return {
      input: [toVector(param, this.scalarBuilder)],
      expected: anyOf(...intervals)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param params array of inputs to try
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  generateVectorToIntervalCases(
  params,
  filter,
  ...ops)
  {
    return params.reduce((cases, e) => {
      const c = this.makeVectorToIntervalCase(e, filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the params and the interval generator provided.
   * The Case will use an interval comparator for matching results.
   * @param param0 the first param to pass in
   * @param param1 the second param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  makeVectorPairToIntervalCase(
  param0,
  param1,
  filter,
  ...ops)
  {
    param0 = param0.map(this.quantize);
    param1 = param1.map(this.quantize);

    const intervals = ops.map((o) => o(param0, param1));
    if (filter === 'finite' && intervals.some((i) => !i.isFinite())) {
      return undefined;
    }
    return {
      input: [toVector(param0, this.scalarBuilder), toVector(param1, this.scalarBuilder)],
      expected: anyOf(...intervals)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param param0s array of inputs to try for the first input
   * @param param1s array of inputs to try for the second input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  generateVectorPairToIntervalCases(
  param0s,
  param1s,
  filter,
  ...ops)
  {
    return cartesianProduct(param0s, param1s).reduce((cases, e) => {
      const c = this.makeVectorPairToIntervalCase(e[0], e[1], filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the param and vector of intervals generator provided
   * @param param the param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance
   *            intervals.
   */
  makeVectorToVectorCase(
  param,
  filter,
  ...ops)
  {
    param = param.map(this.quantize);

    const vectors = ops.map((o) => o(param));
    if (filter === 'finite' && vectors.some((v) => v.some((e) => !e.isFinite()))) {
      return undefined;
    }
    return {
      input: [toVector(param, this.scalarBuilder)],
      expected: anyOf(...vectors)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param params array of inputs to try
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance
   *            intervals.
   */
  generateVectorToVectorCases(
  params,
  filter,
  ...ops)
  {
    return params.reduce((cases, e) => {
      const c = this.makeVectorToVectorCase(e, filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the params and the interval vector generator provided.
   * The Case will use an interval comparator for matching results.
   * @param scalar the scalar param to pass in
   * @param vector the vector param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance intervals
   */
  makeScalarVectorToVectorCase(
  scalar,
  vector,
  filter,
  ...ops)
  {
    scalar = this.quantize(scalar);
    vector = vector.map(this.quantize);

    const results = ops.map((o) => o(scalar, vector));
    if (filter === 'finite' && results.some((r) => r.some((e) => !e.isFinite()))) {
      return undefined;
    }
    return {
      input: [this.scalarBuilder(scalar), toVector(vector, this.scalarBuilder)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param scalars array of scalar inputs to try
   * @param vectors array of vector inputs to try
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance intervals
   */
  generateScalarVectorToVectorCases(
  scalars,
  vectors,
  filter,
  ...ops)
  {
    // Cannot use cartesianProduct here, due to heterogeneous types
    const cases = [];
    scalars.forEach((scalar) => {
      vectors.forEach((vector) => {
        const c = this.makeScalarVectorToVectorCase(scalar, vector, filter, ...ops);
        if (c !== undefined) {
          cases.push(c);
        }
      });
    });
    return cases;
  }

  /**
   * @returns a Case for the params and the interval vector generator provided.
   * The Case will use an interval comparator for matching results.
   * @param vector the vector param to pass in
   * @param scalar the scalar param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance intervals
   */
  makeVectorScalarToVectorCase(
  vector,
  scalar,
  filter,
  ...ops)
  {
    vector = vector.map(this.quantize);
    scalar = this.quantize(scalar);

    const results = ops.map((o) => o(vector, scalar));
    if (filter === 'finite' && results.some((r) => r.some((e) => !e.isFinite()))) {
      return undefined;
    }
    return {
      input: [toVector(vector, this.scalarBuilder), this.scalarBuilder(scalar)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param vectors array of vector inputs to try
   * @param scalars array of scalar inputs to try
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance intervals
   */
  generateVectorScalarToVectorCases(
  vectors,
  scalars,
  filter,
  ...ops)
  {
    // Cannot use cartesianProduct here, due to heterogeneous types
    const cases = [];
    vectors.forEach((vector) => {
      scalars.forEach((scalar) => {
        const c = this.makeVectorScalarToVectorCase(vector, scalar, filter, ...ops);
        if (c !== undefined) {
          cases.push(c);
        }
      });
    });
    return cases;
  }

  /**
   * @returns a Case for the param and vector of intervals generator provided
   * @param param0 the first param to pass in
   * @param param1 the second param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance
   *            intervals.
   */
  makeVectorPairToVectorCase(
  param0,
  param1,
  filter,
  ...ops)
  {
    param0 = param0.map(this.quantize);
    param1 = param1.map(this.quantize);
    const vectors = ops.map((o) => o(param0, param1));
    if (filter === 'finite' && vectors.some((v) => v.some((e) => !e.isFinite()))) {
      return undefined;
    }
    return {
      input: [toVector(param0, this.scalarBuilder), toVector(param1, this.scalarBuilder)],
      expected: anyOf(...vectors)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param param0s array of inputs to try for the first input
   * @param param1s array of inputs to try for the second input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance
   *            intervals.
   */
  generateVectorPairToVectorCases(
  param0s,
  param1s,
  filter,
  ...ops)
  {
    return cartesianProduct(param0s, param1s).reduce((cases, e) => {
      const c = this.makeVectorPairToVectorCase(e[0], e[1], filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the params and the component-wise interval generator provided.
   * The Case will use an interval comparator for matching results.
   * @param param0 the first vector param to pass in
   * @param param1 the second vector param to pass in
   * @param param2 the scalar param to pass in
   * @param filter what interval filtering to apply
   * @param componentWiseOps callbacks that implement generating a component-wise acceptance interval,
   *                         one component result at a time.
   */
  makeVectorPairScalarToVectorComponentWiseCase(
  param0,
  param1,
  param2,
  filter,
  ...componentWiseOps)
  {
    // Width of input vector
    const width = param0.length;
    assert(2 <= width && width <= 4, 'input vector width must between 2 and 4');
    assert(param1.length === width, 'two input vectors must have the same width');
    param0 = param0.map(this.quantize);
    param1 = param1.map(this.quantize);
    param2 = this.quantize(param2);

    // Call the component-wise interval generator and build the expectation FPVector
    const results = componentWiseOps.map((o) => {
      return param0.map((el0, index) => o(el0, param1[index], param2));
    });
    if (filter === 'finite' && results.some((r) => r.some((e) => !e.isFinite()))) {
      return undefined;
    }
    return {
      input: [
      toVector(param0, this.scalarBuilder),
      toVector(param1, this.scalarBuilder),
      this.scalarBuilder(param2)],

      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param param0s array of first vector inputs to try
   * @param param1s array of second vector inputs to try
   * @param param2s array of scalar inputs to try
   * @param filter what interval filtering to apply
   * @param componentWiseOpscallbacks that implement generating a component-wise acceptance interval
   */
  generateVectorPairScalarToVectorComponentWiseCase(
  param0s,
  param1s,
  param2s,
  filter,
  ...componentWiseOps)
  {
    // Cannot use cartesianProduct here, due to heterogeneous types
    const cases = [];
    param0s.forEach((param0) => {
      param1s.forEach((param1) => {
        param2s.forEach((param2) => {
          const c = this.makeVectorPairScalarToVectorComponentWiseCase(
            param0,
            param1,
            param2,
            filter,
            ...componentWiseOps
          );
          if (c !== undefined) {
            cases.push(c);
          }
        });
      });
    });
    return cases;
  }

  /**
   * @returns a Case for the param and an array of interval generators provided
   * @param param the param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  makeMatrixToScalarCase(
  param,
  filter,
  ...ops)
  {
    param = map2DArray(param, this.quantize);

    const results = ops.map((o) => o(param));
    if (filter === 'finite' && results.some((e) => !e.isFinite())) {
      return undefined;
    }

    return {
      input: [toMatrix(param, this.scalarBuilder)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param params array of inputs to try
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  generateMatrixToScalarCases(
  params,
  filter,
  ...ops)
  {
    return params.reduce((cases, e) => {
      const c = this.makeMatrixToScalarCase(e, filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the param and an array of interval generators provided
   * @param param the param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a matrix of acceptance
   *            intervals
   */
  makeMatrixToMatrixCase(
  param,
  filter,
  ...ops)
  {
    param = map2DArray(param, this.quantize);

    const results = ops.map((o) => o(param));
    if (filter === 'finite' && results.some((m) => m.some((c) => c.some((r) => !r.isFinite())))) {
      return undefined;
    }

    return {
      input: [toMatrix(param, this.scalarBuilder)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param params array of inputs to try
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a matrix of acceptance
   *            intervals
   */
  generateMatrixToMatrixCases(
  params,
  filter,
  ...ops)
  {
    return params.reduce((cases, e) => {
      const c = this.makeMatrixToMatrixCase(e, filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the params and matrix of intervals generator provided
   * @param param0 the first param to pass in
   * @param param1 the second param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a matrix of acceptance
   *            intervals
   */
  makeMatrixPairToMatrixCase(
  param0,
  param1,
  filter,
  ...ops)
  {
    param0 = map2DArray(param0, this.quantize);
    param1 = map2DArray(param1, this.quantize);
    const results = ops.map((o) => o(param0, param1));
    if (filter === 'finite' && results.some((m) => m.some((c) => c.some((r) => !r.isFinite())))) {
      return undefined;
    }
    return {
      input: [toMatrix(param0, this.scalarBuilder), toMatrix(param1, this.scalarBuilder)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param param0s array of inputs to try for the first input
   * @param param1s array of inputs to try for the second input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a matrix of acceptance
   *            intervals
   */
  generateMatrixPairToMatrixCases(
  param0s,
  param1s,
  filter,
  ...ops)
  {
    return cartesianProduct(param0s, param1s).reduce((cases, e) => {
      const c = this.makeMatrixPairToMatrixCase(e[0], e[1], filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  /**
   * @returns a Case for the params and matrix of intervals generator provided
   * @param mat the matrix param to pass in
   * @param scalar the scalar to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a matrix of acceptance
   *            intervals
   */
  makeMatrixScalarToMatrixCase(
  mat,
  scalar,
  filter,
  ...ops)
  {
    mat = map2DArray(mat, this.quantize);
    scalar = this.quantize(scalar);

    const results = ops.map((o) => o(mat, scalar));
    if (filter === 'finite' && results.some((m) => m.some((c) => c.some((r) => !r.isFinite())))) {
      return undefined;
    }
    return {
      input: [toMatrix(mat, this.scalarBuilder), this.scalarBuilder(scalar)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param mats array of inputs to try for the matrix input
   * @param scalars array of inputs to try for the scalar input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a matrix of acceptance
   *            intervals
   */
  generateMatrixScalarToMatrixCases(
  mats,
  scalars,
  filter,
  ...ops)
  {
    // Cannot use cartesianProduct here, due to heterogeneous types
    const cases = [];
    mats.forEach((mat) => {
      scalars.forEach((scalar) => {
        const c = this.makeMatrixScalarToMatrixCase(mat, scalar, filter, ...ops);
        if (c !== undefined) {
          cases.push(c);
        }
      });
    });
    return cases;
  }

  /**
   * @returns a Case for the params and matrix of intervals generator provided
   * @param scalar the scalar to pass in
   * @param mat the matrix param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a matrix of acceptance
   *            intervals
   */
  makeScalarMatrixToMatrixCase(
  scalar,
  mat,
  filter,
  ...ops)
  {
    scalar = this.quantize(scalar);
    mat = map2DArray(mat, this.quantize);

    const results = ops.map((o) => o(scalar, mat));
    if (filter === 'finite' && results.some((m) => m.some((c) => c.some((r) => !r.isFinite())))) {
      return undefined;
    }
    return {
      input: [this.scalarBuilder(scalar), toMatrix(mat, this.scalarBuilder)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param scalars array of inputs to try for the scalar input
   * @param mats array of inputs to try for the matrix input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a matrix of acceptance
   *            intervals
   */
  generateScalarMatrixToMatrixCases(
  scalars,
  mats,
  filter,
  ...ops)
  {
    // Cannot use cartesianProduct here, due to heterogeneous types
    const cases = [];
    mats.forEach((mat) => {
      scalars.forEach((scalar) => {
        const c = this.makeScalarMatrixToMatrixCase(scalar, mat, filter, ...ops);
        if (c !== undefined) {
          cases.push(c);
        }
      });
    });
    return cases;
  }

  /**
   * @returns a Case for the params and the vector of intervals generator provided
   * @param mat the matrix param to pass in
   * @param vec the vector to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance
   *            intervals
   */
  makeMatrixVectorToVectorCase(
  mat,
  vec,
  filter,
  ...ops)
  {
    mat = map2DArray(mat, this.quantize);
    vec = vec.map(this.quantize);

    const results = ops.map((o) => o(mat, vec));
    if (filter === 'finite' && results.some((v) => v.some((e) => !e.isFinite()))) {
      return undefined;
    }
    return {
      input: [toMatrix(mat, this.scalarBuilder), toVector(vec, this.scalarBuilder)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param mats array of inputs to try for the matrix input
   * @param vecs array of inputs to try for the vector input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance
   *            intervals
   */
  generateMatrixVectorToVectorCases(
  mats,
  vecs,
  filter,
  ...ops)
  {
    // Cannot use cartesianProduct here, due to heterogeneous types
    const cases = [];
    mats.forEach((mat) => {
      vecs.forEach((vec) => {
        const c = this.makeMatrixVectorToVectorCase(mat, vec, filter, ...ops);
        if (c !== undefined) {
          cases.push(c);
        }
      });
    });
    return cases;
  }

  /**
   * @returns a Case for the params and the vector of intervals generator provided
   * @param vec the vector to pass in
   * @param mat the matrix param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance
   *            intervals
   */
  makeVectorMatrixToVectorCase(
  vec,
  mat,
  filter,
  ...ops)
  {
    vec = vec.map(this.quantize);
    mat = map2DArray(mat, this.quantize);

    const results = ops.map((o) => o(vec, mat));
    if (filter === 'finite' && results.some((v) => v.some((e) => !e.isFinite()))) {
      return undefined;
    }
    return {
      input: [toVector(vec, this.scalarBuilder), toMatrix(mat, this.scalarBuilder)],
      expected: anyOf(...results)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param vecs array of inputs to try for the vector input
   * @param mats array of inputs to try for the matrix input
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating a vector of acceptance
   *            intervals
   */
  generateVectorMatrixToVectorCases(
  vecs,
  mats,
  filter,
  ...ops)
  {
    // Cannot use cartesianProduct here, due to heterogeneous types
    const cases = [];
    vecs.forEach((vec) => {
      mats.forEach((mat) => {
        const c = this.makeVectorMatrixToVectorCase(vec, mat, filter, ...ops);
        if (c !== undefined) {
          cases.push(c);
        }
      });
    });
    return cases;
  }

  // Framework - Intervals

  /**
   * Converts a point to an acceptance interval, using a specific function
   *
   * This handles correctly rounding and flushing inputs as needed.
   * Duplicate inputs are pruned before invoking op.impl.
   * op.extrema is invoked before this point in the call stack.
   * op.domain is tested before this point in the call stack.
   *
   * @param n value to flush & round then invoke op.impl on
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  roundAndFlushScalarToInterval(n, op) {
    assert(!Number.isNaN(n), `flush not defined for NaN`);
    const values = this.correctlyRounded(n);
    const inputs = this.addFlushedIfNeeded(values);

    if (op.domain !== undefined) {
      // Cannot invoke op.domain() directly in the .some, because the narrowing doesn't propegate.
      const domain = op.domain();
      if (inputs.some((i) => !domain.contains(i))) {
        return this.constants().unboundedInterval;
      }
    }

    const results = new Set(inputs.map(op.impl));
    return this.spanIntervals(...results);
  }

  /**
   * Converts a pair to an acceptance interval, using a specific function
   *
   * This handles correctly rounding and flushing inputs as needed.
   * Duplicate inputs are pruned before invoking op.impl.
   * All unique combinations of x & y are run.
   * op.extrema is invoked before this point in the call stack.
   * op.domain is tested before this point in the call stack.
   *
   * @param x first param to flush & round then invoke op.impl on
   * @param y second param to flush & round then invoke op.impl on
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  roundAndFlushScalarPairToInterval(
  x,
  y,
  op)
  {
    assert(!Number.isNaN(x), `flush not defined for NaN`);
    assert(!Number.isNaN(y), `flush not defined for NaN`);

    const x_values = this.correctlyRounded(x);
    const y_values = this.correctlyRounded(y);
    const x_inputs = this.addFlushedIfNeeded(x_values);
    const y_inputs = this.addFlushedIfNeeded(y_values);

    if (op.domain !== undefined) {
      // Cannot invoke op.domain() directly in the .some, because the narrowing doesn't propegate.
      const domain = op.domain();

      if (x_inputs.some((i) => !domain.x.some((e) => e.contains(i)))) {
        return this.constants().unboundedInterval;
      }

      if (y_inputs.some((j) => !domain.y.some((e) => e.contains(j)))) {
        return this.constants().unboundedInterval;
      }
    }

    const intervals = new Set();
    x_inputs.forEach((inner_x) => {
      y_inputs.forEach((inner_y) => {
        intervals.add(op.impl(inner_x, inner_y));
      });
    });
    return this.spanIntervals(...intervals);
  }

  /**
   * Converts a triplet to an acceptance interval, using a specific function
   *
   * This handles correctly rounding and flushing inputs as needed.
   * Duplicate inputs are pruned before invoking op.impl.
   * All unique combinations of x, y & z are run.
   *
   * @param x first param to flush & round then invoke op.impl on
   * @param y second param to flush & round then invoke op.impl on
   * @param z third param to flush & round then invoke op.impl on
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  roundAndFlushScalarTripleToInterval(
  x,
  y,
  z,
  op)
  {
    assert(!Number.isNaN(x), `flush not defined for NaN`);
    assert(!Number.isNaN(y), `flush not defined for NaN`);
    assert(!Number.isNaN(z), `flush not defined for NaN`);
    const x_values = this.correctlyRounded(x);
    const y_values = this.correctlyRounded(y);
    const z_values = this.correctlyRounded(z);
    const x_inputs = this.addFlushedIfNeeded(x_values);
    const y_inputs = this.addFlushedIfNeeded(y_values);
    const z_inputs = this.addFlushedIfNeeded(z_values);
    const intervals = new Set();

    x_inputs.forEach((inner_x) => {
      y_inputs.forEach((inner_y) => {
        z_inputs.forEach((inner_z) => {
          intervals.add(op.impl(inner_x, inner_y, inner_z));
        });
      });
    });

    return this.spanIntervals(...intervals);
  }

  /**
   * Converts a vector to an acceptance interval using a specific function
   *
   * This handles correctly rounding and flushing inputs as needed.
   * Duplicate inputs are pruned before invoking op.impl.
   *
   * @param x param to flush & round then invoke op.impl on
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  roundAndFlushVectorToInterval(x, op) {
    assert(
      x.every((e) => !Number.isNaN(e)),
      `flush not defined for NaN`
    );

    const x_rounded = x.map(this.correctlyRounded);
    const x_flushed = x_rounded.map(this.addFlushedIfNeeded.bind(this));
    const x_inputs = cartesianProduct(...x_flushed);

    const intervals = new Set();
    x_inputs.forEach((inner_x) => {
      intervals.add(op.impl(inner_x));
    });
    return this.spanIntervals(...intervals);
  }

  /**
   * Converts a pair of vectors to an acceptance interval using a specific
   * function
   *
   * This handles correctly rounding and flushing inputs as needed.
   * Duplicate inputs are pruned before invoking op.impl.
   * All unique combinations of x & y are run.
   *
   * @param x first param to flush & round then invoke op.impl on
   * @param y second param to flush & round then invoke op.impl on
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  roundAndFlushVectorPairToInterval(
  x,
  y,
  op)
  {
    assert(
      x.every((e) => !Number.isNaN(e)),
      `flush not defined for NaN`
    );
    assert(
      y.every((e) => !Number.isNaN(e)),
      `flush not defined for NaN`
    );

    const x_rounded = x.map(this.correctlyRounded);
    const y_rounded = y.map(this.correctlyRounded);
    const x_flushed = x_rounded.map(this.addFlushedIfNeeded.bind(this));
    const y_flushed = y_rounded.map(this.addFlushedIfNeeded.bind(this));
    const x_inputs = cartesianProduct(...x_flushed);
    const y_inputs = cartesianProduct(...y_flushed);

    const intervals = new Set();
    x_inputs.forEach((inner_x) => {
      y_inputs.forEach((inner_y) => {
        intervals.add(op.impl(inner_x, inner_y));
      });
    });
    return this.spanIntervals(...intervals);
  }

  /**
   * Converts a vector to a vector of acceptance intervals using a specific
   * function
   *
   * This handles correctly rounding and flushing inputs as needed.
   * Duplicate inputs are pruned before invoking op.impl.
   *
   * @param x param to flush & round then invoke op.impl on
   * @param op operation defining the function being run
   * @returns a vector of spans for each outputs of op.impl
   */
  roundAndFlushVectorToVector(x, op) {
    assert(
      x.every((e) => !Number.isNaN(e)),
      `flush not defined for NaN`
    );

    const x_rounded = x.map(this.correctlyRounded);
    const x_flushed = x_rounded.map(this.addFlushedIfNeeded.bind(this));
    const x_inputs = cartesianProduct(...x_flushed);

    const interval_vectors = new Set();
    x_inputs.forEach((inner_x) => {
      interval_vectors.add(op.impl(inner_x));
    });

    return this.spanVectors(...interval_vectors);
  }

  /**
   * Converts a pair of vectors to a vector of acceptance intervals using a
   * specific function
   *
   * This handles correctly rounding and flushing inputs as needed.
   * Duplicate inputs are pruned before invoking op.impl.
   *
   * @param x first param to flush & round then invoke op.impl on
   * @param y second param to flush & round then invoke op.impl on
   * @param op operation defining the function being run
   * @returns a vector of spans for each output of op.impl
   */
  roundAndFlushVectorPairToVector(
  x,
  y,
  op)
  {
    assert(
      x.every((e) => !Number.isNaN(e)),
      `flush not defined for NaN`
    );
    assert(
      y.every((e) => !Number.isNaN(e)),
      `flush not defined for NaN`
    );

    const x_rounded = x.map(this.correctlyRounded);
    const y_rounded = y.map(this.correctlyRounded);
    const x_flushed = x_rounded.map(this.addFlushedIfNeeded.bind(this));
    const y_flushed = y_rounded.map(this.addFlushedIfNeeded.bind(this));
    const x_inputs = cartesianProduct(...x_flushed);
    const y_inputs = cartesianProduct(...y_flushed);

    const interval_vectors = new Set();
    x_inputs.forEach((inner_x) => {
      y_inputs.forEach((inner_y) => {
        interval_vectors.add(op.impl(inner_x, inner_y));
      });
    });

    return this.spanVectors(...interval_vectors);
  }

  /**
   * Converts a matrix to a matrix of acceptance intervals using a specific
   * function
   *
   * This handles correctly rounding and flushing inputs as needed.
   * Duplicate inputs are pruned before invoking op.impl.
   *
   * @param m param to flush & round then invoke op.impl on
   * @param op operation defining the function being run
   * @returns a matrix of spans for each outputs of op.impl
   */
  roundAndFlushMatrixToMatrix(m, op) {
    const num_cols = m.length;
    const num_rows = m[0].length;
    assert(
      m.every((c) => c.every((r) => !Number.isNaN(r))),
      `flush not defined for NaN`
    );

    const m_flat = flatten2DArray(m);
    const m_rounded = m_flat.map(this.correctlyRounded);
    const m_flushed = m_rounded.map(this.addFlushedIfNeeded.bind(this));
    const m_options = cartesianProduct(...m_flushed);
    const m_inputs = m_options.map((e) =>
    unflatten2DArray(e, num_cols, num_rows)
    );

    const interval_matrices = new Set();
    m_inputs.forEach((inner_m) => {
      interval_matrices.add(op.impl(inner_m));
    });

    return this.spanMatrices(...interval_matrices);
  }

  /**
   * Calculate the acceptance interval for a unary function over an interval
   *
   * If the interval is actually a point, this just decays to
   * roundAndFlushScalarToInterval.
   *
   * The provided domain interval may be adjusted if the operation defines an
   * extrema function.
   *
   * @param x input domain interval
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  runScalarToIntervalOp(x, op) {
    if (!x.isFinite()) {
      return this.constants().unboundedInterval;
    }

    if (op.extrema !== undefined) {
      x = op.extrema(x);
    }

    const result = this.spanIntervals(
      ...x.endpoints().map((b) => this.roundAndFlushScalarToInterval(b, op))
    );
    return result.isFinite() ? result : this.constants().unboundedInterval;
  }

  /**
   * Calculate the acceptance interval for a binary function over an interval
   *
   * The provided domain intervals may be adjusted if the operation defines an
   * extrema function.
   *
   * @param x first input domain interval
   * @param y second input domain interval
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  runScalarPairToIntervalOp(
  x,
  y,
  op)
  {
    if (!x.isFinite() || !y.isFinite()) {
      return this.constants().unboundedInterval;
    }

    if (op.extrema !== undefined) {
      [x, y] = op.extrema(x, y);
    }

    const outputs = new Set();
    x.endpoints().forEach((inner_x) => {
      y.endpoints().forEach((inner_y) => {
        outputs.add(this.roundAndFlushScalarPairToInterval(inner_x, inner_y, op));
      });
    });

    const result = this.spanIntervals(...outputs);
    return result.isFinite() ? result : this.constants().unboundedInterval;
  }

  /**
   * Calculate the acceptance interval for a ternary function over an interval
   *
   * @param x first input domain interval
   * @param y second input domain interval
   * @param z third input domain interval
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  runScalarTripleToIntervalOp(
  x,
  y,
  z,
  op)
  {
    if (!x.isFinite() || !y.isFinite() || !z.isFinite()) {
      return this.constants().unboundedInterval;
    }

    const outputs = new Set();
    x.endpoints().forEach((inner_x) => {
      y.endpoints().forEach((inner_y) => {
        z.endpoints().forEach((inner_z) => {
          outputs.add(this.roundAndFlushScalarTripleToInterval(inner_x, inner_y, inner_z, op));
        });
      });
    });

    const result = this.spanIntervals(...outputs);
    return result.isFinite() ? result : this.constants().unboundedInterval;
  }

  /**
   * Calculate the acceptance interval for a vector function over given
   * intervals
   *
   * @param x input domain intervals vector
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  runVectorToIntervalOp(x, op) {
    if (x.some((e) => !e.isFinite())) {
      return this.constants().unboundedInterval;
    }

    const x_values = cartesianProduct(...x.map((e) => e.endpoints()));

    const outputs = new Set();
    x_values.forEach((inner_x) => {
      outputs.add(this.roundAndFlushVectorToInterval(inner_x, op));
    });

    const result = this.spanIntervals(...outputs);
    return result.isFinite() ? result : this.constants().unboundedInterval;
  }

  /**
   * Calculate the acceptance interval for a vector pair function over given
   * intervals
   *
   * @param x first input domain intervals vector
   * @param y second input domain intervals vector
   * @param op operation defining the function being run
   * @returns a span over all the outputs of op.impl
   */
  runVectorPairToIntervalOp(
  x,
  y,
  op)
  {
    if (x.some((e) => !e.isFinite()) || y.some((e) => !e.isFinite())) {
      return this.constants().unboundedInterval;
    }

    const x_values = cartesianProduct(...x.map((e) => e.endpoints()));
    const y_values = cartesianProduct(...y.map((e) => e.endpoints()));

    const outputs = new Set();
    x_values.forEach((inner_x) => {
      y_values.forEach((inner_y) => {
        outputs.add(this.roundAndFlushVectorPairToInterval(inner_x, inner_y, op));
      });
    });

    const result = this.spanIntervals(...outputs);
    return result.isFinite() ? result : this.constants().unboundedInterval;
  }

  /**
   * Calculate the vector of acceptance intervals for a pair of vector function
   * over given intervals
   *
   * @param x input domain intervals vector
   * @param op operation defining the function being run
   * @returns a vector of spans over all the outputs of op.impl
   */
  runVectorToVectorOp(x, op) {
    if (x.some((e) => !e.isFinite())) {
      return this.constants().unboundedVector[x.length];
    }

    const x_values = cartesianProduct(...x.map((e) => e.endpoints()));

    const outputs = new Set();
    x_values.forEach((inner_x) => {
      outputs.add(this.roundAndFlushVectorToVector(inner_x, op));
    });

    const result = this.spanVectors(...outputs);
    return result.every((e) => e.isFinite()) ?
    result :
    this.constants().unboundedVector[result.length];
  }

  /**
   * Calculate the vector of acceptance intervals by running a scalar operation
   * component-wise over a vector.
   *
   * This is used for situations where a component-wise operation, like vector
   * negation, is needed as part of an inherited accuracy, but the top-level
   * operation test don't require an explicit vector definition of the function,
   * due to the generated 'vectorize' tests being sufficient.
   *
   * @param x input domain intervals vector
   * @param op scalar operation to be run component-wise
   * @returns a vector of intervals with the outputs of op.impl
   */
  runScalarToIntervalOpComponentWise(x, op) {
    return this.toVector(x.map((e) => this.runScalarToIntervalOp(e, op)));
  }

  /**
   * Calculate the vector of acceptance intervals for a vector function over
   * given intervals
   *
   * @param x first input domain intervals vector
   * @param y second input domain intervals vector
   * @param op operation defining the function being run
   * @returns a vector of spans over all the outputs of op.impl
   */
  runVectorPairToVectorOp(x, y, op) {
    if (x.some((e) => !e.isFinite()) || y.some((e) => !e.isFinite())) {
      return this.constants().unboundedVector[x.length];
    }

    const x_values = cartesianProduct(...x.map((e) => e.endpoints()));
    const y_values = cartesianProduct(...y.map((e) => e.endpoints()));

    const outputs = new Set();
    x_values.forEach((inner_x) => {
      y_values.forEach((inner_y) => {
        outputs.add(this.roundAndFlushVectorPairToVector(inner_x, inner_y, op));
      });
    });

    const result = this.spanVectors(...outputs);
    return result.every((e) => e.isFinite()) ?
    result :
    this.constants().unboundedVector[result.length];
  }

  /**
   * Calculate the vector of acceptance intervals by running a scalar operation
   * component-wise over a pair of vectors.
   *
   * This is used for situations where a component-wise operation, like vector
   * subtraction, is needed as part of an inherited accuracy, but the top-level
   * operation test don't require an explicit vector definition of the function,
   * due to the generated 'vectorize' tests being sufficient.
   *
   * @param x first input domain intervals vector
   * @param y second input domain intervals vector
   * @param op scalar operation to be run component-wise
   * @returns a vector of intervals with the outputs of op.impl
   */
  runScalarPairToIntervalOpVectorComponentWise(
  x,
  y,
  op)
  {
    assert(
      x.length === y.length,
      `runScalarPairToIntervalOpVectorComponentWise requires vectors of the same dimensions`
    );

    return this.toVector(
      x.map((i, idx) => {
        return this.runScalarPairToIntervalOp(i, y[idx], op);
      })
    );
  }

  /**
   * Calculate the matrix of acceptance intervals for a pair of matrix function over
   * given intervals
   *
   * @param m input domain intervals matrix
   * @param op operation defining the function being run
   * @returns a matrix of spans over all the outputs of op.impl
   */
  runMatrixToMatrixOp(m, op) {
    const num_cols = m.length;
    const num_rows = m[0].length;

    // Do not check for OOB inputs and exit early here, because the shape of
    // the output matrix may be determined by the operation being run,
    // i.e. transpose.

    const m_flat = flatten2DArray(m);
    const m_values = cartesianProduct(
      ...m_flat.map((e) => e.endpoints())
    );

    const outputs = new Set();
    m_values.forEach((inner_m) => {
      const unflat_m = unflatten2DArray(inner_m, num_cols, num_rows);
      outputs.add(this.roundAndFlushMatrixToMatrix(unflat_m, op));
    });

    const result = this.spanMatrices(...outputs);
    const result_cols = result.length;
    const result_rows = result[0].length;

    // FPMatrix has to be coerced to ROArrayArray<FPInterval> to use .every. This should
    // always be safe, since FPMatrix are defined as fixed length array of
    // arrays.
    return result.every((c) => c.every((r) => r.isFinite())) ?
    result :
    this.constants().unboundedMatrix[result_cols][result_rows];
  }

  /**
   * Calculate the Matrix of acceptance intervals by running a scalar operation
   * component-wise over a scalar and a matrix.
   *
   * An example of this is performing constant scaling.
   *
   * @param i scalar  input
   * @param m matrix input
   * @param op scalar operation to be run component-wise
   * @returns a matrix of intervals with the outputs of op.impl
   */
  runScalarPairToIntervalOpScalarMatrixComponentWise(
  i,
  m,
  op)
  {
    const cols = m.length;
    const rows = m[0].length;
    return this.toMatrix(
      unflatten2DArray(
        flatten2DArray(m).map((e) => this.runScalarPairToIntervalOp(i, e, op)),
        cols,
        rows
      )
    );
  }

  /**
   * Calculate the Matrix of acceptance intervals by running a scalar operation
   * component-wise over a pair of matrices.
   *
   * An example of this is performing matrix addition.
   *
   * @param x first input domain intervals matrix
   * @param y second input domain intervals matrix
   * @param op scalar operation to be run component-wise
   * @returns a matrix of intervals with the outputs of op.impl
   */
  runScalarPairToIntervalOpMatrixMatrixComponentWise(
  x,
  y,
  op)
  {
    assert(
      x.length === y.length && x[0].length === y[0].length,
      `runScalarPairToIntervalOpMatrixMatrixComponentWise requires matrices of the same dimensions`
    );

    const cols = x.length;
    const rows = x[0].length;
    const flat_x = flatten2DArray(x);
    const flat_y = flatten2DArray(y);

    return this.toMatrix(
      unflatten2DArray(
        flat_x.map((i, idx) => {
          return this.runScalarPairToIntervalOp(i, flat_y[idx], op);
        }),
        cols,
        rows
      )
    );
  }

  // API - Fundamental Error Intervals

  /** @returns a ScalarToIntervalOp for [n - error_range, n + error_range] */
  AbsoluteErrorIntervalOp(error_range) {
    const op = {
      impl: (_) => {
        return this.constants().unboundedInterval;
      }
    };

    assert(
      error_range >= 0,
      `absoluteErrorInterval must have non-negative error range, get ${error_range}`
    );

    if (this.isFinite(error_range)) {
      op.impl = (n) => {
        assert(!Number.isNaN(n), `absolute error not defined for NaN`);
        // Return anyInterval if given center n is infinity.
        if (!this.isFinite(n)) {
          return this.constants().unboundedInterval;
        }
        return this.toInterval([n - error_range, n + error_range]);
      };
    }

    return op;
  }

  absoluteErrorIntervalImpl(n, error_range) {
    error_range = Math.abs(error_range);
    return this.runScalarToIntervalOp(
      this.toInterval(n),
      this.AbsoluteErrorIntervalOp(error_range)
    );
  }

  /** @returns an interval of the absolute error around the point */


  /**
   * Defines a ScalarToIntervalOp for an interval of the correctly rounded values
   * around the point
   */
  CorrectlyRoundedIntervalOp = {
    impl: (n) => {
      assert(!Number.isNaN(n), `absolute not defined for NaN`);
      return this.toInterval(n);
    }
  };

  correctlyRoundedIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.CorrectlyRoundedIntervalOp);
  }

  /** @returns an interval of the correctly rounded values around the point */


  correctlyRoundedMatrixImpl(m) {
    return this.toMatrix(map2DArray(m, this.correctlyRoundedInterval));
  }

  /** @returns a matrix of correctly rounded intervals for the provided matrix */


  /** @returns a ScalarToIntervalOp for [n - numULP * ULP(n), n + numULP * ULP(n)] */
  ULPIntervalOp(numULP) {
    const op = {
      impl: (_) => {
        return this.constants().unboundedInterval;
      }
    };

    if (this.isFinite(numULP)) {
      op.impl = (n) => {
        assert(!Number.isNaN(n), `ULP error not defined for NaN`);

        const ulp = this.oneULP(n);
        const begin = n - numULP * ulp;
        const end = n + numULP * ulp;

        return this.toInterval([
        Math.min(begin, this.flushSubnormal(begin)),
        Math.max(end, this.flushSubnormal(end))]
        );
      };
    }

    return op;
  }

  ulpIntervalImpl(n, numULP) {
    numULP = Math.abs(numULP);
    return this.runScalarToIntervalOp(this.toInterval(n), this.ULPIntervalOp(numULP));
  }

  /** @returns an interval of N * ULP around the point */


  // API - Acceptance Intervals

  AbsIntervalOp = {
    impl: (n) => {
      return this.correctlyRoundedInterval(Math.abs(n));
    }
  };

  absIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.AbsIntervalOp);
  }

  /** Calculate an acceptance interval for abs(n) */


  // This op is implemented differently for f32 and f16.
  AcosIntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      // acos(n) = atan2(sqrt(1.0 - n * n), n) or a polynomial approximation with absolute error
      const y = this.sqrtInterval(this.subtractionInterval(1, this.multiplicationInterval(n, n)));
      const approx_abs_error = this.kind === 'f32' ? 6.77e-5 : 3.91e-3;
      return this.spanIntervals(
        this.atan2Interval(y, n),
        this.absoluteErrorInterval(Math.acos(n), approx_abs_error)
      );
    },
    domain: () => {
      return this.constants().negOneToOneInterval;
    }
  };

  acosIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.AcosIntervalOp);
  }

  /** Calculate an acceptance interval for acos(n) */


  AcoshAlternativeIntervalOp = {
    impl: (x) => {
      // acosh(x) = log(x + sqrt((x + 1.0f) * (x - 1.0)))
      const inner_value = this.multiplicationInterval(
        this.additionInterval(x, 1.0),
        this.subtractionInterval(x, 1.0)
      );
      const sqrt_value = this.sqrtInterval(inner_value);
      return this.logInterval(this.additionInterval(x, sqrt_value));
    }
  };

  acoshAlternativeIntervalImpl(x) {
    return this.runScalarToIntervalOp(this.toInterval(x), this.AcoshAlternativeIntervalOp);
  }

  /** Calculate an acceptance interval of acosh(x) using log(x + sqrt((x + 1.0f) * (x - 1.0))) */


  AcoshPrimaryIntervalOp = {
    impl: (x) => {
      // acosh(x) = log(x + sqrt(x * x - 1.0))
      const inner_value = this.subtractionInterval(this.multiplicationInterval(x, x), 1.0);
      const sqrt_value = this.sqrtInterval(inner_value);
      return this.logInterval(this.additionInterval(x, sqrt_value));
    }
  };

  acoshPrimaryIntervalImpl(x) {
    return this.runScalarToIntervalOp(this.toInterval(x), this.AcoshPrimaryIntervalOp);
  }

  /** Calculate an acceptance interval of acosh(x) using log(x + sqrt(x * x - 1.0)) */


  /** All acceptance interval functions for acosh(x) */


  AdditionIntervalOp = {
    impl: (x, y) => {
      return this.correctlyRoundedInterval(x + y);
    }
  };

  additionIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.AdditionIntervalOp
    );
  }

  /** Calculate an acceptance interval of x + y, when x and y are both scalars */





  additionMatrixMatrixIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOpMatrixMatrixComponentWise(
      this.toMatrix(x),
      this.toMatrix(y),
      this.AdditionIntervalOp
    );
  }

  /** Calculate an acceptance interval of x + y, when x and y are matrices */





  // This op is implemented differently for f32 and f16.
  AsinIntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      // asin(n) = atan2(n, sqrt(1.0 - n * n)) or a polynomial approximation with absolute error
      const x = this.sqrtInterval(this.subtractionInterval(1, this.multiplicationInterval(n, n)));
      const approx_abs_error = this.kind === 'f32' ? 6.81e-5 : 3.91e-3;
      return this.spanIntervals(
        this.atan2Interval(n, x),
        this.absoluteErrorInterval(Math.asin(n), approx_abs_error)
      );
    },
    domain: () => {
      return this.constants().negOneToOneInterval;
    }
  };

  /** Calculate an acceptance interval for asin(n) */
  asinIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.AsinIntervalOp);
  }

  /** Calculate an acceptance interval for asin(n) */


  AsinhIntervalOp = {
    impl: (x) => {
      // asinh(x) = log(x + sqrt(x * x + 1.0))
      const inner_value = this.additionInterval(this.multiplicationInterval(x, x), 1.0);
      const sqrt_value = this.sqrtInterval(inner_value);
      return this.logInterval(this.additionInterval(x, sqrt_value));
    }
  };

  asinhIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.AsinhIntervalOp);
  }

  /** Calculate an acceptance interval of asinh(x) */


  AtanIntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      const ulp_error = this.kind === 'f32' ? 4096 : 5;
      return this.ulpInterval(Math.atan(n), ulp_error);
    }
  };

  /** Calculate an acceptance interval of atan(x) */
  atanIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.AtanIntervalOp);
  }

  /** Calculate an acceptance interval of atan(x) */


  // This op is implemented differently for f32 and f16.
  Atan2IntervalOpBuilder() {
    assert(this.kind === 'f32' || this.kind === 'f16');
    const constants = this.constants();
    // For atan2, the params are labelled (y, x), not (x, y), so domain.x is first parameter (y),
    // and domain.y is the second parameter (x).
    // The first param must be finite and normal.
    const domain_x = [
    this.toInterval([constants.negative.min, constants.negative.max]),
    this.toInterval([constants.positive.min, constants.positive.max])];

    // inherited from division
    const domain_y =
    this.kind === 'f32' ?
    [this.toInterval([-(2 ** 126), -(2 ** -126)]), this.toInterval([2 ** -126, 2 ** 126])] :
    [this.toInterval([-(2 ** 14), -(2 ** -14)]), this.toInterval([2 ** -14, 2 ** 14])];
    const ulp_error = this.kind === 'f32' ? 4096 : 5;
    return {
      impl: (y, x) => {
        // Accurate result in f64
        let atan_yx = Math.atan(y / x);
        // Offset by +/-pi according to the definition. Use pi value in f64 because we are
        // handling accurate result.
        if (x < 0) {
          // x < 0, y > 0, result is atan(y/x) + π
          if (y > 0) {
            atan_yx = atan_yx + kValue.f64.positive.pi.whole;
          } else {
            // x < 0, y < 0, result is atan(y/x) - π
            atan_yx = atan_yx - kValue.f64.positive.pi.whole;
          }
        }

        return this.ulpInterval(atan_yx, ulp_error);
      },
      extrema: (y, x) => {
        // There is discontinuity, which generates an unbounded result, at y/x = 0 that will dominate the accuracy
        if (y.contains(0)) {
          if (x.contains(0)) {
            return [this.toInterval(0), this.toInterval(0)];
          }
          return [this.toInterval(0), x];
        }
        return [y, x];
      },
      domain: () => {
        return { x: domain_x, y: domain_y };
      }
    };
  }

  atan2IntervalImpl(y, x) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(y),
      this.toInterval(x),
      this.Atan2IntervalOpBuilder()
    );
  }

  /** Calculate an acceptance interval of atan2(y, x) */





  AtanhIntervalOp = {
    impl: (n) => {
      // atanh(x) = log((1.0 + x) / (1.0 - x)) * 0.5
      const numerator = this.additionInterval(1.0, n);
      const denominator = this.subtractionInterval(1.0, n);
      const log_interval = this.logInterval(this.divisionInterval(numerator, denominator));
      return this.multiplicationInterval(log_interval, 0.5);
    }
  };

  atanhIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.AtanhIntervalOp);
  }

  /** Calculate an acceptance interval of atanh(x) */


  CeilIntervalOp = {
    impl: (n) => {
      return this.correctlyRoundedInterval(Math.ceil(n));
    }
  };

  ceilIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.CeilIntervalOp);
  }

  /** Calculate an acceptance interval of ceil(x) */


  ClampMedianIntervalOp = {
    impl: (x, y, z) => {
      return this.correctlyRoundedInterval(
        // Default sort is string sort, so have to implement numeric comparison.
        // Cannot use the b-a one-liner, because that assumes no infinities.
        [x, y, z].sort((a, b) => {
          if (a < b) {
            return -1;
          }
          if (a > b) {
            return 1;
          }
          return 0;
        })[1]
      );
    }
  };

  clampMedianIntervalImpl(
  x,
  y,
  z)
  {
    return this.runScalarTripleToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.toInterval(z),
      this.ClampMedianIntervalOp
    );
  }

  /** Calculate an acceptance interval of clamp(x, y, z) via median(x, y, z) */






  ClampMinMaxIntervalOp = {
    impl: (x, low, high) => {
      return this.minInterval(this.maxInterval(x, low), high);
    }
  };

  clampMinMaxIntervalImpl(
  x,
  low,
  high)
  {
    return this.runScalarTripleToIntervalOp(
      this.toInterval(x),
      this.toInterval(low),
      this.toInterval(high),
      this.ClampMinMaxIntervalOp
    );
  }

  /** Calculate an acceptance interval of clamp(x, high, low) via min(max(x, low), high) */






  /** All acceptance interval functions for clamp(x, y, z) */


  CosIntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      const abs_error = this.kind === 'f32' ? 2 ** -11 : 2 ** -7;
      return this.absoluteErrorInterval(Math.cos(n), abs_error);
    },
    domain: () => {
      return this.constants().negPiToPiInterval;
    }
  };

  cosIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.CosIntervalOp);
  }

  /** Calculate an acceptance interval of cos(x) */


  CoshIntervalOp = {
    impl: (n) => {
      // cosh(x) = (exp(x) + exp(-x)) * 0.5
      const minus_n = this.negationInterval(n);
      return this.multiplicationInterval(
        this.additionInterval(this.expInterval(n), this.expInterval(minus_n)),
        0.5
      );
    }
  };

  coshIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.CoshIntervalOp);
  }

  /** Calculate an acceptance interval of cosh(x) */


  CrossIntervalOp = {
    impl: (x, y) => {
      assert(x.length === 3, `CrossIntervalOp received x with ${x.length} instead of 3`);
      assert(y.length === 3, `CrossIntervalOp received y with ${y.length} instead of 3`);

      // cross(x, y) = r, where
      //   r[0] = x[1] * y[2] - x[2] * y[1]
      //   r[1] = x[2] * y[0] - x[0] * y[2]
      //   r[2] = x[0] * y[1] - x[1] * y[0]

      const r0 = this.subtractionInterval(
        this.multiplicationInterval(x[1], y[2]),
        this.multiplicationInterval(x[2], y[1])
      );
      const r1 = this.subtractionInterval(
        this.multiplicationInterval(x[2], y[0]),
        this.multiplicationInterval(x[0], y[2])
      );
      const r2 = this.subtractionInterval(
        this.multiplicationInterval(x[0], y[1]),
        this.multiplicationInterval(x[1], y[0])
      );

      if (r0.isFinite() && r1.isFinite() && r2.isFinite()) {
        return [r0, r1, r2];
      }
      return this.constants().unboundedVector[3];
    }
  };

  crossIntervalImpl(x, y) {
    assert(x.length === 3, `Cross is only defined for vec3`);
    assert(y.length === 3, `Cross is only defined for vec3`);
    return this.runVectorPairToVectorOp(this.toVector(x), this.toVector(y), this.CrossIntervalOp);
  }

  /** Calculate a vector of acceptance intervals for cross(x, y) */


  DegreesIntervalOp = {
    impl: (n) => {
      return this.multiplicationInterval(n, 57.295779513082322865);
    }
  };

  degreesIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.DegreesIntervalOp);
  }

  /** Calculate an acceptance interval of degrees(x) */


  /**
   * Calculate the minor of a NxN matrix.
   *
   * The ijth minor of a square matrix, is the N-1xN-1 matrix created by removing
   * the ith column and jth row from the original matrix.
   */
  minorNxN(m, col, row) {
    const dim = m.length;
    assert(m.length === m[0].length, `minorMatrix is only defined for square matrices`);
    assert(col >= 0 && col < dim, `col ${col} needs be in [0, # of columns '${dim}')`);
    assert(row >= 0 && row < dim, `row ${row} needs be in [0, # of rows '${dim}')`);

    const result = [...Array(dim - 1)].map((_) => [...Array(dim - 1)]);

    const col_indices = [...Array(dim).keys()].filter((e) => e !== col);
    const row_indices = [...Array(dim).keys()].filter((e) => e !== row);

    col_indices.forEach((c, i) => {
      row_indices.forEach((r, j) => {
        result[i][j] = m[c][r];
      });
    });
    return result;
  }

  /** Calculate an acceptance interval for determinant(m), where m is a 2x2 matrix */
  determinant2x2Interval(m) {
    assert(
      m.length === m[0].length && m.length === 2,
      `determinant2x2Interval called on non-2x2 matrix`
    );
    return this.subtractionInterval(
      this.multiplicationInterval(m[0][0], m[1][1]),
      this.multiplicationInterval(m[0][1], m[1][0])
    );
  }

  /** Calculate an acceptance interval for determinant(m), where m is a 3x3 matrix */
  determinant3x3Interval(m) {
    assert(
      m.length === m[0].length && m.length === 3,
      `determinant3x3Interval called on non-3x3 matrix`
    );

    // M is a 3x3 matrix
    // det(M) is A + B + C, where A, B, C are three elements in a row/column times
    // their own co-factor.
    // (The co-factor is the determinant of the minor of that position with the
    // appropriate +/-)
    // For simplicity sake A, B, C are calculated as the elements of the first
    // column
    const A = this.multiplicationInterval(
      m[0][0],
      this.determinant2x2Interval(this.minorNxN(m, 0, 0))
    );
    const B = this.multiplicationInterval(
      -m[0][1],
      this.determinant2x2Interval(this.minorNxN(m, 0, 1))
    );
    const C = this.multiplicationInterval(
      m[0][2],
      this.determinant2x2Interval(this.minorNxN(m, 0, 2))
    );

    // Need to calculate permutations, since for fp addition is not associative,
    // so A + B + C is not guaranteed to equal B + C + A, etc.
    const permutations = calculatePermutations([A, B, C]);
    return this.spanIntervals(
      ...permutations.map((p) =>
      p.reduce((prev, cur) => this.additionInterval(prev, cur))
      )
    );
  }

  /** Calculate an acceptance interval for determinant(m), where m is a 4x4 matrix */
  determinant4x4Interval(m) {
    assert(
      m.length === m[0].length && m.length === 4,
      `determinant3x3Interval called on non-4x4 matrix`
    );

    // M is a 4x4 matrix
    // det(M) is A + B + C + D, where A, B, C, D are four elements in a row/column
    // times their own co-factor.
    // (The co-factor is the determinant of the minor of that position with the
    // appropriate +/-)
    // For simplicity sake A, B, C, D are calculated as the elements of the
    // first column
    const A = this.multiplicationInterval(
      m[0][0],
      this.determinant3x3Interval(this.minorNxN(m, 0, 0))
    );
    const B = this.multiplicationInterval(
      -m[0][1],
      this.determinant3x3Interval(this.minorNxN(m, 0, 1))
    );
    const C = this.multiplicationInterval(
      m[0][2],
      this.determinant3x3Interval(this.minorNxN(m, 0, 2))
    );
    const D = this.multiplicationInterval(
      -m[0][3],
      this.determinant3x3Interval(this.minorNxN(m, 0, 3))
    );

    // Need to calculate permutations, since for fp addition is not associative
    // so A + B + C + D is not guaranteed to equal B + C + A + D, etc.
    const permutations = calculatePermutations([A, B, C, D]);
    return this.spanIntervals(
      ...permutations.map((p) =>
      p.reduce((prev, cur) => this.additionInterval(prev, cur))
      )
    );
  }

  /**
   * This code calculates 3x3 and 4x4 determinants using the textbook co-factor
   * method, using the first column for the co-factor selection.
   *
   * For matrices composed of integer elements, e, with |e|^4 < 2**21, this
   * should be fine.
   *
   * For e, where e is subnormal or 4*(e^4) might not be precisely expressible as
   * a f32 values, this approach breaks down, because the rule of all co-factor
   * definitions of determinant being equal doesn't hold in these cases.
   *
   * The general solution for this is to calculate all the permutations of the
   * operations in the worked out formula for determinant.
   * For 3x3 this is tractable, but for 4x4 this works out to ~23! permutations
   * that need to be calculated.
   * Thus, CTS testing and the spec definition of accuracy is restricted to the
   * space that the simple implementation is valid.
   */
  determinantIntervalImpl(x) {
    const dim = x.length;
    assert(
      x[0].length === dim && (dim === 2 || dim === 3 || dim === 4),
      `determinantInterval only defined for 2x2, 3x3 and 4x4 matrices`
    );
    switch (dim) {
      case 2:
        return this.determinant2x2Interval(x);
      case 3:
        return this.determinant3x3Interval(x);
      case 4:
        return this.determinant4x4Interval(x);
    }
    unreachable(
      "determinantInterval called on x, where which has an unexpected dimension of '${dim}'"
    );
  }

  /** Calculate an acceptance interval for determinant(x) */


  DistanceIntervalScalarOp = {
    impl: (x, y) => {
      return this.lengthInterval(this.subtractionInterval(x, y));
    }
  };

  DistanceIntervalVectorOp = {
    impl: (x, y) => {
      return this.lengthInterval(
        this.runScalarPairToIntervalOpVectorComponentWise(
          this.toVector(x),
          this.toVector(y),
          this.SubtractionIntervalOp
        )
      );
    }
  };

  distanceIntervalImpl(
  x,
  y)
  {
    if (x instanceof Array && y instanceof Array) {
      assert(
        x.length === y.length,
        `distanceInterval requires both params to have the same number of elements`
      );
      return this.runVectorPairToIntervalOp(
        this.toVector(x),
        this.toVector(y),
        this.DistanceIntervalVectorOp
      );
    } else if (!(x instanceof Array) && !(y instanceof Array)) {
      return this.runScalarPairToIntervalOp(
        this.toInterval(x),
        this.toInterval(y),
        this.DistanceIntervalScalarOp
      );
    }
    unreachable(
      `distanceInterval requires both params to both the same type, either scalars or vectors`
    );
  }

  /** Calculate an acceptance interval of distance(x, y) */





  // This op is implemented differently for f32 and f16.
  DivisionIntervalOpBuilder() {
    const constants = this.constants();
    const domain_x = [this.toInterval([constants.negative.min, constants.positive.max])];
    const domain_y =
    this.kind === 'f32' || this.kind === 'abstract' ?
    [this.toInterval([-(2 ** 126), -(2 ** -126)]), this.toInterval([2 ** -126, 2 ** 126])] :
    [this.toInterval([-(2 ** 14), -(2 ** -14)]), this.toInterval([2 ** -14, 2 ** 14])];
    return {
      impl: (x, y) => {
        if (y === 0) {
          return constants.unboundedInterval;
        }
        return this.ulpInterval(x / y, 2.5);
      },
      extrema: (x, y) => {
        // division has a discontinuity at y = 0.
        if (y.contains(0)) {
          y = this.toInterval(0);
        }
        return [x, y];
      },
      domain: () => {
        return { x: domain_x, y: domain_y };
      }
    };
  }

  divisionIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.DivisionIntervalOpBuilder()
    );
  }

  /** Calculate an acceptance interval of x / y */





  DotIntervalOp = {
    impl: (x, y) => {
      // dot(x, y) = sum of x[i] * y[i]
      const multiplications = this.runScalarPairToIntervalOpVectorComponentWise(
        this.toVector(x),
        this.toVector(y),
        this.MultiplicationIntervalOp
      );

      // vec2 doesn't require permutations, since a + b = b + a for floats
      if (multiplications.length === 2) {
        return this.additionInterval(multiplications[0], multiplications[1]);
      }

      // The spec does not state the ordering of summation, so all the
      // permutations are calculated and their results spanned, since addition
      // of more than two floats is not transitive, i.e. a + b + c is not
      // guaranteed to equal b + a + c
      const permutations = calculatePermutations(multiplications);
      return this.spanIntervals(
        ...permutations.map((p) => p.reduce((prev, cur) => this.additionInterval(prev, cur)))
      );
    }
  };

  dotIntervalImpl(
  x,
  y)
  {
    assert(
      x.length === y.length,
      `dot not defined for vectors with different lengths, x = ${x}, y = ${y}`
    );
    return this.runVectorPairToIntervalOp(this.toVector(x), this.toVector(y), this.DotIntervalOp);
  }

  /** Calculated the acceptance interval for dot(x, y) */





  ExpIntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      const ulp_error = this.kind === 'f32' ? 3 + 2 * Math.abs(n) : 1 + 2 * Math.abs(n);
      return this.ulpInterval(Math.exp(n), ulp_error);
    }
  };

  expIntervalImpl(x) {
    return this.runScalarToIntervalOp(this.toInterval(x), this.ExpIntervalOp);
  }

  /** Calculate an acceptance interval for exp(x) */


  Exp2IntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      const ulp_error = this.kind === 'f32' ? 3 + 2 * Math.abs(n) : 1 + 2 * Math.abs(n);
      return this.ulpInterval(Math.pow(2, n), ulp_error);
    }
  };

  exp2IntervalImpl(x) {
    return this.runScalarToIntervalOp(this.toInterval(x), this.Exp2IntervalOp);
  }

  /** Calculate an acceptance interval for exp2(x) */


  /**
   * faceForward(x, y, z) = select(-x, x, dot(z, y) < 0.0)
   *
   * This builtin selects from two discrete results (delta rounding/flushing),
   * so the majority of the framework code is not appropriate, since the
   * framework attempts to span results.
   *
   * Thus, a bespoke implementation is used instead of
   * defining an Op and running that through the framework.
   */
  faceForwardIntervalsImpl(
  x,
  y,
  z)
  {
    const x_vec = this.toVector(x);
    // Running vector through this.runScalarToIntervalOpComponentWise to make
    // sure that flushing/rounding is handled, since toVector does not perform
    // those operations.
    const positive_x = this.runScalarToIntervalOpComponentWise(x_vec, {
      impl: (i) => {
        return this.toInterval(i);
      }
    });
    const negative_x = this.runScalarToIntervalOpComponentWise(x_vec, this.NegationIntervalOp);

    const dot_interval = this.dotInterval(z, y);

    const results = [];

    if (!dot_interval.isFinite()) {
      // dot calculation went out of bounds
      // Inserting undefined in the result, so that the test running framework
      // is aware of this potential OOB.
      // For const-eval tests, it means that the test case should be skipped,
      // since the shader will fail to compile.
      // For non-const-eval the undefined should be stripped out of the possible
      // results.

      results.push(undefined);
    }

    // Because the result of dot can be an interval, it might span across 0, thus
    // it is possible that both -x and x are valid responses.
    if (dot_interval.begin < 0 || dot_interval.end < 0) {
      results.push(positive_x);
    }

    if (dot_interval.begin >= 0 || dot_interval.end >= 0) {
      results.push(negative_x);
    }

    assert(
      results.length > 0 || results.every((r) => r === undefined),
      `faceForwardInterval selected neither positive x or negative x for the result, this shouldn't be possible`
    );
    return results;
  }

  /** Calculate the acceptance intervals for faceForward(x, y, z) */






  FloorIntervalOp = {
    impl: (n) => {
      return this.correctlyRoundedInterval(Math.floor(n));
    }
  };

  floorIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.FloorIntervalOp);
  }

  /** Calculate an acceptance interval of floor(x) */


  FmaIntervalOp = {
    impl: (x, y, z) => {
      return this.additionInterval(this.multiplicationInterval(x, y), z);
    }
  };

  fmaIntervalImpl(x, y, z) {
    return this.runScalarTripleToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.toInterval(z),
      this.FmaIntervalOp
    );
  }

  /** Calculate an acceptance interval for fma(x, y, z) */


  FractIntervalOp = {
    impl: (n) => {
      // fract(x) = x - floor(x) is defined in the spec.
      // For people coming from a non-graphics background this will cause some
      // unintuitive results. For example,
      // fract(-1.1) is not 0.1 or -0.1, but instead 0.9.
      // This is how other shading languages operate and allows for a desirable
      // wrap around in graphics programming.
      const result = this.subtractionInterval(n, this.floorInterval(n));
      assert(
        // negative.subnormal.min instead of 0, because FTZ can occur
        // selectively during the calculation
        this.toInterval([this.constants().negative.subnormal.min, 1.0]).contains(result),
        `fract(${n}) interval [${result}] unexpectedly extends beyond [~0.0, 1.0]`
      );
      if (result.contains(1)) {
        // Very small negative numbers can lead to catastrophic cancellation,
        // thus calculating a fract of 1.0, which is technically not a
        // fractional part, so some implementations clamp the result to next
        // nearest number.
        return this.spanIntervals(result, this.toInterval(this.constants().positive.less_than_one));
      }
      return result;
    }
  };

  fractIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.FractIntervalOp);
  }

  /** Calculate an acceptance interval of fract(x) */


  InverseSqrtIntervalOp = {
    impl: (n) => {
      return this.ulpInterval(1 / Math.sqrt(n), 2);
    },
    domain: () => {
      return this.constants().greaterThanZeroInterval;
    }
  };

  inverseSqrtIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.InverseSqrtIntervalOp);
  }

  /** Calculate an acceptance interval of inverseSqrt(x) */


  LdexpIntervalOp = {
    impl: (e1, e2) => {
      assert(Number.isInteger(e2), 'the second param of ldexp must be an integer');
      // Spec explicitly calls indeterminate value if e2 > bias + 1
      if (e2 > this.constants().bias + 1) {
        return this.constants().unboundedInterval;
      }
      // The spec says the result of ldexp(e1, e2) = e1 * 2 ^ e2, and the
      // accuracy is correctly rounded to the true value, so the inheritance
      // framework does not need to be invoked to determine endpoints.
      // Instead, the value at a higher precision is calculated and passed to
      // correctlyRoundedInterval.
      const result = e1 * 2 ** e2;
      if (!Number.isFinite(result)) {
        // Overflowed TS's number type, so definitely out of bounds
        return this.constants().unboundedInterval;
      }
      // The result may be zero if e2 + bias <= 0, but we can't simply span the interval to 0.0.
      // For example, for f32 input e1 = 2**120 and e2 = -130, e2 + bias = -3 <= 0, but
      // e1 * 2 ** e2 = 2**-10, so the valid result is 2**-10 or 0.0, instead of [0.0, 2**-10].
      // Always return the correctly-rounded interval, and special examination should be taken when
      // using the result.
      return this.correctlyRoundedInterval(result);
    }
  };

  ldexpIntervalImpl(e1, e2) {
    // Only round and flush e1, as e2 is of integer type (i32 or abstract integer) and should be
    // precise.
    return this.roundAndFlushScalarToInterval(e1, {
      impl: (e1) => this.LdexpIntervalOp.impl(e1, e2)
    });
  }

  /**
   * Calculate an acceptance interval of ldexp(e1, e2), where e2 is integer
   *
   * Spec indicate that the result may be zero if e2 + bias <= 0, no matter how large
   * was e1 * 2 ** e2, i.e. the actual valid result is correctlyRounded(e1 * 2 ** e2) or 0.0, if
   * e2 + bias <= 0. Such discontinious flush-to-zero behavior is hard to be expressed using
   * FPInterval, therefore in the situation of e2 + bias <= 0 the returned interval would be just
   * correctlyRounded(e1 * 2 ** e2), and special examination should be taken when using the result.
   *
   */


  LengthIntervalScalarOp = {
    impl: (n) => {
      return this.sqrtInterval(this.multiplicationInterval(n, n));
    }
  };

  LengthIntervalVectorOp = {
    impl: (n) => {
      return this.sqrtInterval(this.dotInterval(n, n));
    }
  };

  lengthIntervalImpl(n) {
    if (n instanceof Array) {
      return this.runVectorToIntervalOp(this.toVector(n), this.LengthIntervalVectorOp);
    } else {
      return this.runScalarToIntervalOp(this.toInterval(n), this.LengthIntervalScalarOp);
    }
  }

  /** Calculate an acceptance interval of length(x) */




  LogIntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      const abs_error = this.kind === 'f32' ? 2 ** -21 : 2 ** -7;
      if (n >= 0.5 && n <= 2.0) {
        return this.absoluteErrorInterval(Math.log(n), abs_error);
      }
      return this.ulpInterval(Math.log(n), 3);
    },
    domain: () => {
      return this.constants().greaterThanZeroInterval;
    }
  };

  logIntervalImpl(x) {
    return this.runScalarToIntervalOp(this.toInterval(x), this.LogIntervalOp);
  }

  /** Calculate an acceptance interval of log(x) */


  Log2IntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      const abs_error = this.kind === 'f32' ? 2 ** -21 : 2 ** -7;
      if (n >= 0.5 && n <= 2.0) {
        return this.absoluteErrorInterval(Math.log2(n), abs_error);
      }
      return this.ulpInterval(Math.log2(n), 3);
    },
    domain: () => {
      return this.constants().greaterThanZeroInterval;
    }
  };

  log2IntervalImpl(x) {
    return this.runScalarToIntervalOp(this.toInterval(x), this.Log2IntervalOp);
  }

  /** Calculate an acceptance interval of log2(x) */


  MaxIntervalOp = {
    impl: (x, y) => {
      // If both of the inputs are subnormal, then either of the inputs can be returned
      if (this.isSubnormal(x) && this.isSubnormal(y)) {
        return this.correctlyRoundedInterval(
          this.spanIntervals(this.toInterval(x), this.toInterval(y))
        );
      }

      return this.correctlyRoundedInterval(Math.max(x, y));
    }
  };

  maxIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.MaxIntervalOp
    );
  }

  /** Calculate an acceptance interval of max(x, y) */





  MinIntervalOp = {
    impl: (x, y) => {
      // If both of the inputs are subnormal, then either of the inputs can be returned
      if (this.isSubnormal(x) && this.isSubnormal(y)) {
        return this.correctlyRoundedInterval(
          this.spanIntervals(this.toInterval(x), this.toInterval(y))
        );
      }

      return this.correctlyRoundedInterval(Math.min(x, y));
    }
  };

  minIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.MinIntervalOp
    );
  }

  /** Calculate an acceptance interval of min(x, y) */





  MixImpreciseIntervalOp = {
    impl: (x, y, z) => {
      // x + (y - x) * z =
      //  x + t, where t = (y - x) * z
      const t = this.multiplicationInterval(this.subtractionInterval(y, x), z);
      return this.additionInterval(x, t);
    }
  };

  mixImpreciseIntervalImpl(x, y, z) {
    return this.runScalarTripleToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.toInterval(z),
      this.MixImpreciseIntervalOp
    );
  }

  /** Calculate an acceptance interval of mix(x, y, z) using x + (y - x) * z */


  MixPreciseIntervalOp = {
    impl: (x, y, z) => {
      // x * (1.0 - z) + y * z =
      //   t + s, where t = x * (1.0 - z), s = y * z
      const t = this.multiplicationInterval(x, this.subtractionInterval(1.0, z));
      const s = this.multiplicationInterval(y, z);
      return this.additionInterval(t, s);
    }
  };

  mixPreciseIntervalImpl(x, y, z) {
    return this.runScalarTripleToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.toInterval(z),
      this.MixPreciseIntervalOp
    );
  }

  /** Calculate an acceptance interval of mix(x, y, z) using x * (1.0 - z) + y * z */


  /** All acceptance interval functions for mix(x, y, z) */


  modfIntervalImpl(n) {
    const fract = this.correctlyRoundedInterval(n % 1.0);
    const whole = this.correctlyRoundedInterval(n - n % 1.0);
    return { fract, whole };
  }

  /** Calculate an acceptance interval of modf(x) */


  MultiplicationInnerOp = {
    impl: (x, y) => {
      return this.correctlyRoundedInterval(x * y);
    }
  };

  MultiplicationIntervalOp = {
    impl: (x, y) => {
      return this.roundAndFlushScalarPairToInterval(x, y, this.MultiplicationInnerOp);
    }
  };

  multiplicationIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.MultiplicationIntervalOp
    );
  }

  /** Calculate an acceptance interval of x * y */





  /**
   * @returns the vector result of multiplying the given vector by the given
   *          scalar
   */
  multiplyVectorByScalar(v, c) {
    return this.toVector(v.map((x) => this.multiplicationInterval(x, c)));
  }

  multiplicationMatrixScalarIntervalImpl(mat, scalar) {
    return this.runScalarPairToIntervalOpScalarMatrixComponentWise(
      this.toInterval(scalar),
      this.toMatrix(mat),
      this.MultiplicationIntervalOp
    );
  }

  /** Calculate an acceptance interval of x * y, when x is a matrix and y is a scalar */





  multiplicationScalarMatrixIntervalImpl(scalar, mat) {
    return this.multiplicationMatrixScalarInterval(mat, scalar);
  }

  /** Calculate an acceptance interval of x * y, when x is a scalar and y is a matrix */





  multiplicationMatrixMatrixIntervalImpl(
  mat_x,
  mat_y)
  {
    const x_cols = mat_x.length;
    const x_rows = mat_x[0].length;
    const y_cols = mat_y.length;
    const y_rows = mat_y[0].length;
    assert(x_cols === y_rows, `'mat${x_cols}x${x_rows} * mat${y_cols}x${y_rows}' is not defined`);

    const x_transposed = this.transposeInterval(mat_x);

    let oob_result = false;
    const result = [...Array(y_cols)].map((_) => [...Array(x_rows)]);
    mat_y.forEach((y, i) => {
      x_transposed.forEach((x, j) => {
        result[i][j] = this.dotInterval(x, y);
        if (!oob_result && !result[i][j].isFinite()) {
          oob_result = true;
        }
      });
    });

    if (oob_result) {
      return this.constants().unboundedMatrix[result.length][
      result[0].length];

    }
    return result;
  }

  /** Calculate an acceptance interval of x * y, when x is a matrix and y is a matrix */





  multiplicationMatrixVectorIntervalImpl(
  x,
  y)
  {
    const cols = x.length;
    const rows = x[0].length;
    assert(y.length === cols, `'mat${cols}x${rows} * vec${y.length}' is not defined`);

    return this.transposeInterval(x).map((e) => this.dotInterval(e, y));
  }

  /** Calculate an acceptance interval of x * y, when x is a matrix and y is a vector */





  multiplicationVectorMatrixIntervalImpl(
  x,
  y)
  {
    const cols = y.length;
    const rows = y[0].length;
    assert(x.length === rows, `'vec${x.length} * mat${cols}x${rows}' is not defined`);

    return y.map((e) => this.dotInterval(x, e));
  }

  /** Calculate an acceptance interval of x * y, when x is a vector and y is a matrix */





  NegationIntervalOp = {
    impl: (n) => {
      return this.correctlyRoundedInterval(-n);
    }
  };

  negationIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.NegationIntervalOp);
  }

  /** Calculate an acceptance interval of -x */


  NormalizeIntervalOp = {
    impl: (n) => {
      const length = this.lengthInterval(n);
      const result = this.toVector(n.map((e) => this.divisionInterval(e, length)));
      if (result.some((r) => !r.isFinite())) {
        return this.constants().unboundedVector[result.length];
      }
      return result;
    }
  };

  normalizeIntervalImpl(n) {
    return this.runVectorToVectorOp(this.toVector(n), this.NormalizeIntervalOp);
  }



  PowIntervalOp = {
    // pow(x, y) has no explicit domain restrictions, but inherits the x <= 0
    // domain restriction from log2(x). Invoking log2Interval(x) in impl will
    // enforce this, so there is no need to wrap the impl call here.
    impl: (x, y) => {
      return this.exp2Interval(this.multiplicationInterval(y, this.log2Interval(x)));
    }
  };

  powIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.PowIntervalOp
    );
  }

  /** Calculate an acceptance interval of pow(x, y) */





  RadiansIntervalOp = {
    impl: (n) => {
      return this.multiplicationInterval(n, 0.017453292519943295474);
    }
  };

  radiansIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.RadiansIntervalOp);
  }

  /** Calculate an acceptance interval of radians(x) */


  ReflectIntervalOp = {
    impl: (x, y) => {
      assert(
        x.length === y.length,
        `ReflectIntervalOp received x (${x}) and y (${y}) with different numbers of elements`
      );

      // reflect(x, y) = x - 2.0 * dot(x, y) * y
      //               = x - t * y, t = 2.0 * dot(x, y)
      // x = incident vector
      // y = normal of reflecting surface
      const t = this.multiplicationInterval(2.0, this.dotInterval(x, y));
      const rhs = this.multiplyVectorByScalar(y, t);
      const result = this.runScalarPairToIntervalOpVectorComponentWise(
        this.toVector(x),
        rhs,
        this.SubtractionIntervalOp
      );

      if (result.some((r) => !r.isFinite())) {
        return this.constants().unboundedVector[result.length];
      }
      return result;
    }
  };

  reflectIntervalImpl(x, y) {
    assert(
      x.length === y.length,
      `reflect is only defined for vectors with the same number of elements`
    );
    return this.runVectorPairToVectorOp(this.toVector(x), this.toVector(y), this.ReflectIntervalOp);
  }

  /** Calculate an acceptance interval of reflect(x, y) */





  /**
   * refract is a singular function in the sense that it is the only builtin that
   * takes in (FPVector, FPVector, F32/F16) and returns FPVector and is basically
   * defined in terms of other functions.
   *
   * Instead of implementing all the framework code to integrate it with its
   * own operation type, etc, it instead has a bespoke implementation that is a
   * composition of other builtin functions that use the framework.
   */
  refractIntervalImpl(i, s, r) {
    assert(
      i.length === s.length,
      `refract is only defined for vectors with the same number of elements`
    );

    const r_squared = this.multiplicationInterval(r, r);
    const dot = this.dotInterval(s, i);
    const dot_squared = this.multiplicationInterval(dot, dot);
    const one_minus_dot_squared = this.subtractionInterval(1, dot_squared);
    const k = this.subtractionInterval(
      1.0,
      this.multiplicationInterval(r_squared, one_minus_dot_squared)
    );

    if (!k.isFinite() || k.containsZeroOrSubnormals()) {
      // There is a discontinuity at k == 0, due to sqrt(k) being calculated, so exiting early
      return this.constants().unboundedVector[this.toVector(i).length];
    }

    if (k.end < 0.0) {
      // if k is negative, then the zero vector is the valid response
      return this.constants().zeroVector[this.toVector(i).length];
    }

    const dot_times_r = this.multiplicationInterval(dot, r);
    const k_sqrt = this.sqrtInterval(k);
    const t = this.additionInterval(dot_times_r, k_sqrt); // t = r * dot(i, s) + sqrt(k)

    const result = this.runScalarPairToIntervalOpVectorComponentWise(
      this.multiplyVectorByScalar(i, r),
      this.multiplyVectorByScalar(s, t),
      this.SubtractionIntervalOp
    ); // (i * r) - (s * t)

    if (result.some((r) => !r.isFinite())) {
      return this.constants().unboundedVector[result.length];
    }
    return result;
  }

  /** Calculate acceptance interval vectors of reflect(i, s, r) */






  RemainderIntervalOp = {
    impl: (x, y) => {
      // x % y = x - y * trunc(x/y)
      return this.subtractionInterval(
        x,
        this.multiplicationInterval(y, this.truncInterval(this.divisionInterval(x, y)))
      );
    }
  };

  /** Calculate an acceptance interval for x % y */
  remainderIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.RemainderIntervalOp
    );
  }

  /** Calculate an acceptance interval for x % y */


  RoundIntervalOp = {
    impl: (n) => {
      const k = Math.floor(n);
      const diff_before = n - k;
      const diff_after = k + 1 - n;
      if (diff_before < diff_after) {
        return this.correctlyRoundedInterval(k);
      } else if (diff_before > diff_after) {
        return this.correctlyRoundedInterval(k + 1);
      }

      // n is in the middle of two integers.
      // The tie breaking rule is 'k if k is even, k + 1 if k is odd'
      if (k % 2 === 0) {
        return this.correctlyRoundedInterval(k);
      }
      return this.correctlyRoundedInterval(k + 1);
    }
  };

  roundIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.RoundIntervalOp);
  }

  /** Calculate an acceptance interval of round(x) */


  /**
   * The definition of saturate does not specify which version of clamp to use.
   * Using min-max here, since it has wider acceptance intervals, that include
   * all of median's.
   */
  saturateIntervalImpl(n) {
    return this.runScalarTripleToIntervalOp(
      this.toInterval(n),
      this.toInterval(0.0),
      this.toInterval(1.0),
      this.ClampMinMaxIntervalOp
    );
  }

  /*** Calculate an acceptance interval of saturate(n) as clamp(n, 0.0, 1.0) */


  SignIntervalOp = {
    impl: (n) => {
      if (n > 0.0) {
        return this.correctlyRoundedInterval(1.0);
      }
      if (n < 0.0) {
        return this.correctlyRoundedInterval(-1.0);
      }

      return this.correctlyRoundedInterval(0.0);
    }
  };

  signIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.SignIntervalOp);
  }

  /** Calculate an acceptance interval of sign(x) */


  SinIntervalOp = {
    impl: (n) => {
      assert(this.kind === 'f32' || this.kind === 'f16');
      const abs_error = this.kind === 'f32' ? 2 ** -11 : 2 ** -7;
      return this.absoluteErrorInterval(Math.sin(n), abs_error);
    },
    domain: () => {
      return this.constants().negPiToPiInterval;
    }
  };

  sinIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.SinIntervalOp);
  }

  /** Calculate an acceptance interval of sin(x) */


  SinhIntervalOp = {
    impl: (n) => {
      // sinh(x) = (exp(x) - exp(-x)) * 0.5
      const minus_n = this.negationInterval(n);
      return this.multiplicationInterval(
        this.subtractionInterval(this.expInterval(n), this.expInterval(minus_n)),
        0.5
      );
    }
  };

  sinhIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.SinhIntervalOp);
  }

  /** Calculate an acceptance interval of sinh(x) */


  SmoothStepOp = {
    impl: (low, high, x) => {
      // For clamp(foo, 0.0, 1.0) the different implementations of clamp provide
      // the same value, so arbitrarily picking the minmax version to use.
      // t = clamp((x - low) / (high - low), 0.0, 1.0)

      const t = this.clampMedianInterval(
        this.divisionInterval(
          this.subtractionInterval(x, low),
          this.subtractionInterval(high, low)),
        0.0,
        1.0);
      // Inherited from t * t * (3.0 - 2.0 * t)

      return this.multiplicationInterval(
        t,
        this.multiplicationInterval(t,
        this.subtractionInterval(3.0,
        this.multiplicationInterval(2.0, t))));
    }
  };

  smoothStepIntervalImpl(low, high, x) {
    return this.runScalarTripleToIntervalOp(
      this.toInterval(low),
      this.toInterval(high),
      this.toInterval(x),
      this.SmoothStepOp
    );
  }

  /** Calculate an acceptance interval of smoothStep(low, high, x) */


  SqrtIntervalOp = {
    impl: (n) => {
      return this.divisionInterval(1.0, this.inverseSqrtInterval(n));
    }
  };

  sqrtIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.SqrtIntervalOp);
  }

  /** Calculate an acceptance interval of sqrt(x) */


  StepIntervalOp = {
    impl: (edge, x) => {
      if (edge <= x) {
        return this.correctlyRoundedInterval(1.0);
      }
      return this.correctlyRoundedInterval(0.0);
    }
  };

  stepIntervalImpl(edge, x) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(edge),
      this.toInterval(x),
      this.StepIntervalOp
    );
  }

  /**
   * Calculate an acceptance 'interval' for step(edge, x)
   *
   * step only returns two possible values, so its interval requires special
   * interpretation in CTS tests.
   * This interval will be one of four values: [0, 0], [0, 1], [1, 1] & [-∞, +∞].
   * [0, 0] and [1, 1] indicate that the correct answer in point they encapsulate.
   * [0, 1] should not be treated as a span, i.e. 0.1 is acceptable, but instead
   * indicate either 0.0 or 1.0 are acceptable answers.
   * [-∞, +∞] is treated as unbounded interval, since an unbounded or
   * infinite value was passed in.
   */


  SubtractionIntervalOp = {
    impl: (x, y) => {
      return this.correctlyRoundedInterval(x - y);
    }
  };

  subtractionIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOp(
      this.toInterval(x),
      this.toInterval(y),
      this.SubtractionIntervalOp
    );
  }

  /** Calculate an acceptance interval of x - y */





  subtractionMatrixMatrixIntervalImpl(x, y) {
    return this.runScalarPairToIntervalOpMatrixMatrixComponentWise(
      this.toMatrix(x),
      this.toMatrix(y),
      this.SubtractionIntervalOp
    );
  }

  /** Calculate an acceptance interval of x - y, when x and y are matrices */





  TanIntervalOp = {
    impl: (n) => {
      return this.divisionInterval(this.sinInterval(n), this.cosInterval(n));
    }
  };

  tanIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.TanIntervalOp);
  }

  /** Calculate an acceptance interval of tan(x) */


  TanhIntervalOp = {
    impl: (n) => {
      return this.divisionInterval(this.sinhInterval(n), this.coshInterval(n));
    }
  };

  tanhIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.TanhIntervalOp);
  }

  /** Calculate an acceptance interval of tanh(x) */


  TransposeIntervalOp = {
    impl: (m) => {
      const num_cols = m.length;
      const num_rows = m[0].length;
      const result = [...Array(num_rows)].map((_) => [...Array(num_cols)]);

      for (let i = 0; i < num_cols; i++) {
        for (let j = 0; j < num_rows; j++) {
          result[j][i] = this.correctlyRoundedInterval(m[i][j]);
        }
      }
      return this.toMatrix(result);
    }
  };

  transposeIntervalImpl(m) {
    return this.runMatrixToMatrixOp(this.toMatrix(m), this.TransposeIntervalOp);
  }

  /** Calculate an acceptance interval of transpose(m) */


  TruncIntervalOp = {
    impl: (n) => {
      return this.correctlyRoundedInterval(Math.trunc(n));
    }
  };

  truncIntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.TruncIntervalOp);
  }

  /** Calculate an acceptance interval of trunc(x) */

}

// Pre-defined values that get used multiple times in _constants' initializers. Cannot use FPTraits members, since this
// executes before they are defined.
const kF32UnboundedInterval = new FPInterval(
  'f32',
  Number.NEGATIVE_INFINITY,
  Number.POSITIVE_INFINITY
);
const kF32ZeroInterval = new FPInterval('f32', 0);

class F32Traits extends FPTraits {
  static _constants = {
    positive: {
      min: kValue.f32.positive.min,
      max: kValue.f32.positive.max,
      infinity: kValue.f32.positive.infinity,
      nearest_max: kValue.f32.positive.nearest_max,
      less_than_one: kValue.f32.positive.less_than_one,
      subnormal: {
        min: kValue.f32.positive.subnormal.min,
        max: kValue.f32.positive.subnormal.max
      },
      pi: {
        whole: kValue.f32.positive.pi.whole,
        three_quarters: kValue.f32.positive.pi.three_quarters,
        half: kValue.f32.positive.pi.half,
        third: kValue.f32.positive.pi.third,
        quarter: kValue.f32.positive.pi.quarter,
        sixth: kValue.f32.positive.pi.sixth
      },
      e: kValue.f32.positive.e
    },
    negative: {
      min: kValue.f32.negative.min,
      max: kValue.f32.negative.max,
      infinity: kValue.f32.negative.infinity,
      nearest_min: kValue.f32.negative.nearest_min,
      less_than_one: kValue.f32.negative.less_than_one,
      subnormal: {
        min: kValue.f32.negative.subnormal.min,
        max: kValue.f32.negative.subnormal.max
      },
      pi: {
        whole: kValue.f32.negative.pi.whole,
        three_quarters: kValue.f32.negative.pi.three_quarters,
        half: kValue.f32.negative.pi.half,
        third: kValue.f32.negative.pi.third,
        quarter: kValue.f32.negative.pi.quarter,
        sixth: kValue.f32.negative.pi.sixth
      }
    },
    bias: 127,
    unboundedInterval: kF32UnboundedInterval,
    zeroInterval: kF32ZeroInterval,
    // Have to use the constants.ts values here, because values defined in the
    // initializer cannot be referenced in the initializer
    negPiToPiInterval: new FPInterval(
      'f32',
      kValue.f32.negative.pi.whole,
      kValue.f32.positive.pi.whole
    ),
    greaterThanZeroInterval: new FPInterval(
      'f32',
      kValue.f32.positive.subnormal.min,
      kValue.f32.positive.max
    ),
    negOneToOneInterval: new FPInterval('f32', -1, 1),
    zeroVector: {
      2: [kF32ZeroInterval, kF32ZeroInterval],
      3: [kF32ZeroInterval, kF32ZeroInterval, kF32ZeroInterval],
      4: [kF32ZeroInterval, kF32ZeroInterval, kF32ZeroInterval, kF32ZeroInterval]
    },
    unboundedVector: {
      2: [kF32UnboundedInterval, kF32UnboundedInterval],
      3: [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval],
      4: [
      kF32UnboundedInterval,
      kF32UnboundedInterval,
      kF32UnboundedInterval,
      kF32UnboundedInterval]

    },
    unboundedMatrix: {
      2: {
        2: [
        [kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval]],

        3: [
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval]],

        4: [
        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval],

        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval]]


      },
      3: {
        2: [
        [kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval]],

        3: [
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval]],

        4: [
        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval],

        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval],

        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval]]


      },
      4: {
        2: [
        [kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval]],

        3: [
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval],
        [kF32UnboundedInterval, kF32UnboundedInterval, kF32UnboundedInterval]],

        4: [
        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval],

        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval],

        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval],

        [
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval,
        kF32UnboundedInterval]]


      }
    }
  };

  constructor() {
    super('f32');
  }

  constants() {
    return F32Traits._constants;
  }

  // Utilities - Overrides
  quantize = quantizeToF32;
  correctlyRounded = correctlyRoundedF32;
  isFinite = isFiniteF32;
  isSubnormal = isSubnormalNumberF32;
  flushSubnormal = flushSubnormalNumberF32;
  oneULP = oneULPF32;
  scalarBuilder = f32;
  scalarRange = scalarF32Range;
  sparseScalarRange = sparseScalarF32Range;
  vectorRange = vectorF32Range;
  sparseVectorRange = sparseVectorF32Range;
  sparseMatrixRange = sparseMatrixF32Range;

  // Framework - Fundamental Error Intervals - Overrides
  absoluteErrorInterval = this.absoluteErrorIntervalImpl.bind(this);
  correctlyRoundedInterval = this.correctlyRoundedIntervalImpl.bind(this);
  correctlyRoundedMatrix = this.correctlyRoundedMatrixImpl.bind(this);
  ulpInterval = this.ulpIntervalImpl.bind(this);

  // Framework - API - Overrides
  absInterval = this.absIntervalImpl.bind(this);
  acosInterval = this.acosIntervalImpl.bind(this);
  acoshAlternativeInterval = this.acoshAlternativeIntervalImpl.bind(this);
  acoshPrimaryInterval = this.acoshPrimaryIntervalImpl.bind(this);
  acoshIntervals = [this.acoshAlternativeInterval, this.acoshPrimaryInterval];
  additionInterval = this.additionIntervalImpl.bind(this);
  additionMatrixMatrixInterval = this.additionMatrixMatrixIntervalImpl.bind(this);
  asinInterval = this.asinIntervalImpl.bind(this);
  asinhInterval = this.asinhIntervalImpl.bind(this);
  atanInterval = this.atanIntervalImpl.bind(this);
  atan2Interval = this.atan2IntervalImpl.bind(this);
  atanhInterval = this.atanhIntervalImpl.bind(this);
  ceilInterval = this.ceilIntervalImpl.bind(this);
  clampMedianInterval = this.clampMedianIntervalImpl.bind(this);
  clampMinMaxInterval = this.clampMinMaxIntervalImpl.bind(this);
  clampIntervals = [this.clampMedianInterval, this.clampMinMaxInterval];
  cosInterval = this.cosIntervalImpl.bind(this);
  coshInterval = this.coshIntervalImpl.bind(this);
  crossInterval = this.crossIntervalImpl.bind(this);
  degreesInterval = this.degreesIntervalImpl.bind(this);
  determinantInterval = this.determinantIntervalImpl.bind(this);
  distanceInterval = this.distanceIntervalImpl.bind(this);
  divisionInterval = this.divisionIntervalImpl.bind(this);
  dotInterval = this.dotIntervalImpl.bind(this);
  expInterval = this.expIntervalImpl.bind(this);
  exp2Interval = this.exp2IntervalImpl.bind(this);
  faceForwardIntervals = this.faceForwardIntervalsImpl.bind(this);
  floorInterval = this.floorIntervalImpl.bind(this);
  fmaInterval = this.fmaIntervalImpl.bind(this);
  fractInterval = this.fractIntervalImpl.bind(this);
  inverseSqrtInterval = this.inverseSqrtIntervalImpl.bind(this);
  ldexpInterval = this.ldexpIntervalImpl.bind(this);
  lengthInterval = this.lengthIntervalImpl.bind(this);
  logInterval = this.logIntervalImpl.bind(this);
  log2Interval = this.log2IntervalImpl.bind(this);
  maxInterval = this.maxIntervalImpl.bind(this);
  minInterval = this.minIntervalImpl.bind(this);
  mixImpreciseInterval = this.mixImpreciseIntervalImpl.bind(this);
  mixPreciseInterval = this.mixPreciseIntervalImpl.bind(this);
  mixIntervals = [this.mixImpreciseInterval, this.mixPreciseInterval];
  modfInterval = this.modfIntervalImpl.bind(this);
  multiplicationInterval = this.multiplicationIntervalImpl.bind(this);
  multiplicationMatrixMatrixInterval =
  this.multiplicationMatrixMatrixIntervalImpl.bind(this);
  multiplicationMatrixScalarInterval =
  this.multiplicationMatrixScalarIntervalImpl.bind(this);
  multiplicationScalarMatrixInterval =
  this.multiplicationScalarMatrixIntervalImpl.bind(this);
  multiplicationMatrixVectorInterval =
  this.multiplicationMatrixVectorIntervalImpl.bind(this);
  multiplicationVectorMatrixInterval =
  this.multiplicationVectorMatrixIntervalImpl.bind(this);
  negationInterval = this.negationIntervalImpl.bind(this);
  normalizeInterval = this.normalizeIntervalImpl.bind(this);
  powInterval = this.powIntervalImpl.bind(this);
  radiansInterval = this.radiansIntervalImpl.bind(this);
  reflectInterval = this.reflectIntervalImpl.bind(this);
  refractInterval = this.refractIntervalImpl.bind(this);
  remainderInterval = this.remainderIntervalImpl.bind(this);
  roundInterval = this.roundIntervalImpl.bind(this);
  saturateInterval = this.saturateIntervalImpl.bind(this);
  signInterval = this.signIntervalImpl.bind(this);
  sinInterval = this.sinIntervalImpl.bind(this);
  sinhInterval = this.sinhIntervalImpl.bind(this);
  smoothStepInterval = this.smoothStepIntervalImpl.bind(this);
  sqrtInterval = this.sqrtIntervalImpl.bind(this);
  stepInterval = this.stepIntervalImpl.bind(this);
  subtractionInterval = this.subtractionIntervalImpl.bind(this);
  subtractionMatrixMatrixInterval =
  this.subtractionMatrixMatrixIntervalImpl.bind(this);
  tanInterval = this.tanIntervalImpl.bind(this);
  tanhInterval = this.tanhIntervalImpl.bind(this);
  transposeInterval = this.transposeIntervalImpl.bind(this);
  truncInterval = this.truncIntervalImpl.bind(this);

  // Framework - Cases

  // U32 -> Interval is used for testing f32 specific unpack* functions
  /**
   * @returns a Case for the param and the interval generator provided.
   * The Case will use an interval comparator for matching results.
   * @param param the param to pass in
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  makeU32ToVectorCase(
  param,
  filter,
  ...ops)
  {
    param = Math.trunc(param);

    const vectors = ops.map((o) => o(param));
    if (filter === 'finite' && vectors.some((v) => !v.every((e) => e.isFinite()))) {
      return undefined;
    }
    return {
      input: u32(param),
      expected: anyOf(...vectors)
    };
  }

  /**
   * @returns an array of Cases for operations over a range of inputs
   * @param params array of inputs to try
   * @param filter what interval filtering to apply
   * @param ops callbacks that implement generating an acceptance interval
   */
  generateU32ToIntervalCases(
  params,
  filter,
  ...ops)
  {
    return params.reduce((cases, e) => {
      const c = this.makeU32ToVectorCase(e, filter, ...ops);
      if (c !== undefined) {
        cases.push(c);
      }
      return cases;
    }, new Array());
  }

  // Framework - API

  QuantizeToF16IntervalOp = {
    impl: (n) => {
      const rounded = correctlyRoundedF16(n);
      const flushed = addFlushedIfNeededF16(rounded);
      return this.spanIntervals(...flushed.map((f) => this.toInterval(f)));
    }
  };

  quantizeToF16IntervalImpl(n) {
    return this.runScalarToIntervalOp(this.toInterval(n), this.QuantizeToF16IntervalOp);
  }

  /** Calculate an acceptance interval of quantizeToF16(x) */
  quantizeToF16Interval = this.quantizeToF16IntervalImpl.bind(this);

  /**
   * Once-allocated ArrayBuffer/views to avoid overhead of allocation when
   * converting between numeric formats
   *
   * unpackData* is shared between all the unpack*Interval functions, so to
   * avoid re-entrancy problems, they should not call each other or themselves
   * directly or indirectly.
   */
  unpackData = new ArrayBuffer(4);
  unpackDataU32 = new Uint32Array(this.unpackData);
  unpackDataU16 = new Uint16Array(this.unpackData);
  unpackDataU8 = new Uint8Array(this.unpackData);
  unpackDataI16 = new Int16Array(this.unpackData);
  unpackDataI8 = new Int8Array(this.unpackData);
  unpackDataF16 = new Float16Array(this.unpackData);

  unpack2x16floatIntervalImpl(n) {
    assert(
      n >= kValue.u32.min && n <= kValue.u32.max,
      'unpack2x16floatInterval only accepts valid u32 values'
    );
    this.unpackDataU32[0] = n;
    if (this.unpackDataF16.some((f) => !isFiniteF16(f))) {
      return [this.constants().unboundedInterval, this.constants().unboundedInterval];
    }

    const result = [
    this.quantizeToF16Interval(this.unpackDataF16[0]),
    this.quantizeToF16Interval(this.unpackDataF16[1])];


    if (result.some((r) => !r.isFinite())) {
      return [this.constants().unboundedInterval, this.constants().unboundedInterval];
    }
    return result;
  }

  /** Calculate an acceptance interval vector for unpack2x16float(x) */
  unpack2x16floatInterval = this.unpack2x16floatIntervalImpl.bind(this);

  unpack2x16snormIntervalImpl(n) {
    assert(
      n >= kValue.u32.min && n <= kValue.u32.max,
      'unpack2x16snormInterval only accepts valid u32 values'
    );
    const op = (n) => {
      return this.ulpInterval(Math.max(n / 32767, -1), 3);
    };

    this.unpackDataU32[0] = n;
    return [op(this.unpackDataI16[0]), op(this.unpackDataI16[1])];
  }

  /** Calculate an acceptance interval vector for unpack2x16snorm(x) */
  unpack2x16snormInterval = this.unpack2x16snormIntervalImpl.bind(this);

  unpack2x16unormIntervalImpl(n) {
    assert(
      n >= kValue.u32.min && n <= kValue.u32.max,
      'unpack2x16unormInterval only accepts valid u32 values'
    );
    const op = (n) => {
      return this.ulpInterval(n / 65535, 3);
    };

    this.unpackDataU32[0] = n;
    return [op(this.unpackDataU16[0]), op(this.unpackDataU16[1])];
  }

  /** Calculate an acceptance interval vector for unpack2x16unorm(x) */
  unpack2x16unormInterval = this.unpack2x16unormIntervalImpl.bind(this);

  unpack4x8snormIntervalImpl(n) {
    assert(
      n >= kValue.u32.min && n <= kValue.u32.max,
      'unpack4x8snormInterval only accepts valid u32 values'
    );
    const op = (n) => {
      return this.ulpInterval(Math.max(n / 127, -1), 3);
    };
    this.unpackDataU32[0] = n;
    return [
    op(this.unpackDataI8[0]),
    op(this.unpackDataI8[1]),
    op(this.unpackDataI8[2]),
    op(this.unpackDataI8[3])];

  }

  /** Calculate an acceptance interval vector for unpack4x8snorm(x) */
  unpack4x8snormInterval = this.unpack4x8snormIntervalImpl.bind(this);

  unpack4x8unormIntervalImpl(n) {
    assert(
      n >= kValue.u32.min && n <= kValue.u32.max,
      'unpack4x8unormInterval only accepts valid u32 values'
    );
    const op = (n) => {
      return this.ulpInterval(n / 255, 3);
    };

    this.unpackDataU32[0] = n;
    return [
    op(this.unpackDataU8[0]),
    op(this.unpackDataU8[1]),
    op(this.unpackDataU8[2]),
    op(this.unpackDataU8[3])];

  }

  /** Calculate an acceptance interval vector for unpack4x8unorm(x) */
  unpack4x8unormInterval = this.unpack4x8unormIntervalImpl.bind(this);
}

// Need to separately allocate f32 traits, so they can be referenced by
// FPAbstractTraits for forwarding.
const kF32Traits = new F32Traits();

// Pre-defined values that get used multiple times in _constants' initializers. Cannot use FPTraits members, since this
// executes before they are defined.
const kAbstractUnboundedInterval = new FPInterval(
  'abstract',
  Number.NEGATIVE_INFINITY,
  Number.POSITIVE_INFINITY
);
const kAbstractZeroInterval = new FPInterval('abstract', 0);

// This is implementation is incomplete
class FPAbstractTraits extends FPTraits {
  static _constants = {
    positive: {
      min: kValue.f64.positive.min,
      max: kValue.f64.positive.max,
      infinity: kValue.f64.positive.infinity,
      nearest_max: kValue.f64.positive.nearest_max,
      less_than_one: kValue.f64.positive.less_than_one,
      subnormal: {
        min: kValue.f64.positive.subnormal.min,
        max: kValue.f64.positive.subnormal.max
      },
      pi: {
        whole: kValue.f64.positive.pi.whole,
        three_quarters: kValue.f64.positive.pi.three_quarters,
        half: kValue.f64.positive.pi.half,
        third: kValue.f64.positive.pi.third,
        quarter: kValue.f64.positive.pi.quarter,
        sixth: kValue.f64.positive.pi.sixth
      },
      e: kValue.f64.positive.e
    },
    negative: {
      min: kValue.f64.negative.min,
      max: kValue.f64.negative.max,
      infinity: kValue.f64.negative.infinity,
      nearest_min: kValue.f64.negative.nearest_min,
      less_than_one: kValue.f64.negative.less_than_one,
      subnormal: {
        min: kValue.f64.negative.subnormal.min,
        max: kValue.f64.negative.subnormal.max
      },
      pi: {
        whole: kValue.f64.negative.pi.whole,
        three_quarters: kValue.f64.negative.pi.three_quarters,
        half: kValue.f64.negative.pi.half,
        third: kValue.f64.negative.pi.third,
        quarter: kValue.f64.negative.pi.quarter,
        sixth: kValue.f64.negative.pi.sixth
      }
    },
    bias: 1023,
    unboundedInterval: kAbstractUnboundedInterval,
    zeroInterval: kAbstractZeroInterval,
    // Have to use the constants.ts values here, because values defined in the
    // initializer cannot be referenced in the initializer
    negPiToPiInterval: new FPInterval(
      'abstract',
      kValue.f64.negative.pi.whole,
      kValue.f64.positive.pi.whole
    ),
    greaterThanZeroInterval: new FPInterval(
      'abstract',
      kValue.f64.positive.subnormal.min,
      kValue.f64.positive.max
    ),
    negOneToOneInterval: new FPInterval('abstract', -1, 1),

    zeroVector: {
      2: [kAbstractZeroInterval, kAbstractZeroInterval],
      3: [kAbstractZeroInterval, kAbstractZeroInterval, kAbstractZeroInterval],
      4: [
      kAbstractZeroInterval,
      kAbstractZeroInterval,
      kAbstractZeroInterval,
      kAbstractZeroInterval]

    },
    unboundedVector: {
      2: [kAbstractUnboundedInterval, kAbstractUnboundedInterval],
      3: [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval],
      4: [
      kAbstractUnboundedInterval,
      kAbstractUnboundedInterval,
      kAbstractUnboundedInterval,
      kAbstractUnboundedInterval]

    },
    unboundedMatrix: {
      2: {
        2: [
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval]],

        3: [
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval]],

        4: [
        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval],

        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval]]


      },
      3: {
        2: [
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval]],

        3: [
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval]],

        4: [
        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval],

        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval],

        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval]]


      },
      4: {
        2: [
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval]],

        3: [
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval],
        [kAbstractUnboundedInterval, kAbstractUnboundedInterval, kAbstractUnboundedInterval]],

        4: [
        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval],

        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval],

        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval],

        [
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval,
        kAbstractUnboundedInterval]]


      }
    }
  };

  constructor() {
    super('abstract');
  }

  constants() {
    return FPAbstractTraits._constants;
  }

  // Utilities - Overrides
  // number is represented as a f64 internally, so all number values are already
  // quantized to f64
  quantize = (n) => {
    return n;
  };
  correctlyRounded = correctlyRoundedF64;
  isFinite = Number.isFinite;
  isSubnormal = isSubnormalNumberF64;
  flushSubnormal = flushSubnormalNumberF64;
  oneULP = (_target, _mode = 'flush') => {
    unreachable(`'FPAbstractTraits.oneULP should never be called`);
  };
  scalarBuilder = abstractFloat;
  scalarRange = scalarF64Range;
  sparseScalarRange = sparseScalarF64Range;
  vectorRange = vectorF64Range;
  sparseVectorRange = sparseVectorF64Range;
  sparseMatrixRange = sparseMatrixF64Range;

  // Framework - Fundamental Error Intervals - Overrides
  absoluteErrorInterval = this.unimplementedAbsoluteErrorInterval.bind(this); // Should use FP.f32 instead
  correctlyRoundedInterval = this.correctlyRoundedIntervalImpl.bind(this);
  correctlyRoundedMatrix = this.correctlyRoundedMatrixImpl.bind(this);
  ulpInterval = this.unimplementedUlpInterval.bind(this); // Should use FP.f32 instead

  // Framework - API - Overrides
  absInterval = this.absIntervalImpl.bind(this);
  acosInterval = this.unimplementedScalarToInterval.bind(this, 'acosInterval');
  acoshAlternativeInterval = this.unimplementedScalarToInterval.bind(
    this,
    'acoshAlternativeInterval'
  );
  acoshPrimaryInterval = this.unimplementedScalarToInterval.bind(
    this,
    'acoshPrimaryInterval'
  );
  acoshIntervals = [this.acoshAlternativeInterval, this.acoshPrimaryInterval];
  additionInterval = this.unimplementedScalarPairToInterval.bind(
    this,
    'additionInterval'
  );
  additionMatrixMatrixInterval = this.unimplementedMatrixPairToMatrix.bind(
    this,
    'additionMatrixMatrixInterval'
  );
  asinInterval = this.unimplementedScalarToInterval.bind(this, 'asinInterval');
  asinhInterval = this.unimplementedScalarToInterval.bind(this, 'asinhInterval');
  atanInterval = this.unimplementedScalarToInterval.bind(this, 'atanInterval');
  atan2Interval = this.unimplementedScalarPairToInterval.bind(
    this,
    'atan2Interval'
  );
  atanhInterval = this.unimplementedScalarToInterval.bind(this, 'atanhInterval');
  ceilInterval = this.ceilIntervalImpl.bind(this);
  clampMedianInterval = this.clampMedianIntervalImpl.bind(this);
  clampMinMaxInterval = this.clampMinMaxIntervalImpl.bind(this);
  clampIntervals = [this.clampMedianInterval, this.clampMinMaxInterval];
  cosInterval = this.unimplementedScalarToInterval.bind(this, 'cosInterval');
  coshInterval = this.unimplementedScalarToInterval.bind(this, 'coshInterval');
  crossInterval = this.unimplementedVectorPairToVector.bind(this, 'crossInterval');
  degreesInterval = this.unimplementedScalarToInterval.bind(
    this,
    'degreesInterval'
  );
  determinantInterval = this.unimplementedMatrixToInterval.bind(
    this,
    'determinant'
  );
  distanceInterval = this.unimplementedDistance.bind(this);
  divisionInterval = this.unimplementedScalarPairToInterval.bind(
    this,
    'divisionInterval'
  );
  dotInterval = this.unimplementedVectorPairToInterval.bind(this, 'dotInterval');
  expInterval = this.unimplementedScalarToInterval.bind(this, 'expInterval');
  exp2Interval = this.unimplementedScalarToInterval.bind(this, 'exp2Interval');
  faceForwardIntervals = this.unimplementedFaceForward.bind(this);
  floorInterval = this.floorIntervalImpl.bind(this);
  fmaInterval = this.unimplementedScalarTripleToInterval.bind(this, 'fmaInterval');
  fractInterval = this.unimplementedScalarToInterval.bind(this, 'fractInterval');
  inverseSqrtInterval = this.unimplementedScalarToInterval.bind(
    this,
    'inverseSqrtInterval'
  );
  ldexpInterval = this.ldexpIntervalImpl.bind(this);
  lengthInterval = this.unimplementedLength.bind(this);
  logInterval = this.unimplementedScalarToInterval.bind(this, 'logInterval');
  log2Interval = this.unimplementedScalarToInterval.bind(this, 'log2Interval');
  maxInterval = this.maxIntervalImpl.bind(this);
  minInterval = this.minIntervalImpl.bind(this);
  mixImpreciseInterval = this.unimplementedScalarTripleToInterval.bind(
    this,
    'mixImpreciseInterval'
  );
  mixPreciseInterval = this.unimplementedScalarTripleToInterval.bind(
    this,
    'mixPreciseInterval'
  );
  mixIntervals = [this.mixImpreciseInterval, this.mixPreciseInterval];
  modfInterval = this.modfIntervalImpl.bind(this);
  multiplicationInterval = this.unimplementedScalarPairToInterval.bind(
    this,
    'multiplicationInterval'
  );
  multiplicationMatrixMatrixInterval = this.unimplementedMatrixPairToMatrix.bind(
    this,
    'multiplicationMatrixMatrixInterval'
  );
  multiplicationMatrixScalarInterval = this.unimplementedMatrixScalarToMatrix.bind(
    this,
    'multiplicationMatrixScalarInterval'
  );
  multiplicationScalarMatrixInterval = this.unimplementedScalarMatrixToMatrix.bind(
    this,
    'multiplicationScalarMatrixInterval'
  );
  multiplicationMatrixVectorInterval = this.unimplementedMatrixVectorToVector.bind(
    this,
    'multiplicationMatrixVectorInterval'
  );
  multiplicationVectorMatrixInterval = this.unimplementedVectorMatrixToVector.bind(
    this,
    'multiplicationVectorMatrixInterval'
  );
  negationInterval = this.negationIntervalImpl.bind(this);
  normalizeInterval = this.unimplementedVectorToVector.bind(
    this,
    'normalizeInterval'
  );
  powInterval = this.unimplementedScalarPairToInterval.bind(this, 'powInterval');
  radiansInterval = this.unimplementedScalarToInterval.bind(this, 'radiansImpl');
  reflectInterval = this.unimplementedVectorPairToVector.bind(
    this,
    'reflectInterval'
  );
  refractInterval = this.unimplementedRefract.bind(this);
  remainderInterval = this.unimplementedScalarPairToInterval.bind(
    this,
    'remainderInterval'
  );
  roundInterval = this.roundIntervalImpl.bind(this);
  saturateInterval = this.saturateIntervalImpl.bind(this);
  signInterval = this.signIntervalImpl.bind(this);
  sinInterval = this.unimplementedScalarToInterval.bind(this, 'sinInterval');
  sinhInterval = this.unimplementedScalarToInterval.bind(this, 'sinhInterval');
  smoothStepInterval = this.unimplementedScalarTripleToInterval.bind(
    this,
    'smoothStepInterval'
  );
  sqrtInterval = this.unimplementedScalarToInterval.bind(this, 'sqrtInterval');
  stepInterval = this.stepIntervalImpl.bind(this);
  subtractionInterval = this.unimplementedScalarPairToInterval.bind(
    this,
    'subtractionInterval'
  );
  subtractionMatrixMatrixInterval = this.unimplementedMatrixPairToMatrix.bind(
    this,
    'subtractionMatrixMatrixInterval'
  );
  tanInterval = this.unimplementedScalarToInterval.bind(this, 'tanInterval');
  tanhInterval = this.unimplementedScalarToInterval.bind(this, 'tanhInterval');
  transposeInterval = this.transposeIntervalImpl.bind(this);
  truncInterval = this.truncIntervalImpl.bind(this);
}

// Pre-defined values that get used multiple times in _constants' initializers. Cannot use FPTraits members, since this
// executes before they are defined.
const kF16UnboundedInterval = new FPInterval(
  'f16',
  Number.NEGATIVE_INFINITY,
  Number.POSITIVE_INFINITY
);
const kF16ZeroInterval = new FPInterval('f16', 0);

// This is implementation is incomplete
class F16Traits extends FPTraits {
  static _constants = {
    positive: {
      min: kValue.f16.positive.min,
      max: kValue.f16.positive.max,
      infinity: kValue.f16.positive.infinity,
      nearest_max: kValue.f16.positive.nearest_max,
      less_than_one: kValue.f16.positive.less_than_one,
      subnormal: {
        min: kValue.f16.positive.subnormal.min,
        max: kValue.f16.positive.subnormal.max
      },
      pi: {
        whole: kValue.f16.positive.pi.whole,
        three_quarters: kValue.f16.positive.pi.three_quarters,
        half: kValue.f16.positive.pi.half,
        third: kValue.f16.positive.pi.third,
        quarter: kValue.f16.positive.pi.quarter,
        sixth: kValue.f16.positive.pi.sixth
      },
      e: kValue.f16.positive.e
    },
    negative: {
      min: kValue.f16.negative.min,
      max: kValue.f16.negative.max,
      infinity: kValue.f16.negative.infinity,
      nearest_min: kValue.f16.negative.nearest_min,
      less_than_one: kValue.f16.negative.less_than_one,
      subnormal: {
        min: kValue.f16.negative.subnormal.min,
        max: kValue.f16.negative.subnormal.max
      },
      pi: {
        whole: kValue.f16.negative.pi.whole,
        three_quarters: kValue.f16.negative.pi.three_quarters,
        half: kValue.f16.negative.pi.half,
        third: kValue.f16.negative.pi.third,
        quarter: kValue.f16.negative.pi.quarter,
        sixth: kValue.f16.negative.pi.sixth
      }
    },
    bias: 15,
    unboundedInterval: kF16UnboundedInterval,
    zeroInterval: kF16ZeroInterval,
    // Have to use the constants.ts values here, because values defined in the
    // initializer cannot be referenced in the initializer
    negPiToPiInterval: new FPInterval(
      'f16',
      kValue.f16.negative.pi.whole,
      kValue.f16.positive.pi.whole
    ),
    greaterThanZeroInterval: new FPInterval(
      'f16',
      kValue.f16.positive.subnormal.min,
      kValue.f16.positive.max
    ),
    negOneToOneInterval: new FPInterval('f16', -1, 1),

    zeroVector: {
      2: [kF16ZeroInterval, kF16ZeroInterval],
      3: [kF16ZeroInterval, kF16ZeroInterval, kF16ZeroInterval],
      4: [kF16ZeroInterval, kF16ZeroInterval, kF16ZeroInterval, kF16ZeroInterval]
    },
    unboundedVector: {
      2: [kF16UnboundedInterval, kF16UnboundedInterval],
      3: [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval],
      4: [
      kF16UnboundedInterval,
      kF16UnboundedInterval,
      kF16UnboundedInterval,
      kF16UnboundedInterval]

    },
    unboundedMatrix: {
      2: {
        2: [
        [kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval]],

        3: [
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval]],

        4: [
        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval],

        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval]]


      },
      3: {
        2: [
        [kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval]],

        3: [
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval]],

        4: [
        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval],

        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval],

        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval]]


      },
      4: {
        2: [
        [kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval]],

        3: [
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval],
        [kF16UnboundedInterval, kF16UnboundedInterval, kF16UnboundedInterval]],

        4: [
        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval],

        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval],

        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval],

        [
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval,
        kF16UnboundedInterval]]


      }
    }
  };

  constructor() {
    super('f16');
  }

  constants() {
    return F16Traits._constants;
  }

  // Utilities - Overrides
  quantize = quantizeToF16;
  correctlyRounded = correctlyRoundedF16;
  isFinite = isFiniteF16;
  isSubnormal = isSubnormalNumberF16;
  flushSubnormal = flushSubnormalNumberF16;
  oneULP = oneULPF16;
  scalarBuilder = f16;
  scalarRange = scalarF16Range;
  sparseScalarRange = sparseScalarF16Range;
  vectorRange = vectorF16Range;
  sparseVectorRange = sparseVectorF16Range;
  sparseMatrixRange = sparseMatrixF16Range;

  // Framework - Fundamental Error Intervals - Overrides
  absoluteErrorInterval = this.absoluteErrorIntervalImpl.bind(this);
  correctlyRoundedInterval = this.correctlyRoundedIntervalImpl.bind(this);
  correctlyRoundedMatrix = this.correctlyRoundedMatrixImpl.bind(this);
  ulpInterval = this.ulpIntervalImpl.bind(this);

  // Framework - API - Overrides
  absInterval = this.absIntervalImpl.bind(this);
  acosInterval = this.acosIntervalImpl.bind(this);
  acoshAlternativeInterval = this.acoshAlternativeIntervalImpl.bind(this);
  acoshPrimaryInterval = this.acoshPrimaryIntervalImpl.bind(this);
  acoshIntervals = [this.acoshAlternativeInterval, this.acoshPrimaryInterval];
  additionInterval = this.additionIntervalImpl.bind(this);
  additionMatrixMatrixInterval = this.additionMatrixMatrixIntervalImpl.bind(this);
  asinInterval = this.asinIntervalImpl.bind(this);
  asinhInterval = this.asinhIntervalImpl.bind(this);
  atanInterval = this.atanIntervalImpl.bind(this);
  atan2Interval = this.atan2IntervalImpl.bind(this);
  atanhInterval = this.atanhIntervalImpl.bind(this);
  ceilInterval = this.ceilIntervalImpl.bind(this);
  clampMedianInterval = this.clampMedianIntervalImpl.bind(this);
  clampMinMaxInterval = this.clampMinMaxIntervalImpl.bind(this);
  clampIntervals = [this.clampMedianInterval, this.clampMinMaxInterval];
  cosInterval = this.cosIntervalImpl.bind(this);
  coshInterval = this.coshIntervalImpl.bind(this);
  crossInterval = this.crossIntervalImpl.bind(this);
  degreesInterval = this.degreesIntervalImpl.bind(this);
  determinantInterval = this.determinantIntervalImpl.bind(this);
  distanceInterval = this.distanceIntervalImpl.bind(this);
  divisionInterval = this.divisionIntervalImpl.bind(this);
  dotInterval = this.dotIntervalImpl.bind(this);
  expInterval = this.expIntervalImpl.bind(this);
  exp2Interval = this.exp2IntervalImpl.bind(this);
  faceForwardIntervals = this.faceForwardIntervalsImpl.bind(this);
  floorInterval = this.floorIntervalImpl.bind(this);
  fmaInterval = this.fmaIntervalImpl.bind(this);
  fractInterval = this.fractIntervalImpl.bind(this);
  inverseSqrtInterval = this.inverseSqrtIntervalImpl.bind(this);
  ldexpInterval = this.ldexpIntervalImpl.bind(this);
  lengthInterval = this.lengthIntervalImpl.bind(this);
  logInterval = this.logIntervalImpl.bind(this);
  log2Interval = this.log2IntervalImpl.bind(this);
  maxInterval = this.maxIntervalImpl.bind(this);
  minInterval = this.minIntervalImpl.bind(this);
  mixImpreciseInterval = this.mixImpreciseIntervalImpl.bind(this);
  mixPreciseInterval = this.mixPreciseIntervalImpl.bind(this);
  mixIntervals = [this.mixImpreciseInterval, this.mixPreciseInterval];
  modfInterval = this.modfIntervalImpl.bind(this);
  multiplicationInterval = this.multiplicationIntervalImpl.bind(this);
  multiplicationMatrixMatrixInterval =
  this.multiplicationMatrixMatrixIntervalImpl.bind(this);
  multiplicationMatrixScalarInterval =
  this.multiplicationMatrixScalarIntervalImpl.bind(this);
  multiplicationScalarMatrixInterval =
  this.multiplicationScalarMatrixIntervalImpl.bind(this);
  multiplicationMatrixVectorInterval =
  this.multiplicationMatrixVectorIntervalImpl.bind(this);
  multiplicationVectorMatrixInterval =
  this.multiplicationVectorMatrixIntervalImpl.bind(this);
  negationInterval = this.negationIntervalImpl.bind(this);
  normalizeInterval = this.normalizeIntervalImpl.bind(this);
  powInterval = this.powIntervalImpl.bind(this);
  radiansInterval = this.radiansIntervalImpl.bind(this);
  reflectInterval = this.reflectIntervalImpl.bind(this);
  refractInterval = this.refractIntervalImpl.bind(this);
  remainderInterval = this.remainderIntervalImpl.bind(this);
  roundInterval = this.roundIntervalImpl.bind(this);
  saturateInterval = this.saturateIntervalImpl.bind(this);
  signInterval = this.signIntervalImpl.bind(this);
  sinInterval = this.sinIntervalImpl.bind(this);
  sinhInterval = this.sinhIntervalImpl.bind(this);
  smoothStepInterval = this.smoothStepIntervalImpl.bind(this);
  sqrtInterval = this.sqrtIntervalImpl.bind(this);
  stepInterval = this.stepIntervalImpl.bind(this);
  subtractionInterval = this.subtractionIntervalImpl.bind(this);
  subtractionMatrixMatrixInterval =
  this.subtractionMatrixMatrixIntervalImpl.bind(this);
  tanInterval = this.tanIntervalImpl.bind(this);
  tanhInterval = this.tanhIntervalImpl.bind(this);
  transposeInterval = this.transposeIntervalImpl.bind(this);
  truncInterval = this.truncIntervalImpl.bind(this);
}

export const FP = {
  f32: kF32Traits,
  f16: new F16Traits(),
  abstract: new FPAbstractTraits()
};

/** @returns the floating-point traits for `type` */
export function fpTraitsFor(type) {
  switch (type.kind) {
    case 'abstract-float':
      return FP.abstract;
    case 'f32':
      return FP.f32;
    case 'f16':
      return FP.f16;
    default:
      unreachable(`unsupported type: ${type}`);
  }
}

/** @returns true if the value `value` is representable with `type` */
export function isRepresentable(value, type) {
  if (!Number.isFinite(value)) {
    return false;
  }
  if (isFloatType(type)) {
    const constants = fpTraitsFor(type).constants();
    return value >= constants.negative.min && value <= constants.positive.max;
  }

  assert(false, `isRepresentable() is not yet implemented for type ${type}`);
}
/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { globalTestConfig } from '../../../../common/framework/test_config.js';
import { assert, objectEquals, unreachable } from '../../../../common/util/util.js';
import { compare } from '../../../util/compare.js';
import {
  ScalarType,
  Scalar,
  TypeVec,
  TypeU32,
  Vector,
  VectorType,
  u32,
  i32,
  Matrix,
  MatrixType,
  scalarTypeOf,
} from '../../../util/conversion.js';
import { FPInterval } from '../../../util/floating_point.js';
import { cartesianProduct, quantizeToI32, quantizeToU32 } from '../../../util/math.js';

/** @returns if this Expectation actually a Comparator */
export function isComparator(e) {
  return !(
    e instanceof FPInterval ||
    e instanceof Scalar ||
    e instanceof Vector ||
    e instanceof Matrix ||
    e instanceof Array
  );
}

/** @returns the input if it is already a Comparator, otherwise wraps it in a 'value' comparator */
export function toComparator(input) {
  if (isComparator(input)) {
    return input;
  }

  return { compare: got => compare(got, input), kind: 'value' };
}

/** Case is a single expression test case. */

// Read-write storage buffer

/** All possible input sources */
export const allInputSources = ['const', 'uniform', 'storage_r', 'storage_rw'];

/** Configuration for running a expression test */

// Helper for returning the stride for a given Type
function valueStride(ty) {
  if (ty instanceof MatrixType) {
    switch (ty.cols) {
      case 2:
        switch (ty.rows) {
          case 2:
            return 16;
          case 3:
            return 32;
          case 4:
            return 32;
        }

        break;
      case 3:
        switch (ty.rows) {
          case 2:
            return 32;
          case 3:
            return 64;
          case 4:
            return 64;
        }

        break;
      case 4:
        switch (ty.rows) {
          case 2:
            return 32;
          case 3:
            return 64;
          case 4:
            return 64;
        }

        break;
    }

    unreachable(
      `Attempted to get stride length for a matrix with dimensions (${ty.cols}x${ty.rows}), which isn't currently handled`
    );
  }

  // Handles scalars and vectors
  return 16;
}

// Helper for summing up all of the stride values for an array of Types
function valueStrides(tys) {
  return tys.map(valueStride).reduce((sum, c) => sum + c);
}

// Helper for returning the WGSL storage type for the given Type.
function storageType(ty) {
  if (ty instanceof ScalarType) {
    if (ty.kind === 'bool') {
      return TypeU32;
    }
  }
  if (ty instanceof VectorType) {
    return TypeVec(ty.width, storageType(ty.elementType));
  }
  return ty;
}

// Helper for converting a value of the type 'ty' from the storage type.
function fromStorage(ty, expr) {
  if (ty instanceof ScalarType) {
    assert(ty.kind !== 'abstract-float', `No storage type defined for AbstractFloat values`);
    if (ty.kind === 'bool') {
      return `${expr} != 0u`;
    }
  }
  if (ty instanceof VectorType) {
    assert(
      ty.elementType.kind !== 'abstract-float',
      `No storage type defined for AbstractFloat values`
    );

    if (ty.elementType.kind === 'bool') {
      return `${expr} != vec${ty.width}<u32>(0u)`;
    }
  }
  return expr;
}

// Helper for converting a value of the type 'ty' to the storage type.
function toStorage(ty, expr) {
  if (ty instanceof ScalarType) {
    assert(ty.kind !== 'abstract-float', `No storage type defined for AbstractFloat values`);
    if (ty.kind === 'bool') {
      return `select(0u, 1u, ${expr})`;
    }
  }
  if (ty instanceof VectorType) {
    assert(
      ty.elementType.kind !== 'abstract-float',
      `No storage type defined for AbstractFloat values`
    );

    if (ty.elementType.kind === 'bool') {
      return `select(vec${ty.width}<u32>(0u), vec${ty.width}<u32>(1u), ${expr})`;
    }
  }
  return expr;
}

// A Pipeline is a map of WGSL shader source to a built pipeline

/**
 * Searches for an entry with the given key, adding and returning the result of calling
 * @p create if the entry was not found.
 * @param map the cache map
 * @param key the entry's key
 * @param create the function used to construct a value, if not found in the cache
 * @returns the value, either fetched from the cache, or newly built.
 */
function getOrCreate(map, key, create) {
  const existing = map.get(key);
  if (existing !== undefined) {
    return existing;
  }
  const value = create();
  map.set(key, value);
  return value;
}

/**
 * Runs the list of expression tests, possibly splitting the tests into multiple
 * dispatches to keep the input data within the buffer binding limits.
 * run() will pack the scalar test cases into smaller set of vectorized tests
 * if `cfg.vectorize` is defined.
 * @param t the GPUTest
 * @param shaderBuilder the shader builder function
 * @param parameterTypes the list of expression parameter types
 * @param resultType the return type for the expression overload
 * @param cfg test configuration values
 * @param cases list of test cases
 */
export async function run(
  t,
  shaderBuilder,
  parameterTypes,
  resultType,
  cfg = { inputSource: 'storage_r' },
  cases
) {
  // If the 'vectorize' config option was provided, pack the cases into vectors.
  if (cfg.vectorize !== undefined) {
    const packed = packScalarsToVector(parameterTypes, resultType, cases, cfg.vectorize);
    cases = packed.cases;
    parameterTypes = packed.parameterTypes;
    resultType = packed.resultType;
  }

  // The size of the input buffer may exceed the maximum buffer binding size,
  // so chunk the tests up into batches that fit into the limits. We also split
  // the cases into smaller batches to help with shader compilation performance.
  const casesPerBatch = (function () {
    switch (cfg.inputSource) {
      case 'const':
        // Some drivers are slow to optimize shaders with many constant values,
        // or statements. 32 is an empirically picked number of cases that works
        // well for most drivers.
        return 32;
      case 'uniform':
        // Some drivers are slow to build pipelines with large uniform buffers.
        // 2k appears to be a sweet-spot when benchmarking.
        return Math.floor(
          Math.min(1024 * 2, t.device.limits.maxUniformBufferBindingSize) /
            valueStrides(parameterTypes)
        );

      case 'storage_r':
      case 'storage_rw':
        return Math.floor(
          t.device.limits.maxStorageBufferBindingSize / valueStrides(parameterTypes)
        );
    }
  })();

  // A cache to hold built shader pipelines.
  const pipelineCache = new Map();

  // Submit all the cases in batches, rate-limiting to ensure not too many
  // batches are in flight simultaneously.
  const maxBatchesInFlight = 5;
  let batchesInFlight = 0;
  let resolvePromiseBlockingBatch = undefined;
  const batchFinishedCallback = () => {
    batchesInFlight -= 1;
    // If there is any batch waiting on a previous batch to finish,
    // unblock it now, and clear the resolve callback.
    if (resolvePromiseBlockingBatch) {
      resolvePromiseBlockingBatch();
      resolvePromiseBlockingBatch = undefined;
    }
  };

  for (let i = 0; i < cases.length; i += casesPerBatch) {
    const batchCases = cases.slice(i, Math.min(i + casesPerBatch, cases.length));

    if (batchesInFlight > maxBatchesInFlight) {
      await new Promise(resolve => {
        // There should only be one batch waiting at a time.
        assert(resolvePromiseBlockingBatch === undefined);
        resolvePromiseBlockingBatch = resolve;
      });
    }
    batchesInFlight += 1;

    const checkBatch = submitBatch(
      t,
      shaderBuilder,
      parameterTypes,
      resultType,
      batchCases,
      cfg.inputSource,
      pipelineCache
    );

    checkBatch();
    t.queue.onSubmittedWorkDone().finally(batchFinishedCallback);
  }
}

/**
 * Submits the list of expression tests. The input data must fit within the
 * buffer binding limits of the given inputSource.
 * @param t the GPUTest
 * @param shaderBuilder the shader builder function
 * @param parameterTypes the list of expression parameter types
 * @param resultType the return type for the expression overload
 * @param cases list of test cases that fit within the binding limits of the device
 * @param inputSource the source of the input values
 * @param pipelineCache the cache of compute pipelines, shared between batches
 * @returns a function that checks the results are as expected
 */
function submitBatch(
  t,
  shaderBuilder,
  parameterTypes,
  resultType,
  cases,
  inputSource,
  pipelineCache
) {
  // Construct a buffer to hold the results of the expression tests
  const outputBufferSize = cases.length * valueStride(resultType);
  const outputBuffer = t.device.createBuffer({
    size: outputBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE,
  });

  const [pipeline, group] = buildPipeline(
    t,
    shaderBuilder,
    parameterTypes,
    resultType,
    cases,
    inputSource,
    outputBuffer,
    pipelineCache
  );

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, group);
  pass.dispatchWorkgroups(1);
  pass.end();

  // Heartbeat to ensure CTS runners know we're alive.
  globalTestConfig.testHeartbeatCallback();

  t.queue.submit([encoder.finish()]);

  // Return a function that can check the results of the shader
  return () => {
    const checkExpectation = outputData => {
      // Read the outputs from the output buffer
      const outputs = new Array(cases.length);
      for (let i = 0; i < cases.length; i++) {
        outputs[i] = resultType.read(outputData, i * valueStride(resultType));
      }

      // The list of expectation failures
      const errs = [];

      // For each case...
      for (let caseIdx = 0; caseIdx < cases.length; caseIdx++) {
        const c = cases[caseIdx];
        const got = outputs[caseIdx];
        const cmp = toComparator(c.expected).compare(got);
        if (!cmp.matched) {
          errs.push(`(${c.input instanceof Array ? c.input.join(', ') : c.input})
    returned: ${cmp.got}
    expected: ${cmp.expected}`);
        }
      }

      return errs.length > 0 ? new Error(errs.join('\n\n')) : undefined;
    };

    // Heartbeat to ensure CTS runners know we're alive.
    globalTestConfig.testHeartbeatCallback();

    t.expectGPUBufferValuesPassCheck(outputBuffer, checkExpectation, {
      type: Uint8Array,
      typedLength: outputBufferSize,
    });
  };
}

/**
 * map is a helper for returning a new array with each element of @p v
 * transformed with @p fn.
 * If @p v is not an array, then @p fn is called with (v, 0).
 */
function map(v, fn) {
  if (v instanceof Array) {
    return v.map(fn);
  }
  return [fn(v, 0)];
}

/**
 * ShaderBuilder is a function used to construct the WGSL shader used by an
 * expression test.
 * @param parameterTypes the list of expression parameter types
 * @param resultType the return type for the expression overload
 * @param cases list of test cases that fit within the binding limits of the device
 * @param inputSource the source of the input values
 */

/**
 * Helper that returns the WGSL to declare the output storage buffer for a shader
 */
function wgslOutputs(resultType, count) {
  return `
struct Output {
  @size(${valueStride(resultType)}) value : ${storageType(resultType)}
};
@group(0) @binding(0) var<storage, read_write> outputs : array<Output, ${count}>;`;
}

/**
 * Helper that returns the WGSL to declare the values array for a shader
 */
function wgslValuesArray(parameterTypes, resultType, cases, expressionBuilder) {
  // AbstractFloat values cannot be stored in an array
  if (parameterTypes.some(ty => scalarTypeOf(ty).kind === 'abstract-float')) {
    return '';
  }
  return `
const values = array(
  ${cases.map(c => expressionBuilder(map(c.input, v => v.wgsl()))).join(',\n  ')}
);`;
}

/**
 * Helper that returns the WGSL 'var' declaration for the given input source
 */
function wgslInputVar(inputSource, count) {
  switch (inputSource) {
    case 'storage_r':
      return `@group(0) @binding(1) var<storage, read> inputs : array<Input, ${count}>;`;
    case 'storage_rw':
      return `@group(0) @binding(1) var<storage, read_write> inputs : array<Input, ${count}>;`;
    case 'uniform':
      return `@group(0) @binding(1) var<uniform> inputs : array<Input, ${count}>;`;
  }

  throw new Error(`InputSource ${inputSource} does not use an input var`);
}

/**
 * Helper that returns the WGSL header before any other declaration, currently include f16
 * enable directive if necessary.
 */
function wgslHeader(parameterTypes, resultType) {
  const usedF16 =
    scalarTypeOf(resultType).kind === 'f16' ||
    parameterTypes.some(ty => scalarTypeOf(ty).kind === 'f16');
  const header = usedF16 ? 'enable f16;\n' : '';
  return header;
}

/**
 * ExpressionBuilder returns the WGSL used to evaluate an expression with the
 * given input values.
 */

/**
 * Returns a ShaderBuilder that builds a basic expression test shader.
 * @param expressionBuilder the expression builder
 */
export function basicExpressionBuilder(expressionBuilder) {
  return (parameterTypes, resultType, cases, inputSource) => {
    if (inputSource === 'const') {
      //////////////////////////////////////////////////////////////////////////
      // Constant eval
      //////////////////////////////////////////////////////////////////////////
      let body = '';
      if (parameterTypes.some(ty => scalarTypeOf(ty).kind === 'abstract-float')) {
        // Directly assign the expression to the output, to avoid an
        // intermediate store, which will concretize the value early
        body = cases
          .map(
            (c, i) => `  outputs[${i}].value = ${expressionBuilder(map(c.input, v => v.wgsl()))};`
          )
          .join('\n  ');
      } else if (globalTestConfig.unrollConstEvalLoops) {
        body = cases
          .map((_, i) => {
            const value = `values[${i}]`;
            return `  outputs[${i}].value = ${toStorage(resultType, value)};`;
          })
          .join('\n  ');
      } else {
        body = `
  for (var i = 0u; i < ${cases.length}; i++) {
    outputs[i].value = ${toStorage(resultType, `values[i]`)};
  }`;
      }

      return `
${wgslHeader(parameterTypes, resultType)}

${wgslOutputs(resultType, cases.length)}

${wgslValuesArray(parameterTypes, resultType, cases, expressionBuilder)}

@compute @workgroup_size(1)
fn main() {
${body}
}`;
    } else {
      //////////////////////////////////////////////////////////////////////////
      // Runtime eval
      //////////////////////////////////////////////////////////////////////////

      // returns the WGSL expression to load the ith parameter of the given type from the input buffer
      const paramExpr = (ty, i) => fromStorage(ty, `inputs[i].param${i}`);

      // resolves to the expression that calls the builtin
      const expr = toStorage(resultType, expressionBuilder(parameterTypes.map(paramExpr)));

      return `
${wgslHeader(parameterTypes, resultType)}

struct Input {
${parameterTypes
  .map((ty, i) => `  @size(${valueStride(ty)}) param${i} : ${storageType(ty)},`)
  .join('\n')}
};

${wgslOutputs(resultType, cases.length)}

${wgslInputVar(inputSource, cases.length)}

@compute @workgroup_size(1)
fn main() {
  for (var i = 0; i < ${cases.length}; i++) {
    outputs[i].value = ${expr};
  }
}
`;
    }
  };
}

/**
 * Returns a ShaderBuilder that builds a compound assignment operator test shader.
 * @param op the compound operator
 */
export function compoundAssignmentBuilder(op) {
  return (parameterTypes, resultType, cases, inputSource) => {
    //////////////////////////////////////////////////////////////////////////
    // Input validation
    //////////////////////////////////////////////////////////////////////////
    if (parameterTypes.length !== 2) {
      throw new Error(`compoundBinaryOp() requires exactly two parameters values per case`);
    }
    const lhsType = parameterTypes[0];
    const rhsType = parameterTypes[1];
    if (!objectEquals(lhsType, resultType)) {
      throw new Error(
        `compoundBinaryOp() requires result type (${resultType}) to be equal to the LHS type (${lhsType})`
      );
    }
    if (inputSource === 'const') {
      //////////////////////////////////////////////////////////////////////////
      // Constant eval
      //////////////////////////////////////////////////////////////////////////
      let body = '';
      if (globalTestConfig.unrollConstEvalLoops) {
        body = cases
          .map((_, i) => {
            return `
  var ret_${i} = lhs[${i}];
  ret_${i} ${op} rhs[${i}];
  outputs[${i}].value = ${storageType(resultType)}(ret_${i});`;
          })
          .join('\n  ');
      } else {
        body = `
  for (var i = 0u; i < ${cases.length}; i++) {
    var ret = lhs[i];
    ret ${op} rhs[i];
    outputs[i].value = ${storageType(resultType)}(ret);
  }`;
      }

      const values = cases.map(c => c.input.map(v => v.wgsl()));

      return `
${wgslHeader(parameterTypes, resultType)}
${wgslOutputs(resultType, cases.length)}

const lhs = array(
${values.map(c => `${c[0]}`).join(',\n  ')}
      );
const rhs = array(
${values.map(c => `${c[1]}`).join(',\n  ')}
);

@compute @workgroup_size(1)
fn main() {
${body}
}`;
    } else {
      //////////////////////////////////////////////////////////////////////////
      // Runtime eval
      //////////////////////////////////////////////////////////////////////////
      return `
${wgslHeader(parameterTypes, resultType)}
${wgslOutputs(resultType, cases.length)}

struct Input {
  @size(${valueStride(lhsType)}) lhs : ${storageType(lhsType)},
  @size(${valueStride(rhsType)}) rhs : ${storageType(rhsType)},
}

${wgslInputVar(inputSource, cases.length)}

@compute @workgroup_size(1)
fn main() {
  for (var i = 0; i < ${cases.length}; i++) {
    var ret = ${lhsType}(inputs[i].lhs);
    ret ${op} ${rhsType}(inputs[i].rhs);
    outputs[i].value = ${storageType(resultType)}(ret);
  }
}
`;
    }
  };
}

/**
 * Constructs and returns a GPUComputePipeline and GPUBindGroup for running a
 * batch of test cases. If a pre-created pipeline can be found in
 * @p pipelineCache, then this may be returned instead of creating a new
 * pipeline.
 * @param t the GPUTest
 * @param shaderBuilder the shader builder
 * @param parameterTypes the list of expression parameter types
 * @param resultType the return type for the expression overload
 * @param cases list of test cases that fit within the binding limits of the device
 * @param inputSource the source of the input values
 * @param outputBuffer the buffer that will hold the output values of the tests
 * @param pipelineCache the cache of compute pipelines, shared between batches
 */
function buildPipeline(
  t,
  shaderBuilder,
  parameterTypes,
  resultType,
  cases,
  inputSource,
  outputBuffer,
  pipelineCache
) {
  cases.forEach(c => {
    const inputTypes = c.input instanceof Array ? c.input.map(i => i.type) : [c.input.type];
    if (!objectEquals(inputTypes, parameterTypes)) {
      const input_str = `[${inputTypes.join(',')}]`;
      const param_str = `[${parameterTypes.join(',')}]`;
      throw new Error(
        `case input types ${input_str} do not match provided runner parameter types ${param_str}`
      );
    }
  });

  const source = shaderBuilder(parameterTypes, resultType, cases, inputSource);

  switch (inputSource) {
    case 'const': {
      // build the shader module
      const module = t.device.createShaderModule({ code: source });

      // build the pipeline
      const pipeline = t.device.createComputePipeline({
        layout: 'auto',
        compute: { module, entryPoint: 'main' },
      });

      // build the bind group
      const group = t.device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [{ binding: 0, resource: { buffer: outputBuffer } }],
      });

      return [pipeline, group];
    }

    case 'uniform':
    case 'storage_r':
    case 'storage_rw': {
      // Input values come from a uniform or storage buffer

      // size in bytes of the input buffer
      const inputSize = cases.length * valueStrides(parameterTypes);

      // Holds all the parameter values for all cases
      const inputData = new Uint8Array(inputSize);

      // Pack all the input parameter values into the inputData buffer
      {
        const caseStride = valueStrides(parameterTypes);
        for (let caseIdx = 0; caseIdx < cases.length; caseIdx++) {
          const caseBase = caseIdx * caseStride;
          let offset = caseBase;
          for (let paramIdx = 0; paramIdx < parameterTypes.length; paramIdx++) {
            const params = cases[caseIdx].input;
            if (params instanceof Array) {
              params[paramIdx].copyTo(inputData, offset);
            } else {
              params.copyTo(inputData, offset);
            }
            offset += valueStride(parameterTypes[paramIdx]);
          }
        }
      }

      // build the compute pipeline, if the shader hasn't been compiled already.
      const pipeline = getOrCreate(pipelineCache, source, () => {
        // build the shader module
        const module = t.device.createShaderModule({ code: source });

        // build the pipeline
        return t.device.createComputePipeline({
          layout: 'auto',
          compute: { module, entryPoint: 'main' },
        });
      });

      // build the input buffer
      const inputBuffer = t.makeBufferWithContents(
        inputData,
        GPUBufferUsage.COPY_SRC |
          (inputSource === 'uniform' ? GPUBufferUsage.UNIFORM : GPUBufferUsage.STORAGE)
      );

      // build the bind group
      const group = t.device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          { binding: 0, resource: { buffer: outputBuffer } },
          { binding: 1, resource: { buffer: inputBuffer } },
        ],
      });

      return [pipeline, group];
    }
  }
}

/**
 * Packs a list of scalar test cases into a smaller list of vector cases.
 * Requires that all parameters of the expression overload are of a scalar type,
 * and the return type of the expression overload is also a scalar type.
 * If `cases.length` is not a multiple of `vectorWidth`, then the last scalar
 * test case value is repeated to fill the vector value.
 */
function packScalarsToVector(parameterTypes, resultType, cases, vectorWidth) {
  // Validate that the parameters and return type are all vectorizable
  for (let i = 0; i < parameterTypes.length; i++) {
    const ty = parameterTypes[i];
    if (!(ty instanceof ScalarType)) {
      throw new Error(
        `packScalarsToVector() can only be used on scalar parameter types, but the ${i}'th parameter type is a ${ty}'`
      );
    }
  }
  if (!(resultType instanceof ScalarType)) {
    throw new Error(
      `packScalarsToVector() can only be used with a scalar return type, but the return type is a ${resultType}'`
    );
  }

  const packedCases = [];
  const packedParameterTypes = parameterTypes.map(p => TypeVec(vectorWidth, p));
  const packedResultType = new VectorType(vectorWidth, resultType);

  const clampCaseIdx = idx => Math.min(idx, cases.length - 1);

  let caseIdx = 0;
  while (caseIdx < cases.length) {
    // Construct the vectorized inputs from the scalar cases
    const packedInputs = new Array(parameterTypes.length);
    for (let paramIdx = 0; paramIdx < parameterTypes.length; paramIdx++) {
      const inputElements = new Array(vectorWidth);
      for (let i = 0; i < vectorWidth; i++) {
        const input = cases[clampCaseIdx(caseIdx + i)].input;
        inputElements[i] = input instanceof Array ? input[paramIdx] : input;
      }
      packedInputs[paramIdx] = new Vector(inputElements);
    }

    // Gather the comparators for the packed cases
    const cmp_impls = new Array(vectorWidth);
    for (let i = 0; i < vectorWidth; i++) {
      cmp_impls[i] = toComparator(cases[clampCaseIdx(caseIdx + i)].expected).compare;
    }
    const comparators = {
      compare: got => {
        let matched = true;
        const gElements = new Array(vectorWidth);
        const eElements = new Array(vectorWidth);
        for (let i = 0; i < vectorWidth; i++) {
          const d = cmp_impls[i](got.elements[i]);
          matched = matched && d.matched;
          gElements[i] = d.got;
          eElements[i] = d.expected;
        }
        return {
          matched,
          got: `${packedResultType}(${gElements.join(', ')})`,
          expected: `${packedResultType}(${eElements.join(', ')})`,
        };
      },
      kind: 'packed',
    };

    // Append the new packed case
    packedCases.push({ input: packedInputs, expected: comparators });
    caseIdx += vectorWidth;
  }

  return {
    cases: packedCases,
    parameterTypes: packedParameterTypes,
    resultType: packedResultType,
  };
}

/**
 * Indicates bounds that acceptance intervals need to be within to avoid inputs
 * being filtered out. This is used for const-eval tests, since going OOB will
 * cause a validation error not an execution error.
 */

/**
 * @returns array of Case for the input params with op applied
 * @param param0s array of inputs to try for the first param
 * @param param1s array of inputs to try for the second param
 * @param op callback called on each pair of inputs to produce each case
 * @param quantize function to quantize all values
 * @param scalarize function to convert numbers to Scalars
 */
function generateScalarBinaryToScalarCases(param0s, param1s, op, quantize, scalarize) {
  param0s = param0s.map(quantize);
  param1s = param1s.map(quantize);
  return cartesianProduct(param0s, param1s).reduce((cases, e) => {
    const expected = op(e[0], e[1]);
    if (expected !== undefined) {
      cases.push({ input: [scalarize(e[0]), scalarize(e[1])], expected: scalarize(expected) });
    }
    return cases;
  }, new Array());
}

/**
 * @returns an array of Cases for operations over a range of inputs
 * @param param0s array of inputs to try for the first param
 * @param param1s array of inputs to try for the second param
 * @param op callback called on each pair of inputs to produce each case
 */
export function generateBinaryToI32Cases(param0s, param1s, op) {
  return generateScalarBinaryToScalarCases(param0s, param1s, op, quantizeToI32, i32);
}

/**
 * @returns an array of Cases for operations over a range of inputs
 * @param param0s array of inputs to try for the first param
 * @param param1s array of inputs to try for the second param
 * @param op callback called on each pair of inputs to produce each case
 */
export function generateBinaryToU32Cases(param0s, param1s, op) {
  return generateScalarBinaryToScalarCases(param0s, param1s, op, quantizeToU32, u32);
}

/**
 * @returns a Case for the input params with op applied
 * @param scalar scalar param
 * @param vector vector param (2, 3, or 4 elements)
 * @param op the op to apply to scalar and vector
 * @param quantize function to quantize all values in vectors and scalars
 * @param scalarize function to convert numbers to Scalars
 */
function makeScalarVectorBinaryToVectorCase(scalar, vector, op, quantize, scalarize) {
  scalar = quantize(scalar);
  vector = vector.map(quantize);
  const result = vector.map(v => op(scalar, v));
  if (result.includes(undefined)) {
    return undefined;
  }
  return {
    input: [scalarize(scalar), new Vector(vector.map(scalarize))],
    expected: new Vector(result.map(scalarize)),
  };
}

/**
 * @returns array of Case for the input params with op applied
 * @param scalars array of scalar params
 * @param vectors array of vector params (2, 3, or 4 elements)
 * @param op the op to apply to each pair of scalar and vector
 * @param quantize function to quantize all values in vectors and scalars
 * @param scalarize function to convert numbers to Scalars
 */
function generateScalarVectorBinaryToVectorCases(scalars, vectors, op, quantize, scalarize) {
  const cases = new Array();
  scalars.forEach(s => {
    vectors.forEach(v => {
      const c = makeScalarVectorBinaryToVectorCase(s, v, op, quantize, scalarize);
      if (c !== undefined) {
        cases.push(c);
      }
    });
  });
  return cases;
}

/**
 * @returns a Case for the input params with op applied
 * @param vector vector param (2, 3, or 4 elements)
 * @param scalar scalar param
 * @param op the op to apply to vector and scalar
 * @param quantize function to quantize all values in vectors and scalars
 * @param scalarize function to convert numbers to Scalars
 */
function makeVectorScalarBinaryToVectorCase(vector, scalar, op, quantize, scalarize) {
  vector = vector.map(quantize);
  scalar = quantize(scalar);
  const result = vector.map(v => op(v, scalar));
  if (result.includes(undefined)) {
    return undefined;
  }
  return {
    input: [new Vector(vector.map(scalarize)), scalarize(scalar)],
    expected: new Vector(result.map(scalarize)),
  };
}

/**
 * @returns array of Case for the input params with op applied
 * @param vectors array of vector params (2, 3, or 4 elements)
 * @param scalars array of scalar params
 * @param op the op to apply to each pair of vector and scalar
 * @param quantize function to quantize all values in vectors and scalars
 * @param scalarize function to convert numbers to Scalars
 */
function generateVectorScalarBinaryToVectorCases(vectors, scalars, op, quantize, scalarize) {
  const cases = new Array();
  scalars.forEach(s => {
    vectors.forEach(v => {
      const c = makeVectorScalarBinaryToVectorCase(v, s, op, quantize, scalarize);
      if (c !== undefined) {
        cases.push(c);
      }
    });
  });
  return cases;
}

/**
 * @returns array of Case for the input params with op applied
 * @param scalars array of scalar params
 * @param vectors array of vector params (2, 3, or 4 elements)
 * @param op he op to apply to each pair of scalar and vector
 */
export function generateU32VectorBinaryToVectorCases(scalars, vectors, op) {
  return generateScalarVectorBinaryToVectorCases(scalars, vectors, op, quantizeToU32, u32);
}

/**
 * @returns array of Case for the input params with op applied
 * @param vectors array of vector params (2, 3, or 4 elements)
 * @param scalars array of scalar params
 * @param op he op to apply to each pair of vector and scalar
 */
export function generateVectorU32BinaryToVectorCases(vectors, scalars, op) {
  return generateVectorScalarBinaryToVectorCases(vectors, scalars, op, quantizeToU32, u32);
}

/**
 * @returns array of Case for the input params with op applied
 * @param scalars array of scalar params
 * @param vectors array of vector params (2, 3, or 4 elements)
 * @param op he op to apply to each pair of scalar and vector
 */
export function generateI32VectorBinaryToVectorCases(scalars, vectors, op) {
  return generateScalarVectorBinaryToVectorCases(scalars, vectors, op, quantizeToI32, i32);
}

/**
 * @returns array of Case for the input params with op applied
 * @param vectors array of vector params (2, 3, or 4 elements)
 * @param scalars array of scalar params
 * @param op he op to apply to each pair of vector and scalar
 */
export function generateVectorI32BinaryToVectorCases(vectors, scalars, op) {
  return generateVectorScalarBinaryToVectorCases(vectors, scalars, op, quantizeToI32, i32);
}

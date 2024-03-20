/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { globalTestConfig } from '../../../../common/framework/test_config.js';import { assert, objectEquals, unreachable } from '../../../../common/util/util.js';

import { kValue } from '../../../util/constants.js';
import {
  MatrixType,

  ScalarType,

  TypeU32,
  TypeVec,

  Vector,
  VectorType,
  scalarTypeOf } from
'../../../util/conversion.js';


import { toComparator } from './expectation.js';

/** The input value source */




// Read-write storage buffer

/** All possible input sources */
export const allInputSources = ['const', 'uniform', 'storage_r', 'storage_rw'];

/** Just constant input source */
export const onlyConstInputSource = ['const'];

/** All input sources except const */
export const allButConstInputSource = ['uniform', 'storage_r', 'storage_rw'];

/** Configuration for running a expression test */












// Helper for returning the stride for a given Type
function valueStride(ty) {
  // AbstractFloats and AbstractInts are passed out of the shader via structs of
  // 2x u32s and unpacking containers as arrays
  if (scalarTypeOf(ty).kind === 'abstract-float' || scalarTypeOf(ty).kind === 'abstract-int') {
    if (ty instanceof ScalarType) {
      return 16;
    }
    if (ty instanceof VectorType) {
      if (ty.width === 2) {
        return 16;
      }
      // vec3s have padding to make them the same size as vec4s
      return 32;
    }
    if (ty instanceof MatrixType) {
      switch (ty.cols) {
        case 2:
          switch (ty.rows) {
            case 2:
              return 32;
            case 3:
              return 64;
            case 4:
              return 64;
          }
          break;
        case 3:
          switch (ty.rows) {
            case 2:
              return 48;
            case 3:
              return 96;
            case 4:
              return 96;
          }
          break;
        case 4:
          switch (ty.rows) {
            case 2:
              return 64;
            case 3:
              return 128;
            case 4:
              return 128;
          }
          break;
      }
    }
    unreachable(`AbstractFloats have not yet been implemented for ${ty.toString()}`);
  }

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

// Helper for summing up all the stride values for an array of Types
function valueStrides(tys) {
  return tys.map(valueStride).reduce((sum, c) => sum + c);
}

// Helper for returning the WGSL storage type for the given Type.
function storageType(ty) {
  if (ty instanceof ScalarType) {
    assert(ty.kind !== 'f64', `No storage type defined for 'f64' values`);
    assert(ty.kind !== 'abstract-int', `Custom handling is implemented for 'abstract-int' values`);
    assert(
      ty.kind !== 'abstract-float',
      `Custom handling is implemented for 'abstract-float' values`
    );
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
    assert(ty.kind !== 'abstract-int', `'abstract-int' values should not be in input storage`);
    assert(ty.kind !== 'abstract-float', `'abstract-float' values should not be in input storage`);
    assert(ty.kind !== 'f64', `'No storage type defined for 'f64' values`);
    if (ty.kind === 'bool') {
      return `${expr} != 0u`;
    }
  }
  if (ty instanceof VectorType) {
    assert(
      ty.elementType.kind !== 'abstract-int',
      `'abstract-int' values cannot appear in input storage`
    );
    assert(
      ty.elementType.kind !== 'abstract-float',
      `'abstract-float' values cannot appear in input storage`
    );
    assert(ty.elementType.kind !== 'f64', `'No storage type defined for 'f64' values`);
    if (ty.elementType.kind === 'bool') {
      return `${expr} != vec${ty.width}<u32>(0u)`;
    }
  }
  return expr;
}

// Helper for converting a value of the type 'ty' to the storage type.
function toStorage(ty, expr) {
  if (ty instanceof ScalarType) {
    assert(
      ty.kind !== 'abstract-int',
      `'abstract-int' values have custom code for writing to storage`
    );
    assert(
      ty.kind !== 'abstract-float',
      `'abstract-float' values have custom code for writing to storage`
    );
    assert(ty.kind !== 'f64', `No storage type defined for 'f64' values`);
    if (ty.kind === 'bool') {
      return `select(0u, 1u, ${expr})`;
    }
  }
  if (ty instanceof VectorType) {
    assert(
      ty.elementType.kind !== 'abstract-int',
      `'abstract-int' values have custom code for writing to storage`
    );
    assert(
      ty.elementType.kind !== 'abstract-float',
      `'abstract-float' values have custom code for writing to storage`
    );
    assert(ty.elementType.kind !== 'f64', `'No storage type defined for 'f64' values`);
    if (ty.elementType.kind === 'bool') {
      return `select(vec${ty.width}<u32>(0u), vec${ty.width}<u32>(1u), ${expr})`;
    }
  }
  return expr;
}

// A Pipeline is a map of WGSL shader source to a built pipeline


/**
 * Searches for an entry with the given key, adding and returning the result of calling
 * `create` if the entry was not found.
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
 * @param batch_size override the calculated casesPerBatch.
 */
export async function run(
t,
shaderBuilder,
parameterTypes,
resultType,
cfg = { inputSource: 'storage_r' },
cases,
batch_size)
{
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
  const casesPerBatch = function () {
    if (batch_size) {
      return batch_size;
    }
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
  }();

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

  const processBatch = async (batchCases) => {
    const checkBatch = await submitBatch(
      t,
      shaderBuilder,
      parameterTypes,
      resultType,
      batchCases,
      cfg.inputSource,
      pipelineCache
    );
    checkBatch();
    void t.queue.onSubmittedWorkDone().finally(batchFinishedCallback);
  };

  const pendingBatches = [];

  for (let i = 0; i < cases.length; i += casesPerBatch) {
    const batchCases = cases.slice(i, Math.min(i + casesPerBatch, cases.length));

    if (batchesInFlight > maxBatchesInFlight) {
      await new Promise((resolve) => {
        // There should only be one batch waiting at a time.
        assert(resolvePromiseBlockingBatch === undefined);
        resolvePromiseBlockingBatch = resolve;
      });
    }
    batchesInFlight += 1;

    pendingBatches.push(processBatch(batchCases));
  }

  await Promise.all(pendingBatches);
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
async function submitBatch(
t,
shaderBuilder,
parameterTypes,
resultType,
cases,
inputSource,
pipelineCache)
{
  // Construct a buffer to hold the results of the expression tests
  const outputBufferSize = cases.length * valueStride(resultType);
  const outputBuffer = t.device.createBuffer({
    size: outputBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  });

  const [pipeline, group] = await buildPipeline(
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
    const checkExpectation = (outputData) => {
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
      typedLength: outputBufferSize
    });
  };
}

/**
 * map is a helper for returning a new array with each element of `v`
 * transformed with `fn`.
 * If `v` is not an array, then `fn` is called with (v, 0).
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
  let output_struct = undefined;
  if (
  scalarTypeOf(resultType).kind !== 'abstract-float' &&
  scalarTypeOf(resultType).kind !== 'abstract-int')
  {
    output_struct = `
struct Output {
  @size(${valueStride(resultType)}) value : ${storageType(resultType)}
};`;
  } else {
    if (resultType instanceof ScalarType) {
      output_struct = `struct AF {
  low: u32,
  high: u32,
};

struct Output {
  @size(${valueStride(resultType)}) value: AF,
};`;
    }
    if (resultType instanceof VectorType) {
      const dim = resultType.width;
      output_struct = `struct AF {
  low: u32,
  high: u32,
};

struct Output {
  @size(${valueStride(resultType)}) value: array<AF, ${dim}>,
};`;
    }

    if (resultType instanceof MatrixType) {
      const cols = resultType.cols;
      const rows = resultType.rows === 2 ? 2 : 4; // 3 element rows have a padding element
      output_struct = `struct AF {
  low: u32,
  high: u32,
};

struct Output {
   @size(${valueStride(resultType)}) value: array<array<AF, ${rows}>, ${cols}>,
};`;
    }

    assert(output_struct !== undefined, `No implementation for result type '${resultType}'`);
  }

  return `${output_struct}
@group(0) @binding(0) var<storage, read_write> outputs : array<Output, ${count}>;
`;
}

/**
 * Helper that returns the WGSL to declare the values array for a shader
 */
function wgslValuesArray(
parameterTypes,
resultType,
cases,
expressionBuilder)
{
  return `
const values = array(
  ${cases.map((c) => expressionBuilder(map(c.input, (v) => v.wgsl()))).join(',\n  ')}
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
  parameterTypes.some((ty) => scalarTypeOf(ty).kind === 'f16');
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
function basicExpressionShaderBody(
expressionBuilder,
parameterTypes,
resultType,
cases,
inputSource)
{
  assert(
    scalarTypeOf(resultType).kind !== 'abstract-int',
    `abstractIntShaderBuilder should be used when result type is 'abstract-int'`
  );
  assert(
    scalarTypeOf(resultType).kind !== 'abstract-float',
    `abstractFloatShaderBuilder should be used when result type is 'abstract-float'`
  );
  if (inputSource === 'const') {
    //////////////////////////////////////////////////////////////////////////
    // Constant eval
    //////////////////////////////////////////////////////////////////////////
    let body = '';
    if (parameterTypes.some((ty) => scalarTypeOf(ty).kind === 'abstract-float')) {
      // Directly assign the expression to the output, to avoid an
      // intermediate store, which will concretize the value early
      body = cases.
      map(
        (c, i) =>
        `  outputs[${i}].value = ${toStorage(
          resultType,
          expressionBuilder(map(c.input, (v) => v.wgsl()))
        )};`
      ).
      join('\n  ');
    } else if (globalTestConfig.unrollConstEvalLoops) {
      body = cases.
      map((_, i) => {
        const value = `values[${i}]`;
        return `  outputs[${i}].value = ${toStorage(resultType, value)};`;
      }).
      join('\n  ');
    } else {
      body = `
  for (var i = 0u; i < ${cases.length}; i++) {
    outputs[i].value = ${toStorage(resultType, `values[i]`)};
  }`;
    }

    return `
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
struct Input {
${parameterTypes.
    map((ty, i) => `  @size(${valueStride(ty)}) param${i} : ${storageType(ty)},`).
    join('\n')}
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
}

/**
 * Returns a ShaderBuilder that builds a basic expression test shader.
 * @param expressionBuilder the expression builder
 */
export function basicExpressionBuilder(expressionBuilder) {
  return (
  parameterTypes,
  resultType,
  cases,
  inputSource) =>
  {
    return `\
${wgslHeader(parameterTypes, resultType)}

${basicExpressionShaderBody(expressionBuilder, parameterTypes, resultType, cases, inputSource)}`;
  };
}

/**
 * Returns a ShaderBuilder that builds a basic expression test shader with given predeclaration
 * string goes after WGSL header (i.e. enable directives) if any but before anything else.
 * @param expressionBuilder the expression builder
 * @param predeclaration the predeclaration string
 */
export function basicExpressionWithPredeclarationBuilder(
expressionBuilder,
predeclaration)
{
  return (
  parameterTypes,
  resultType,
  cases,
  inputSource) =>
  {
    return `\
${wgslHeader(parameterTypes, resultType)}

${predeclaration}

${basicExpressionShaderBody(expressionBuilder, parameterTypes, resultType, cases, inputSource)}`;
  };
}

/**
 * Returns a ShaderBuilder that builds a compound assignment operator test shader.
 * @param op the compound operator
 */
export function compoundAssignmentBuilder(op) {
  return (
  parameterTypes,
  resultType,
  cases,
  inputSource) =>
  {
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
        body = cases.
        map((_, i) => {
          return `
  var ret_${i} = lhs[${i}];
  ret_${i} ${op} rhs[${i}];
  outputs[${i}].value = ${storageType(resultType)}(ret_${i});`;
        }).
        join('\n  ');
      } else {
        body = `
  for (var i = 0u; i < ${cases.length}; i++) {
    var ret = lhs[i];
    ret ${op} rhs[i];
    outputs[i].value = ${storageType(resultType)}(ret);
  }`;
      }

      const values = cases.map((c) => c.input.map((v) => v.wgsl()));

      return `
${wgslHeader(parameterTypes, resultType)}
${wgslOutputs(resultType, cases.length)}

const lhs = array(
${values.map((c) => `${c[0]}`).join(',\n  ')}
      );
const rhs = array(
${values.map((c) => `${c[1]}`).join(',\n  ')}
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
 * @returns a string that extracts the value of an AbstractFloat into an output
 *          destination
 * @param expr expression for an AbstractFloat value, if working with vectors or
 *             matrices, this string needs to include indexing into the
 *             container.
 * @param case_idx index in the case output array to assign the result
 * @param accessor string representing how access to the AF that needs to be
 *                 operated on.
 *                 For scalars this should be left as ''.
 *                 For vectors this will be an indexing operation,
 *                 i.e. '[i]'
 *                 For matrices this will double indexing operation,
 *                 i.e. '[c][r]'
 */
function abstractFloatSnippet(expr, case_idx, accessor = '') {
  // AbstractFloats are f64s under the hood. WebGPU does not support
  // putting f64s in buffers, so the result needs to be split up into u32s
  // and rebuilt in the test framework.
  //
  // Since there is no 64-bit data type that can be used as an element for a
  // vector or a matrix in WGSL, the testing framework needs to pass the u32s
  // via a struct with two u32s, and deconstruct vectors and matrices into
  // arrays.
  //
  // This is complicated by the fact that user defined functions cannot
  // take/return AbstractFloats, and AbstractFloats cannot be stored in
  // variables, so the code cannot just inject a simple utility function
  // at the top of the shader, instead this snippet needs to be inlined
  // everywhere the test needs to return an AbstractFloat.
  //
  // select is used below, since ifs are not available during constant
  // eval. This has the side effect of short-circuiting doesn't occur, so
  // both sides of the select have to evaluate and be valid.
  //
  // This snippet implements FTZ for subnormals to bypass the need for
  // complex subnormal specific logic.
  //
  // Expressions resulting in subnormals can still be reasonably tested,
  // since this snippet will return 0 with the correct sign, which is
  // always in the acceptance interval for a subnormal result, since an
  // implementation may FTZ.
  //
  // Documentation for the snippet working with scalar results is included here
  // in this code block, since shader length affects compilation time
  // significantly on some backends. The code for vectors and matrices basically
  // the same thing, with extra indexing operations.
  //
  // Snippet with documentation:
  //   const kExponentBias = 1022;
  //
  //   // Detect if the value is zero or subnormal, so that FTZ behaviour
  //   // can occur
  //   const subnormal_or_zero : bool = (${expr} <= ${kValue.f64.positive.subnormal.max}) && (${expr} >= ${kValue.f64.negative.subnormal.min});
  //
  //   // MSB of the upper u32 is 1 if the value is negative, otherwise 0
  //   // Extract the sign bit early, so that abs() can be used with
  //   // frexp() so negative cases do not need to be handled
  //   const sign_bit : u32 = select(0, 0x80000000, ${expr} < 0);
  //
  //   // Use frexp() to obtain the exponent and fractional parts, and
  //   // then perform FTZ if needed
  //   const f = frexp(abs(${expr}));
  //   const f_fract = select(f.fract, 0, subnormal_or_zero);
  //   const f_exp = select(f.exp, -kExponentBias, subnormal_or_zero);
  //
  //   // Adjust for the exponent bias and shift for storing in bits
  //   // [20..31] of the upper u32
  //   const exponent_bits : u32 = (f_exp + kExponentBias) << 20;
  //
  //   // Extract the portion of the mantissa that appears in upper u32 as
  //   // a float for later use
  //   const high_mantissa = ldexp(f_fract, 21);
  //
  //   // Extract the portion of the mantissa that appears in upper u32 as
  //   // as bits. This value is masked, because normals will explicitly
  //   // have the implicit leading 1 that should not be in the final
  //   // result.
  //   const high_mantissa_bits : u32 = u32(ldexp(f_fract, 21)) & 0x000fffff;
  //
  //   // Calculate the mantissa stored in the lower u32 as a float
  //   const low_mantissa = f_fract - ldexp(floor(high_mantissa), -21);
  //
  //   // Convert the lower u32 mantissa to bits
  //   const low_mantissa_bits = u32(ldexp(low_mantissa, 53));
  //
  //   outputs[${i}].value.high = sign_bit | exponent_bits | high_mantissa_bits;
  //   outputs[${i}].value.low = low_mantissa_bits;

  return `  {
    const kExponentBias = 1022;
    const subnormal_or_zero : bool = (${expr}${accessor} <= ${kValue.f64.positive.subnormal.max}) && (${expr}${accessor} >= ${kValue.f64.negative.subnormal.min});
    const sign_bit : u32 = select(0, 0x80000000, ${expr}${accessor} < 0);
    const f = frexp(abs(${expr}${accessor}));
    const f_fract = select(f.fract, 0, subnormal_or_zero);
    const f_exp = select(f.exp, -kExponentBias, subnormal_or_zero);
    const exponent_bits : u32 = (f_exp + kExponentBias) << 20;
    const high_mantissa = ldexp(f_fract, 21);
    const high_mantissa_bits : u32 = u32(ldexp(f_fract, 21)) & 0x000fffff;
    const low_mantissa = f_fract - ldexp(floor(high_mantissa), -21);
    const low_mantissa_bits = u32(ldexp(low_mantissa, 53));
    outputs[${case_idx}].value${accessor}.high = sign_bit | exponent_bits | high_mantissa_bits;
    outputs[${case_idx}].value${accessor}.low = low_mantissa_bits;
  }`;
}

/** @returns a string for a specific case that has a AbstractFloat result */
function abstractFloatCaseBody(expr, resultType, i) {
  if (resultType instanceof ScalarType) {
    return abstractFloatSnippet(expr, i);
  }

  if (resultType instanceof VectorType) {
    return [...Array(resultType.width).keys()].
    map((idx) => abstractFloatSnippet(expr, i, `[${idx}]`)).
    join('  \n');
  }

  if (resultType instanceof MatrixType) {
    const cols = resultType.cols;
    const rows = resultType.rows;
    const results = [...Array(cols * rows)];

    for (let c = 0; c < cols; c++) {
      for (let r = 0; r < rows; r++) {
        results[c * rows + r] = abstractFloatSnippet(expr, i, `[${c}][${r}]`);
      }
    }

    return results.join('  \n');
  }

  unreachable(`Results of type '${resultType}' not yet implemented`);
}

/**
 * @returns a ShaderBuilder that builds a test shader hands AbstractFloat results.
 * @param expressionBuilder an expression builder that will return AbstractFloats
 */
export function abstractFloatShaderBuilder(expressionBuilder) {
  return (
  parameterTypes,
  resultType,
  cases,
  inputSource) =>
  {
    assert(inputSource === 'const', `'abstract-float' results are only defined for const-eval`);
    assert(
      scalarTypeOf(resultType).kind === 'abstract-float',
      `Expected resultType of 'abstract-float', received '${scalarTypeOf(resultType).kind}' instead`
    );

    const body = cases.
    map((c, i) => {
      const expr = `${expressionBuilder(map(c.input, (v) => v.wgsl()))}`;
      return abstractFloatCaseBody(expr, resultType, i);
    }).
    join('\n  ');

    return `
${wgslHeader(parameterTypes, resultType)}

${wgslOutputs(resultType, cases.length)}

@compute @workgroup_size(1)
fn main() {
${body}
}`;
  };
}

/**
 * @returns a string that extracts the value of an AbstractInt into an output
 *          destination
 * @param expr expression for an AbstractInt value, if working with vectors,
 *             this string needs to include indexing into the container.
 * @param case_idx index in the case output array to assign the result
 * @param accessor string representing how access to the AbstractInt that needs
 *                 to be operated on.
 *                 For scalars this should be left as ''.
 *                 For vectors this will be an indexing operation,
 *                 i.e. '[i]'
 */
function abstractIntSnippet(expr, case_idx, accessor = '') {
  // AbstractInts are i64s under the hood. WebGPU does not support
  // putting i64s in buffers, or any 64-bit simple types, so the result needs to
  // be split up into u32 bitfields
  //
  // Since there is no 64-bit data type that can be used as an element for a
  // vector or a matrix in WGSL, the testing framework needs to pass the u32s
  // via a struct with two u32s, and deconstruct vectors into arrays.
  //
  // This is complicated by the fact that user defined functions cannot
  // take/return AbstractInts, and AbstractInts cannot be stored in
  // variables, so the code cannot just inject a simple utility function
  // at the top of the shader, instead this snippet needs to be inlined
  // everywhere the test needs to return an AbstractInt.
  return `  {
    outputs[${case_idx}].value${accessor}.high = bitcast<u32>(i32(${expr}${accessor} >> 32)) & 0xFFFFFFFF;
    const low_sign = (${expr}${accessor} & (1 << 31));
    outputs[${case_idx}].value${accessor}.low = bitcast<u32>((${expr}${accessor} & 0x7FFFFFFF)) | low_sign;
  }`;
}

/** @returns a string for a specific case that has a AbstractInt result */
function abstractIntCaseBody(expr, resultType, i) {
  if (resultType instanceof ScalarType) {
    return abstractIntSnippet(expr, i);
  }

  if (resultType instanceof VectorType) {
    return [...Array(resultType.width).keys()].
    map((idx) => abstractIntSnippet(expr, i, `[${idx}]`)).
    join('  \n');
  }

  unreachable(`Results of type '${resultType}' not yet implemented`);
}

/**
 * @returns a ShaderBuilder that builds a test shader hands AbstractInt results.
 * @param expressionBuilder an expression builder that will return AbstractInts
 */
export function abstractIntShaderBuilder(expressionBuilder) {
  return (
  parameterTypes,
  resultType,
  cases,
  inputSource) =>
  {
    assert(inputSource === 'const', `'abstract-int' results are only defined for const-eval`);
    assert(
      scalarTypeOf(resultType).kind === 'abstract-int',
      `Expected resultType of 'abstract-int', received '${scalarTypeOf(resultType).kind}' instead`
    );

    const body = cases.
    map((c, i) => {
      const expr = `${expressionBuilder(map(c.input, (v) => v.wgsl()))}`;
      return abstractIntCaseBody(expr, resultType, i);
    }).
    join('\n  ');

    return `
${wgslHeader(parameterTypes, resultType)}

${wgslOutputs(resultType, cases.length)}

@compute @workgroup_size(1)
fn main() {
${body}
}`;
  };
}

/**
 * Constructs and returns a GPUComputePipeline and GPUBindGroup for running a
 * batch of test cases. If a pre-created pipeline can be found in
 * `pipelineCache`, then this may be returned instead of creating a new
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
async function buildPipeline(
t,
shaderBuilder,
parameterTypes,
resultType,
cases,
inputSource,
outputBuffer,
pipelineCache)
{
  cases.forEach((c) => {
    const inputTypes = c.input instanceof Array ? c.input.map((i) => i.type) : [c.input.type];
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
    case 'const':{
        // build the shader module
        const module = t.device.createShaderModule({ code: source });

        // build the pipeline
        const pipeline = await t.device.createComputePipelineAsync({
          layout: 'auto',
          compute: { module, entryPoint: 'main' }
        });

        // build the bind group
        const group = t.device.createBindGroup({
          layout: pipeline.getBindGroupLayout(0),
          entries: [{ binding: 0, resource: { buffer: outputBuffer } }]
        });

        return [pipeline, group];
      }

    case 'uniform':
    case 'storage_r':
    case 'storage_rw':{
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
            compute: { module, entryPoint: 'main' }
          });
        });

        // build the input buffer
        const inputBuffer = t.makeBufferWithContents(
          inputData,
          GPUBufferUsage.COPY_SRC | (
          inputSource === 'uniform' ? GPUBufferUsage.UNIFORM : GPUBufferUsage.STORAGE)
        );

        // build the bind group
        const group = t.device.createBindGroup({
          layout: pipeline.getBindGroupLayout(0),
          entries: [
          { binding: 0, resource: { buffer: outputBuffer } },
          { binding: 1, resource: { buffer: inputBuffer } }]

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
function packScalarsToVector(
parameterTypes,
resultType,
cases,
vectorWidth)
{
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
  const packedParameterTypes = parameterTypes.map((p) => TypeVec(vectorWidth, p));
  const packedResultType = new VectorType(vectorWidth, resultType);

  const clampCaseIdx = (idx) => Math.min(idx, cases.length - 1);

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
      compare: (got) => {
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
          expected: `${packedResultType}(${eElements.join(', ')})`
        };
      },
      kind: 'packed'
    };

    // Append the new packed case
    packedCases.push({ input: packedInputs, expected: comparators });
    caseIdx += vectorWidth;
  }

  return {
    cases: packedCases,
    parameterTypes: packedParameterTypes,
    resultType: packedResultType
  };
}
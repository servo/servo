/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert, unreachable } from '../../../../../../common/util/util.js';
import { kValue } from '../../../../../util/constants.js';
import {
  TypeF16,
  elementType,
  elementsOf,
  isAbstractType,
} from '../../../../../util/conversion.js';
import { fullF16Range, fullF32Range, fullF64Range, linearRange } from '../../../../../util/math.js';

/// A linear sweep between -2 to 2
export const kMinusTwoToTwo = linearRange(-2, 2, 10);

/// An array of values ranging from -3π to 3π, with a focus on multiples of π
export const kMinus3PiTo3Pi = [
  -3 * Math.PI,
  -2.999 * Math.PI,

  -2.501 * Math.PI,
  -2.5 * Math.PI,
  -2.499 * Math.PI,

  -2.001 * Math.PI,
  -2.0 * Math.PI,
  -1.999 * Math.PI,

  -1.501 * Math.PI,
  -1.5 * Math.PI,
  -1.499 * Math.PI,

  -1.001 * Math.PI,
  -1.0 * Math.PI,
  -0.999 * Math.PI,

  -0.501 * Math.PI,
  -0.5 * Math.PI,
  -0.499 * Math.PI,

  -0.001,
  0,
  0.001,

  0.499 * Math.PI,
  0.5 * Math.PI,
  0.501 * Math.PI,

  0.999 * Math.PI,
  1.0 * Math.PI,
  1.001 * Math.PI,

  1.499 * Math.PI,
  1.5 * Math.PI,
  1.501 * Math.PI,

  1.999 * Math.PI,
  2.0 * Math.PI,
  2.001 * Math.PI,

  2.499 * Math.PI,
  2.5 * Math.PI,
  2.501 * Math.PI,

  2.999 * Math.PI,
  3 * Math.PI,
];

/// The evaluation stages to test
export const kConstantAndOverrideStages = ['constant', 'override'];

/**
 * @returns true if evaluation stage @p stage supports expressions of type @p.
 */
export function stageSupportsType(stage, type) {
  if (stage === 'override' && isAbstractType(elementType(type))) {
    // Abstract numerics are concretized before being used in an override expression.
    return false;
  }
  return true;
}

/**
 * Runs a validation test to check that evaluation of @p builtin either evaluates with or without
 * error at shader creation time or pipeline creation time.
 * @param t the ShaderValidationTest
 * @param builtin the name of the builtin
 * @param expectedResult false if an error is expected, true if no error is expected
 * @param args the arguments to pass to the builtin
 * @param stage the evaluation stage
 */
export function validateConstOrOverrideBuiltinEval(t, builtin, expectedResult, args, stage) {
  const elTys = args.map(arg => elementType(arg.type));
  const enables = elTys.some(ty => ty === TypeF16) ? 'enable f16;' : '';

  switch (stage) {
    case 'constant': {
      t.expectCompileResult(
        expectedResult,
        `${enables}
const v = ${builtin}(${args.map(arg => arg.wgsl()).join(', ')});`
      );

      break;
    }
    case 'override': {
      assert(!elTys.some(ty => isAbstractType(ty)));
      const constants = {};
      const overrideDecls = [];
      const callArgs = [];
      let numOverrides = 0;
      for (const arg of args) {
        const argOverrides = [];
        for (const el of elementsOf(arg)) {
          const name = `o${numOverrides++}`;
          overrideDecls.push(`override ${name} : ${el.type};`);
          argOverrides.push(name);
          constants[name] = Number(el.value);
        }
        callArgs.push(`${arg.type}(${argOverrides.join(', ')})`);
      }
      t.expectPipelineResult({
        expectedResult,
        code: `${enables}
${overrideDecls.join('\n')}
var<private> v = ${builtin}(${callArgs.join(', ')});`,
        constants,
        reference: ['v'],
      });
      break;
    }
  }
}

/** @returns a sweep of the representable values for element type of @p type */
export function fullRangeForType(type, count) {
  if (count === undefined) {
    count = 25;
  }
  switch (elementType(type)?.kind) {
    case 'abstract-float':
      return fullF64Range({
        pos_sub: Math.ceil((count * 1) / 5),
        pos_norm: Math.ceil((count * 4) / 5),
      });
    case 'f32':
      return fullF32Range({
        pos_sub: Math.ceil((count * 1) / 5),
        pos_norm: Math.ceil((count * 4) / 5),
      });
    case 'f16':
      return fullF16Range({
        pos_sub: Math.ceil((count * 1) / 5),
        pos_norm: Math.ceil((count * 4) / 5),
      });
    case 'i32':
      return linearRange(kValue.i32.negative.min, kValue.i32.positive.max, count).map(f =>
        Math.floor(f)
      );

    case 'u32':
      return linearRange(0, kValue.u32.max, count).map(f => Math.floor(f));
  }

  unreachable();
}

/** @returns all the values in the provided arrays with duplicates removed */
export function unique(...arrays) {
  const set = new Set();
  for (const arr of arrays) {
    for (const item of arr) {
      set.add(item);
    }
  }
  return [...set];
}

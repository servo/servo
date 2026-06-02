/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../util/constants.js';import { sparseI64Range, vectorI64Range } from '../../../../util/math.js';import {
  generateBinaryToI64Cases,
  generateI64VectorBinaryToVectorCases,
  generateVectorI64BinaryToVectorCases } from
'../case.js';
import { makeCaseCache } from '../case_cache.js';

function ai_add(x, y) {
  const result = x + y;
  return !kValue.i64.isOOB(result) ? result : undefined;
}

function ai_div(x, y) {
  if (y === 0n) return undefined;
  if (x === kValue.i64.negative.min && y === -1n) return undefined;
  const result = x / y;
  return !kValue.i64.isOOB(result) ? result : undefined;
}

function ai_mul(x, y) {
  const result = x * y;
  return !kValue.i64.isOOB(result) ? result : undefined;
}

function ai_rem(x, y) {
  if (y === 0n) return undefined;
  if (x === kValue.i64.negative.min && y === -1n) return undefined;
  const result = x % y;
  return !kValue.i64.isOOB(result) ? result : undefined;
}

function ai_sub(x, y) {
  const result = x - y;
  return !kValue.i64.isOOB(result) ? result : undefined;
}

export const d = makeCaseCache('binary/ai_arithmetic', {
  addition: () => {
    return generateBinaryToI64Cases(sparseI64Range(), sparseI64Range(), ai_add);
  },
  addition_scalar_vector2: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(2), ai_add);
  },
  addition_scalar_vector3: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(3), ai_add);
  },
  addition_scalar_vector4: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(4), ai_add);
  },
  addition_vector2_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(2), sparseI64Range(), ai_add);
  },
  addition_vector3_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(3), sparseI64Range(), ai_add);
  },
  addition_vector4_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(4), sparseI64Range(), ai_add);
  },
  division: () => {
    return generateBinaryToI64Cases(sparseI64Range(), sparseI64Range(), ai_div);
  },
  division_scalar_vector2: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(2), ai_div);
  },
  division_scalar_vector3: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(3), ai_div);
  },
  division_scalar_vector4: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(4), ai_div);
  },
  division_vector2_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(2), sparseI64Range(), ai_div);
  },
  division_vector3_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(3), sparseI64Range(), ai_div);
  },
  division_vector4_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(4), sparseI64Range(), ai_div);
  },
  multiplication: () => {
    return generateBinaryToI64Cases(sparseI64Range(), sparseI64Range(), ai_mul);
  },
  multiplication_scalar_vector2: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(2), ai_mul);
  },
  multiplication_scalar_vector3: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(3), ai_mul);
  },
  multiplication_scalar_vector4: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(4), ai_mul);
  },
  multiplication_vector2_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(2), sparseI64Range(), ai_mul);
  },
  multiplication_vector3_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(3), sparseI64Range(), ai_mul);
  },
  multiplication_vector4_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(4), sparseI64Range(), ai_mul);
  },
  remainder: () => {
    return generateBinaryToI64Cases(sparseI64Range(), sparseI64Range(), ai_rem);
  },
  remainder_scalar_vector2: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(2), ai_rem);
  },
  remainder_scalar_vector3: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(3), ai_rem);
  },
  remainder_scalar_vector4: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(4), ai_rem);
  },
  remainder_vector2_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(2), sparseI64Range(), ai_rem);
  },
  remainder_vector3_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(3), sparseI64Range(), ai_rem);
  },
  remainder_vector4_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(4), sparseI64Range(), ai_rem);
  },
  subtraction: () => {
    return generateBinaryToI64Cases(sparseI64Range(), sparseI64Range(), ai_sub);
  },
  subtraction_scalar_vector2: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(2), ai_sub);
  },
  subtraction_scalar_vector3: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(3), ai_sub);
  },
  subtraction_scalar_vector4: () => {
    return generateI64VectorBinaryToVectorCases(sparseI64Range(), vectorI64Range(4), ai_sub);
  },
  subtraction_vector2_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(2), sparseI64Range(), ai_sub);
  },
  subtraction_vector3_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(3), sparseI64Range(), ai_sub);
  },
  subtraction_vector4_scalar: () => {
    return generateVectorI64BinaryToVectorCases(vectorI64Range(4), sparseI64Range(), ai_sub);
  }
});
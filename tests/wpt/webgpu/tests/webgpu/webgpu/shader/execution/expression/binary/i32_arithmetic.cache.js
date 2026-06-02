/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kValue } from '../../../../util/constants.js';import { sparseI32Range, vectorI32Range } from '../../../../util/math.js';import {
  generateBinaryToI32Cases,
  generateI32VectorBinaryToVectorCases,
  generateVectorI32BinaryToVectorCases } from
'../case.js';
import { makeCaseCache } from '../case_cache.js';

function i32_add(x, y) {
  return x + y;
}

function i32_subtract(x, y) {
  return x - y;
}

function i32_multiply(x, y) {
  return Math.imul(x, y);
}

function i32_divide_non_const(x, y) {
  if (y === 0) {
    return x;
  }
  if (x === kValue.i32.negative.min && y === -1) {
    return x;
  }
  return x / y;
}

function i32_divide_const(x, y) {
  if (y === 0) {
    return undefined;
  }
  if (x === kValue.i32.negative.min && y === -1) {
    return undefined;
  }
  return x / y;
}

function i32_remainder_non_const(x, y) {
  if (y === 0) {
    return 0;
  }
  if (x === kValue.i32.negative.min && y === -1) {
    return 0;
  }
  return x % y;
}

function i32_remainder_const(x, y) {
  if (y === 0) {
    return undefined;
  }
  if (x === kValue.i32.negative.min && y === -1) {
    return undefined;
  }
  return x % y;
}

export const d = makeCaseCache('binary/i32_arithmetic', {
  addition: () => {
    return generateBinaryToI32Cases(sparseI32Range(), sparseI32Range(), i32_add);
  },
  subtraction: () => {
    return generateBinaryToI32Cases(sparseI32Range(), sparseI32Range(), i32_subtract);
  },
  multiplication: () => {
    return generateBinaryToI32Cases(sparseI32Range(), sparseI32Range(), i32_multiply);
  },
  division_non_const: () => {
    return generateBinaryToI32Cases(sparseI32Range(), sparseI32Range(), i32_divide_non_const);
  },
  division_const: () => {
    return generateBinaryToI32Cases(sparseI32Range(), sparseI32Range(), i32_divide_const);
  },
  remainder_non_const: () => {
    return generateBinaryToI32Cases(sparseI32Range(), sparseI32Range(), i32_remainder_non_const);
  },
  remainder_const: () => {
    return generateBinaryToI32Cases(sparseI32Range(), sparseI32Range(), i32_remainder_const);
  },
  addition_scalar_vector2: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(2), i32_add);
  },
  addition_scalar_vector3: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(3), i32_add);
  },
  addition_scalar_vector4: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(4), i32_add);
  },
  addition_vector2_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(2), sparseI32Range(), i32_add);
  },
  addition_vector3_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(3), sparseI32Range(), i32_add);
  },
  addition_vector4_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(4), sparseI32Range(), i32_add);
  },
  subtraction_scalar_vector2: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(2), i32_subtract);
  },
  subtraction_scalar_vector3: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(3), i32_subtract);
  },
  subtraction_scalar_vector4: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(4), i32_subtract);
  },
  subtraction_vector2_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(2), sparseI32Range(), i32_subtract);
  },
  subtraction_vector3_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(3), sparseI32Range(), i32_subtract);
  },
  subtraction_vector4_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(4), sparseI32Range(), i32_subtract);
  },
  multiplication_scalar_vector2: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(2), i32_multiply);
  },
  multiplication_scalar_vector3: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(3), i32_multiply);
  },
  multiplication_scalar_vector4: () => {
    return generateI32VectorBinaryToVectorCases(sparseI32Range(), vectorI32Range(4), i32_multiply);
  },
  multiplication_vector2_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(2), sparseI32Range(), i32_multiply);
  },
  multiplication_vector3_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(3), sparseI32Range(), i32_multiply);
  },
  multiplication_vector4_scalar: () => {
    return generateVectorI32BinaryToVectorCases(vectorI32Range(4), sparseI32Range(), i32_multiply);
  },
  division_scalar_vector2_non_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(2),
      i32_divide_non_const
    );
  },
  division_scalar_vector3_non_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(3),
      i32_divide_non_const
    );
  },
  division_scalar_vector4_non_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(4),
      i32_divide_non_const
    );
  },
  division_vector2_scalar_non_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(2),
      sparseI32Range(),
      i32_divide_non_const
    );
  },
  division_vector3_scalar_non_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(3),
      sparseI32Range(),
      i32_divide_non_const
    );
  },
  division_vector4_scalar_non_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(4),
      sparseI32Range(),
      i32_divide_non_const
    );
  },
  division_scalar_vector2_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(2),
      i32_divide_const
    );
  },
  division_scalar_vector3_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(3),
      i32_divide_const
    );
  },
  division_scalar_vector4_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(4),
      i32_divide_const
    );
  },
  division_vector2_scalar_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(2),
      sparseI32Range(),
      i32_divide_const
    );
  },
  division_vector3_scalar_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(3),
      sparseI32Range(),
      i32_divide_const
    );
  },
  division_vector4_scalar_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(4),
      sparseI32Range(),
      i32_divide_const
    );
  },
  remainder_scalar_vector2_non_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(2),
      i32_remainder_non_const
    );
  },
  remainder_scalar_vector3_non_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(3),
      i32_remainder_non_const
    );
  },
  remainder_scalar_vector4_non_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(4),
      i32_remainder_non_const
    );
  },
  remainder_vector2_scalar_non_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(2),
      sparseI32Range(),
      i32_remainder_non_const
    );
  },
  remainder_vector3_scalar_non_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(3),
      sparseI32Range(),
      i32_remainder_non_const
    );
  },
  remainder_vector4_scalar_non_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(4),
      sparseI32Range(),
      i32_remainder_non_const
    );
  },
  remainder_scalar_vector2_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(2),
      i32_remainder_const
    );
  },
  remainder_scalar_vector3_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(3),
      i32_remainder_const
    );
  },
  remainder_scalar_vector4_const: () => {
    return generateI32VectorBinaryToVectorCases(
      sparseI32Range(),
      vectorI32Range(4),
      i32_remainder_const
    );
  },
  remainder_vector2_scalar_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(2),
      sparseI32Range(),
      i32_remainder_const
    );
  },
  remainder_vector3_scalar_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(3),
      sparseI32Range(),
      i32_remainder_const
    );
  },
  remainder_vector4_scalar_const: () => {
    return generateVectorI32BinaryToVectorCases(
      vectorI32Range(4),
      sparseI32Range(),
      i32_remainder_const
    );
  }
});
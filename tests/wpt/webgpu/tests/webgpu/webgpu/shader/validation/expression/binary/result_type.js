/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { isAbstractType, isConvertible,
  Type,

  VectorType } from
'../../../../util/conversion.js';

/**
 * @returns the resulting type of a binary arithmetic operation with the operands `lhs` and `rhs`
 */
export function resultType(args)



{
  const { lhs, rhs, canConvertScalarToVector } = args;

  if (lhs === rhs) {
    return lhs;
  }

  if (lhs instanceof VectorType && rhs instanceof VectorType) {
    // vector <op> vector
    if (lhs.width !== rhs.width) {
      return null;
    }
    const elementType = resultType({
      lhs: lhs.elementType,
      rhs: rhs.elementType,
      canConvertScalarToVector
    });
    return elementType !== null ? Type.vec(lhs.width, elementType) : null;
  }

  if (args.canConvertScalarToVector) {
    if (lhs instanceof VectorType && !(rhs instanceof VectorType)) {
      // vector <op> scalar
      const elementType = resultType({
        lhs: lhs.elementType,
        rhs,
        canConvertScalarToVector
      });
      return elementType !== null ? Type.vec(lhs.width, elementType) : null;
    }

    if (!(lhs instanceof VectorType) && rhs instanceof VectorType) {
      // scalar <op> vector
      const elementType = resultType({
        lhs,
        rhs: rhs.elementType,
        canConvertScalarToVector
      });
      return elementType !== null ? Type.vec(rhs.width, elementType) : null;
    }
  }

  if (isAbstractType(lhs) || isAbstractType(rhs)) {
    if (isConvertible(lhs, rhs)) {
      return rhs;
    }
    if (isConvertible(rhs, lhs)) {
      return lhs;
    }
  }
  return null;
}
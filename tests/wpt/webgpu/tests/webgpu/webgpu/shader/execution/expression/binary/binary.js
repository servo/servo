/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { basicExpressionBuilder,
compoundAssignmentBuilder,
abstractFloatShaderBuilder,
abstractIntShaderBuilder } from
'../expression.js';

/* @returns a ShaderBuilder that evaluates a binary operation */
export function binary(op) {
  return basicExpressionBuilder((values) => `(${values.map((v) => `(${v})`).join(op)})`);
}

/* @returns a ShaderBuilder that evaluates a compound binary operation */
export function compoundBinary(op) {
  return compoundAssignmentBuilder(op);
}

/* @returns a ShaderBuilder that evaluates a binary operation that returns AbstractFloats */
export function abstractFloatBinary(op) {
  return abstractFloatShaderBuilder((values) => `(${values.map((v) => `(${v})`).join(op)})`);
}

/* @returns a ShaderBuilder that evaluates a binary operation that returns AbstractFloats */
export function abstractIntBinary(op) {
  return abstractIntShaderBuilder((values) => `(${values.map((v) => `(${v})`).join(op)})`);
}
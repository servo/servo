/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { abstractFloatShaderBuilder, basicExpressionBuilder,
  basicExpressionWithPredeclarationBuilder } from

'../../expression.js';

/* @returns a ShaderBuilder that calls the builtin with the given name */
export function builtin(name) {
  return basicExpressionBuilder((values) => `${name}(${values.join(', ')})`);
}

/* @returns a ShaderBuilder that calls the builtin with the given name that returns AbstractFloats */
export function abstractBuiltin(name) {
  return abstractFloatShaderBuilder((values) => `${name}(${values.join(', ')})`);
}

/* @returns a ShaderBuilder that calls the builtin with the given name and has given predeclaration */
export function builtinWithPredeclaration(name, predeclaration) {
  return basicExpressionWithPredeclarationBuilder(
    (values) => `${name}(${values.join(', ')})`,
    predeclaration
  );
}
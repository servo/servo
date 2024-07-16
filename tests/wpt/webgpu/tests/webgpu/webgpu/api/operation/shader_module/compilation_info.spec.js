/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
ShaderModule CompilationInfo tests.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { assert } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

const kValidShaderSources = [
{
  valid: true,
  name: 'ascii',
  _code: `
      @vertex fn main() -> @builtin(position) vec4<f32> {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
      }`
},
{
  valid: true,
  name: 'unicode',
  _code: `
      // 頂点シェーダー 👩‍💻
      @vertex fn main() -> @builtin(position) vec4<f32> {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
      }`
}];


const kInvalidShaderSources = [
{
  valid: false,
  name: 'ascii',
  _errorLine: 4,
  _code: `
      @vertex fn main() -> @builtin(position) vec4<f32> {
        // Expected Error: unknown function 'unknown'
        return unknown(0.0, 0.0, 0.0, 1.0);
      }`
},
{
  valid: false,
  name: 'unicode',
  _errorLine: 5,
  _code: `
      // 頂点シェーダー 👩‍💻
      @vertex fn main() -> @builtin(position) vec4<f32> {
        // Expected Error: unknown function 'unknown'
        return unknown(0.0, 0.0, 0.0, 1.0);
      }`
},
{
  valid: false,
  name: 'carriage-return',
  _errorLine: 5,
  _code:
  `
      @vertex fn main() -> @builtin(position) vec4<f32> {` +
  '\r\n' +
  `
        // Expected Error: unknown function 'unknown'
        return unknown(0.0, 0.0, 0.0, 1.0);
      }`
},
{
  valid: false,
  name: 'unicode-multi-byte-characters',
  _errorLine: 1,
  // This shader is simplistic enough to always result in the same error position.
  // Generally, various backends may choose to report the error at different positions within the
  // line, so it's difficult to meaningfully validate them.
  _errorLinePos: 19,
  _code: `/*🐈🐈🐈🐈🐈🐈🐈*/?
// Expected Error: invalid character found`
}];


const kAllShaderSources = [...kValidShaderSources, ...kInvalidShaderSources];

// This is the source the sourcemap refers to.
const kOriginalSource = new Array(20).
fill(0).
map((_, i) => `original line ${i}`).
join('\n');

const kSourceMaps = {
  none: undefined,
  empty: {},
  // A valid source map. It maps `unknown` on lines 4 and line 5 to
  // `wasUnknown` from lines 20, 21 respectively
  valid: {
    version: 3,
    sources: ['myCode'],
    sourcesContent: [kOriginalSource],
    names: ['myMain', 'wasUnknown'],
    mappings: ';kBAYkCA,OACd;SAElB;gBAKOC;gBACAA'
  },
  // not a valid sourcemap
  invalid: {
    version: -123,
    notAnything: {}
  },
  // The correct format but this data is for lines 11,12 even
  // though the source only has 5 or 6 lines
  nonMatching: {
    version: 3,
    sources: ['myCode'],
    sourcesContent: [kOriginalSource],
    names: ['myMain'],
    mappings: ';;;;;;;;;;kBAYkCA,OACd;SAElB'
  }
};
const kSourceMapsKeys = keysOf(kSourceMaps);

g.test('getCompilationInfo_returns').
desc(
  `
    Test that getCompilationInfo() can be called on any ShaderModule.

    Note: sourcemaps are not used in the WebGPU API. We are only testing that
    browser that happen to use them don't fail or crash if the sourcemap is
    bad or invalid.

    - Test for both valid and invalid shader modules.
    - Test for shader modules containing only ASCII and those containing unicode characters.
    - Test that the compilation info for valid shader modules contains no errors.
    - Test that the compilation info for invalid shader modules contains at least one error.`
).
params((u) =>
u.combineWithParams(kAllShaderSources).beginSubcases().combine('sourceMapName', kSourceMapsKeys)
).
fn(async (t) => {
  const { _code, valid, sourceMapName } = t.params;

  const shaderModule = t.expectGPUError(
    'validation',
    () => {
      const sourceMap = kSourceMaps[sourceMapName];
      return t.device.createShaderModule({ code: _code, ...(sourceMap && { sourceMap }) });
    },
    !valid
  );

  const info = await shaderModule.getCompilationInfo();

  t.expect(
    info instanceof GPUCompilationInfo,
    'Expected a GPUCompilationInfo object to be returned'
  );

  // Expect that we get zero error messages from a valid shader.
  // Message types other than errors are OK.
  let errorCount = 0;
  for (const message of info.messages) {
    if (message.type === 'error') {
      errorCount++;
    }
  }
  if (valid) {
    t.expect(errorCount === 0, "Expected zero GPUCompilationMessages of type 'error'");
  } else {
    t.expect(errorCount > 0, "Expected at least one GPUCompilationMessages of type 'error'");
  }
});

g.test('line_number_and_position').
desc(
  `
    Test that line numbers reported by compilationInfo either point at an appropriate line and
    position or at 0:0, indicating an unknown position.

    Note: sourcemaps are not used in the WebGPU API. We are only testing that
    browser that happen to use them don't fail or crash if the sourcemap is
    bad or invalid.

    - Test for invalid shader modules containing containing at least one error.
    - Test for shader modules containing only ASCII and those containing unicode characters.`
).
params((u) =>
u.
combineWithParams(kInvalidShaderSources).
beginSubcases().
combine('sourceMapName', kSourceMapsKeys)
).
fn(async (t) => {
  const { _code, _errorLine, _errorLinePos, sourceMapName } = t.params;

  const shaderModule = t.expectGPUError('validation', () => {
    const sourceMap = kSourceMaps[sourceMapName];
    return t.device.createShaderModule({ code: _code, ...(sourceMap && { sourceMap }) });
  });

  const info = await shaderModule.getCompilationInfo();

  let foundAppropriateError = false;
  for (const message of info.messages) {
    if (message.type === 'error') {
      // Some backends may not be able to indicate a precise location for the error. In those
      // cases a line and position of 0 should be reported.
      // If a line is reported, it should point at the correct line (1-based).
      t.expect(
        message.lineNum === 0 === (message.linePos === 0),
        `Got message.lineNum ${message.lineNum}, .linePos ${message.linePos}, but GPUCompilationMessage should specify both or neither`
      );

      if (message.lineNum === 0) {
        foundAppropriateError = true;
        break;
      }

      if (message.lineNum === _errorLine) {
        foundAppropriateError = true;
        if (_errorLinePos !== undefined) {
          t.expect(
            message.linePos === _errorLinePos,
            `Got message.linePos ${message.linePos}, expected ${_errorLinePos}`
          );
        }
        break;
      }
    }
  }
  t.expect(
    foundAppropriateError,
    'Expected to find an error which corresponded with the erroneous line'
  );
});

g.test('offset_and_length').
desc(
  `Test that message offsets and lengths are valid and align with any reported lineNum and linePos.

     Note: sourcemaps are not used in the WebGPU API. We are only testing that
     browser that happen to use them don't fail or crash if the sourcemap is
     bad or invalid.

    - Test for valid and invalid shader modules.
    - Test for shader modules containing only ASCII and those containing unicode characters.`
).
params((u) =>
u.combineWithParams(kAllShaderSources).beginSubcases().combine('sourceMapName', kSourceMapsKeys)
).
fn(async (t) => {
  const { _code, valid, sourceMapName } = t.params;

  const shaderModule = t.expectGPUError(
    'validation',
    () => {
      const sourceMap = kSourceMaps[sourceMapName];
      return t.device.createShaderModule({ code: _code, ...(sourceMap && { sourceMap }) });
    },
    !valid
  );

  const info = await shaderModule.getCompilationInfo();

  for (const message of info.messages) {
    // Any offsets and lengths should reference valid spans of the shader code.
    t.expect(
      message.offset <= _code.length && message.offset + message.length <= _code.length,
      'message.offset and .length should be within the shader source'
    );

    // If a valid line number and position are given, the offset should point the the same
    // location in the shader source.
    if (message.lineNum !== 0 && message.linePos !== 0) {
      let lineOffset = 0;
      for (let i = 0; i < message.lineNum - 1; ++i) {
        lineOffset = _code.indexOf('\n', lineOffset);
        assert(lineOffset !== -1);
        lineOffset += 1;
      }

      const expectedOffset = lineOffset + message.linePos - 1;
      t.expect(
        message.offset === expectedOffset,
        `message.lineNum (${message.lineNum}) and .linePos (${message.linePos}) point to a different offset (${lineOffset} + ${message.linePos} - 1 = ${expectedOffset}) than .offset (${message.offset})`
      );
    }
  }
});
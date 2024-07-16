/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { unreachable } from '../../common/util/util.js';export const kDefaultVertexShaderCode = `
@vertex fn main() -> @builtin(position) vec4<f32> {
  return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
`;

export const kDefaultFragmentShaderCode = `
@fragment fn main() -> @location(0) vec4<f32>  {
  return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}`;

// MAINTENANCE_TODO(#3344): deduplicate fullscreen quad shader code.
export const kFullscreenQuadVertexShaderCode = `
  struct VertexOutput {
    @builtin(position) Position : vec4<f32>
  };

  @vertex fn main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>( 1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(-1.0,  1.0));

    var output : VertexOutput;
    output.Position = vec4<f32>(pos[VertexIndex], 0.0, 1.0);
    return output;
  }
`;

const kPlainTypeInfo = {
  i32: {
    suffix: '',
    fractionDigits: 0
  },
  u32: {
    suffix: 'u',
    fractionDigits: 0
  },
  f32: {
    suffix: '',
    fractionDigits: 4
  }
};

/**
 *
 * @param sampleType sampleType of texture format
 * @returns plain type compatible of the sampleType
 */
export function getPlainTypeInfo(sampleType) {
  switch (sampleType) {
    case 'sint':
      return 'i32';
    case 'uint':
      return 'u32';
    case 'float':
    case 'unfilterable-float':
    case 'depth':
      return 'f32';
    default:
      unreachable();
  }
}

/**
 * Build a fragment shader based on output value and types
 * e.g. write to color target 0 a `vec4<f32>(1.0, 0.0, 1.0, 1.0)` and color target 2 a `vec2<u32>(1, 2)`
 * ```
 * outputs: [
 *   {
 *     values: [1, 0, 1, 1],,
 *     plainType: 'f32',
 *     componentCount: 4,
 *   },
 *   null,
 *   {
 *     values: [1, 2],
 *     plainType: 'u32',
 *     componentCount: 2,
 *   },
 * ]
 * ```
 *
 * return:
 * ```
 * struct Outputs {
 *     @location(0) o1 : vec4<f32>,
 *     @location(2) o3 : vec2<u32>,
 * }
 * @fragment fn main() -> Outputs {
 *     return Outputs(vec4<f32>(1.0, 0.0, 1.0, 1.0), vec4<u32>(1, 2));
 * }
 * ```
 *
 * If fragDepth is given there will be an extra @builtin(frag_depth) output with the specified value assigned.
 *
 * @param outputs the shader outputs for each location attribute
 * @param fragDepth the shader outputs frag_depth value (optional)
 * @returns the fragment shader string
 */
export function getFragmentShaderCodeWithOutput(
outputs,




fragDepth = null)
{
  if (outputs.length === 0) {
    if (fragDepth) {
      return `
        @fragment fn main() -> @builtin(frag_depth) f32 {
          return ${fragDepth.value.toFixed(kPlainTypeInfo['f32'].fractionDigits)};
        }`;
    }
    return `
        @fragment fn main() {
        }`;
  }

  const resultStrings = [];
  let outputStructString = '';

  if (fragDepth) {
    resultStrings.push(`${fragDepth.value.toFixed(kPlainTypeInfo['f32'].fractionDigits)}`);
    outputStructString += `@builtin(frag_depth) depth_out: f32,\n`;
  }

  for (let i = 0; i < outputs.length; i++) {
    const o = outputs[i];
    if (o === null) {
      continue;
    }

    const plainType = o.plainType;
    const { suffix, fractionDigits } = kPlainTypeInfo[plainType];

    let outputType;
    const v = o.values.map((n) => n.toFixed(fractionDigits));
    switch (o.componentCount) {
      case 1:
        outputType = plainType;
        resultStrings.push(`${v[0]}${suffix}`);
        break;
      case 2:
        outputType = `vec2<${plainType}>`;
        resultStrings.push(`${outputType}(${v[0]}${suffix}, ${v[1]}${suffix})`);
        break;
      case 3:
        outputType = `vec3<${plainType}>`;
        resultStrings.push(`${outputType}(${v[0]}${suffix}, ${v[1]}${suffix}, ${v[2]}${suffix})`);
        break;
      case 4:
        outputType = `vec4<${plainType}>`;
        resultStrings.push(
          `${outputType}(${v[0]}${suffix}, ${v[1]}${suffix}, ${v[2]}${suffix}, ${v[3]}${suffix})`
        );
        break;
      default:
        unreachable();
    }

    outputStructString += `@location(${i}) o${i} : ${outputType},\n`;
  }

  return `
    struct Outputs {
      ${outputStructString}
    }

    @fragment fn main() -> Outputs {
        return Outputs(${resultStrings.join(',')});
    }`;
}

export const kValidShaderStages = ['compute', 'vertex', 'fragment'];



/**
 * Return a foo shader of the given stage with the given entry point
 * @param shaderStage
 * @param entryPoint
 * @returns the shader string
 */
export function getShaderWithEntryPoint(shaderStage, entryPoint) {
  let code;
  switch (shaderStage) {
    case 'compute':{
        code = `@compute @workgroup_size(1) fn ${entryPoint}() {}`;
        break;
      }
    case 'vertex':{
        code = `
      @vertex fn ${entryPoint}() -> @builtin(position) vec4<f32> {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
      }`;
        break;
      }
    case 'fragment':{
        code = `
      @fragment fn ${entryPoint}() -> @location(0) vec4<f32> {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
      }`;
        break;
      }
    case 'empty':
    default:{
        code = '';
        break;
      }
  }
  return code;
}
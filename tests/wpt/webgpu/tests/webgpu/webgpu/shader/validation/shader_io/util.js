/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ /**
 * Generate an entry point that uses an entry point IO variable.
 *
 * @param {Object} params
 * @param params.attribute The entry point IO attribute.
 * @param params.type The type to use for the entry point IO variable.
 * @param params.stage The shader stage.
 * @param params.io An "in|out" string specifying whether the entry point IO is an input or an output.
 * @param params.use_struct True to wrap the entry point IO in a struct.
 * @returns The generated shader code.
 */ export function generateShader({ attribute, type, stage, io, use_struct }) {
  let code = '';

  if (use_struct) {
    // Generate a struct that wraps the entry point IO variable.
    code += 'struct S {\n';
    code += `  ${attribute} value : ${type},\n`;
    if (stage === 'vertex' && io === 'out' && !attribute.includes('builtin(position)')) {
      // Add position builtin for vertex outputs.
      code += `  @builtin(position) position : vec4<f32>,\n`;
    }
    code += '};\n\n';
  }

  if (stage !== '') {
    // Generate the entry point attributes.
    code += `@${stage}`;
    if (stage === 'compute') {
      code += ' @workgroup_size(1)';
    }
  }

  // Generate the entry point parameter and return type.
  let param = '';
  let retType = '';
  let retVal = '';
  if (io === 'in') {
    if (use_struct) {
      param = `in : S`;
    } else {
      param = `${attribute} value : ${type}`;
    }

    // Vertex shaders must always return `@builtin(position)`.
    if (stage === 'vertex') {
      retType = `-> @builtin(position) vec4<f32>`;
      retVal = `return vec4<f32>();`;
    }
  } else if (io === 'out') {
    if (use_struct) {
      retType = '-> S';
      retVal = `return S();`;
    } else {
      retType = `-> ${attribute} ${type}`;
      retVal = `return ${type}();`;
    }
  }

  code += `
    fn main(${param}) ${retType} {
      ${retVal}
    }
  `;

  return code;
}

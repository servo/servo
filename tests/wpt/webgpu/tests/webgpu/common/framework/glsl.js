/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert, unreachable } from './util/util.js';

let glslangAttempted = false;
let glslangInstance;

export async function initGLSL() {
  if (glslangAttempted) {
    assert(glslangInstance !== undefined, 'glslang is not available');
  } else {
    glslangAttempted = true;
    const glslangPath = '../../third_party/glslang_js/lib/glslang.js';
    let glslangModule;
    try {
      glslangModule = (await import(glslangPath)).default;
    } catch (ex) {
      unreachable('glslang is not available');
    }

    const glslang = await glslangModule();
    glslangInstance = glslang;
  }
}

export function compileGLSL(glsl, shaderType, genDebug, spirvVersion) {
  assert(
    glslangInstance !== undefined,
    'GLSL compiler is not instantiated. Run `await initGLSL()` first'
  );

  return glslangInstance.compileGLSL(glsl.trimLeft(), shaderType, genDebug, spirvVersion);
}

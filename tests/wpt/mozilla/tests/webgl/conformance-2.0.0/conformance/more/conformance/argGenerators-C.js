/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/

// ArgGenerators contains argument generators for WebGL functions.
// The argument generators are used for running random tests against the WebGL
// functions.
//
// ArgGenerators is an object consisting of functionName : argGen -properties.
//
// functionName is a WebGL context function name and the argGen is an argument
// generator object that encapsulates the requirements to run
// randomly generated tests on the WebGL function.
//
// An argGen object has the following methods:
//   - setup    -- set up state for testing the GL function, returns values
//                 that need cleanup in teardown. Run once before entering a
//                 test loop.
//   - teardown -- do cleanup on setup's return values after testing is complete
//   - generate -- generate a valid set of random arguments for the GL function
//   - returnValueCleanup -- do cleanup on value returned by the tested GL function
//   - cleanup  -- do cleanup on generated arguments from generate
//   - checkArgValidity -- check if passed args are valid. Has a call signature
//                         that matches generate's return value. Returns true
//                         if args are valid, false if not.
//
//   Example test loop that demonstrates how the function args and return
//   values flow together:
//
//   var setupArgs = argGen.setup();
//   for (var i=0; i<numberOfTests; i++) {
//     var generatedArgs = argGen.generate.apply(argGen, setupArgs);
//     var validArgs = argGen.checkArgValidity.apply(argGen, generatedArgs);
//     var rv = call the GL function with generatedArgs;
//     argGen.returnValueCleanup(rv);
//     argGen.cleanup.apply(argGen, generatedArgs);
//   }
//   argGen.teardown.apply(argGen, setupArgs);
//
ArgGenerators = {

// GL functions in alphabetical order

// C

  checkFramebufferStatus : {
    generate : function() {
      return [Math.random() > 0.5 ? null : GL.createFramebuffer()];
    },
    checkArgValidity : function(fbo) {
      if (fbo != null)
        GL.bindFramebuffer(GL.FRAMEBUFFER, fbo);
      return fbo == null || GL.isFramebuffer(fbo);
    },
    cleanup : function(fbo){
      GL.bindFramebuffer(GL.FRAMEBUFFER, null);
      if (fbo != null)
        try{ GL.deleteFramebuffer(fbo); } catch(e) {}
    }
  },
  clear : {
    generate : function() { return [clearMask.random()]; },
    checkArgValidity : function(mask) { return clearMask.has(mask); }
  },
  clearColor : {
    generate : function() { return randomColor(); },
    teardown : function() { GL.clearColor(0,0,0,0); }
  },
  clearDepth : {
    generate : function() { return [Math.random()]; },
    teardown : function() { GL.clearDepth(1); }
  },
  clearStencil : {
    generate : function() { return [randomStencil()]; },
    teardown : function() { GL.clearStencil(0); }
  },
  colorMask : {
    generate : function() {
      return [randomBool(), randomBool(), randomBool(), randomBool()];
    },
    teardown : function() { GL.colorMask(true, true, true, true); }
  },
  compileShader : {}, // FIXME
  copyTexImage2D : {}, // FIXME
  copyTexSubImage2D : {}, // FIXME
  createBuffer : {
    generate : function() { return []; },
    returnValueCleanup : function(o) { GL.deleteBuffer(o); }
  },
  createFramebuffer : {
    generate : function() { return []; },
    returnValueCleanup : function(o) { GL.deleteFramebuffer(o); }
  },
  createProgram : {
    generate : function() { return []; },
    returnValueCleanup : function(o) { GL.deleteProgram(o); }
  },
  createRenderbuffer : {
    generate : function() { return []; },
    returnValueCleanup : function(o) { GL.deleteRenderbuffer(o); }
  },
  createShader : {
    generate : function() { return [shaderType.random()]; },
    checkArgValidity : function(t) { return shaderType.has(t); },
    returnValueCleanup : function(o) { GL.deleteShader(o); }
  },
  createTexture : {
    generate : function() { return []; },
    returnValueCleanup : function(o) { GL.deleteTexture(o); }
  },
  cullFace : {
    generate : function() { return [cullFace.random()]; },
    checkArgValidity : function(f) { return cullFace.has(f); },
    teardown : function() { GL.cullFace(GL.BACK); }
  }

};

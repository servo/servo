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

// B-2

  bindBuffer : {
    generate : function(buf) {
      return [bufferTarget.random(), GL.createBuffer()];
    },
    checkArgValidity : function(target, buf) {
      if (!bufferTarget.has(target))
        return false;
      GL.bindBuffer(target, buf);
      return GL.isBuffer(buf);
    },
    cleanup : function(t, buf, m) {
      GL.deleteBuffer(buf);
    }
  },
  bindFramebuffer : {
    generate : function() {
      return [GL.FRAMEBUFFER, Math.random() > 0.5 ? null : GL.createFramebuffer()];
    },
    checkArgValidity : function(target, fbo) {
      if (target != GL.FRAMEBUFFER)
        return false;
      if (fbo != null)
          GL.bindFramebuffer(target, fbo);
      return (fbo == null || GL.isFramebuffer(fbo));
    },
    cleanup : function(target, fbo) {
      GL.bindFramebuffer(target, null);
      if (fbo)
        GL.deleteFramebuffer(fbo);
    }
  },
  bindRenderbuffer : {
    generate : function() {
      return [GL.RENDERBUFFER, Math.random() > 0.5 ? null : GL.createRenderbuffer()];
    },
    checkArgValidity : function(target, rbo) {
      if (target != GL.RENDERBUFFER)
        return false;
      if (rbo != null)
        GL.bindRenderbuffer(target, rbo);
      return (rbo == null || GL.isRenderbuffer(rbo));
    },
    cleanup : function(target, rbo) {
      GL.bindRenderbuffer(target, null);
      if (rbo)
        GL.deleteRenderbuffer(rbo);
    }
  },
  bindTexture : {
    generate : function() {
      return [bindTextureTarget.random(), Math.random() > 0.5 ? null : GL.createTexture()];
    },
    checkArgValidity : function(target, o) {
      if (!bindTextureTarget.has(target))
        return false;
      if (o != null)
        GL.bindTexture(target, o);
      return (o == null || GL.isTexture(o));
    },
    cleanup : function(target, o) {
      GL.bindTexture(target, null);
      if (o)
        GL.deleteTexture(o);
    }
  },
  blendColor : {
    generate : function() { return randomColor(); },
    teardown : function() { GL.blendColor(0,0,0,0); }
  },
  blendEquation : {
    generate : function() { return [blendEquationMode.random()]; },
    checkArgValidity : function(o) { return blendEquationMode.has(o); },
    teardown : function() { GL.blendEquation(GL.FUNC_ADD); }
  },
  blendEquationSeparate : {
    generate : function() {
      return [blendEquationMode.random(), blendEquationMode.random()];
    },
    checkArgValidity : function(o,p) {
      return blendEquationMode.has(o) && blendEquationMode.has(p);
    },
    teardown : function() { GL.blendEquationSeparate(GL.FUNC_ADD, GL.FUNC_ADD); }
  },
  blendFunc : {
    generate : function() {
      return [blendFuncSfactor.random(), blendFuncDfactor.random()];
    },
    checkArgValidity : function(s,d) {
      return blendFuncSfactor.has(s) && blendFuncDfactor.has(d);
    },
    teardown : function() { GL.blendFunc(GL.ONE, GL.ZERO); }
  },
  blendFuncSeparate : {
    generate : function() {
      return [blendFuncSfactor.random(), blendFuncDfactor.random(),
              blendFuncSfactor.random(), blendFuncDfactor.random()];
    },
    checkArgValidity : function(s,d,as,ad) {
      return blendFuncSfactor.has(s) && blendFuncDfactor.has(d) &&
              blendFuncSfactor.has(as) && blendFuncDfactor.has(ad) ;
    },
    teardown : function() {
      GL.blendFuncSeparate(GL.ONE, GL.ZERO, GL.ONE, GL.ZERO);
    }
  }

};

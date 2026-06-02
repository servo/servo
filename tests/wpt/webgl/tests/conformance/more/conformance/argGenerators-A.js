/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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

// A

  activeTexture : {
    generate : function() { return [textureUnit.random()]; },
    checkArgValidity : function(t) { return textureUnit.has(t); },
    teardown : function() { GL.activeTexture(GL.TEXTURE0); }
  },
  attachShader : {
    generate : function() {
      var p = GL.createProgram();
      var sh = GL.createShader(shaderType.random());
      return [p, sh];
    },
    checkArgValidity : function(p, sh) {
      return GL.isProgram(p) && GL.isShader(sh) && !GL.getAttachedShaders(p).has(sh);
    },
    cleanup : function(p, sh) {
      try {GL.detachShader(p,sh);} catch(e) {}
      try {GL.deleteProgram(p);} catch(e) {}
      try {GL.deleteShader(sh);} catch(e) {}
    }
  }

};

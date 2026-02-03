GLSL optimizer
==============

> :warning: As of mid-2016, the project is unlikely to have any significant developments. At Unity we are moving to a different
shader compilation pipeline, with glsl-optimizer is not used. So from my side there won't be significant work done on it. :warning:


A C++ library that takes GLSL shaders, does some GPU-independent optimizations on them
and outputs GLSL or Metal source back. Optimizations are function inlining, dead code removal, copy propagation,
constant folding, constant propagation, arithmetic optimizations and so on.

Apparently quite a few mobile platforms are pretty bad at optimizing shaders; and
unfortunately they *also* lack offline shader compilers. So using a GLSL optimizer offline
before can make the shader run much faster on a platform like that. See performance numbers
in [this blog post](http://aras-p.info/blog/2010/09/29/glsl-optimizer/).

Even for drivers that have decent shader optimization, GLSL optimizer could be useful to just strip away
dead code, make shaders smaller and do uniform/input reflection offline.

Almost all actual code is [Mesa 3D's GLSL](http://cgit.freedesktop.org/mesa/mesa/log/)
compiler; all this library does is spits out optimized GLSL or Metal back, and adds GLES type precision
handling to the optimizer.

This GLSL optimizer is made for [Unity's](http://unity3d.com/) purposes and is built-in
starting with Unity 3.0.

GLSL Optimizer is licensed according to the terms of the MIT license.

See [change log here](Changelog.md).


Usage
-----

Visual Studio 2010 (Windows, x86/x64) and Xcode 5+ (Mac, i386) project files for a static
library are provided in `projects/vs2010/glsl_optimizer.sln` and `projects/xcode5/glsl_optimizer_lib`
respectively.

> Note: only the VS and Xcode project files are maintained and should work at any time.
> There's also a cmake and gyp build system for Linux et al., and some stuff in contrib folder -
> all that may or might not work.

For Linux you can use cmake. Just type "cmake . && make" in the root directory.
This will build the optimizer library and some executable binaries.

Interface for the library is `src/glsl/glsl_optimizer.h`. General usage is:
 
	ctx = glslopt_initialize(targetVersion);
	for (lots of shaders) {
		shader = glslopt_optimize (ctx, shaderType, shaderSource, options);
		if (glslopt_get_status (shader)) {
			newSource = glslopt_get_output (shader);
		} else {
			errorLog = glslopt_get_log (shader);
		}
		glslopt_shader_delete (shader);
	}
	glslopt_cleanup (ctx);


Tests
-----

There's a testing suite for catching regressions, see `tests` folder. In VS, build
and run `glsl_optimizer_tests` project; in Xcode use `projects/xcode5/glsl_optimizer_tests`
project. The test executable requires path to the `tests` folder as an argument.

Each test comes as three text files; input, expected IR dump and expected optimized
GLSL dump. GLES3 tests are also converted into Metal.

If you're making changes to the project and want pull requests accepted easier, I'd
appreciate if there would be no test suite regressions. If you are implementing a
feature, it would be cool to add tests to cover it as well!


Notes
-----

* GLSL versions 1.10 and 1.20 are supported. 1.10 is the default, use #version 120 to specify 
1.20. Higher GLSL versions might work, but aren't tested now.
* GLSL ES versions 1.00 and 3.00 are supported.


Dev Notes
---------

Pulling Mesa upstream:

    git fetch upstream
    git merge upstream/master
    sh removeDeletedByUs.sh
    # inspect files, git rm unneeded ones, fix conflicts etc.
    # git commit
    
Rebuilding flex/bison parsers:

* When .y/.l files are changed, the parsers are *not* rebuilt automatically,
* Run ./generateParsers.sh to do that. You'll need bison & flex (on Mac, do "Install Command Line Tools" from Xcode)
* I use bison 2.3 and flex 2.5.35 (in OS X 10.8/10.9)


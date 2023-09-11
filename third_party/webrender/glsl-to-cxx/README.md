A GLSL to C++ translator.

Translates GLSL to vectorized C++. Intended for use with WebRender software backend.

Architecture
------------
GLSL code is parsed by the glsl crate. In hir.rs we traverse the resulting AST
and build a higher level representation by doing type checking and name
resolution. The resulting hir tree is traversed by lib.rs to output C++ code.

The generated C++ code is 4x wider then the original glsl. i.e. a glsl 'float'
becomes a C++ 'Float' which is represented by a xmm register (a vector of 4 floats).
Likewise, a vec4 becomes a struct of 4 'Float's for a total of 4 xmm registers and
16 floating point values.

Vector branching is flattened to non-branching code that unconditionally runs both
sides of the branch and combines the results with a mask based on the condition.

The compiler also supports scalarization. Values that are known to be the same
across all vector lanes are translated to scalars instead of vectors. Branches on
scalars are translated as actual branches.

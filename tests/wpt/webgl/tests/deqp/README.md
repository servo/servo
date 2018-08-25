DEQP tests for WebGL
===========================================

1. Running
Tests can be run as part of the WebGL Conformance suite or individually
by navigating to one of pages in functional/gles3 or in data/gles(2|3)/shaders/

2. Filtering
One can limit the tests to run with a 'filter' query. For example:

functional/gles3/textureformat.html?filter=2d

will executed only the tests with '2d' in the test name.
Filter query accepts a regular expression.

3. Compiling.
The tests have been annotated for closure and can be compiled with run-closure script.

4. Implementation notes.
Tests use a minimal subset of google closure library for dependency management.
The closure compiler is used solely for error checking. The compiler output is discarded.

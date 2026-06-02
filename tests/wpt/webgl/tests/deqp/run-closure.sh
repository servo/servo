#!/bin/sh

# Helper script for properly invoking the closure compiler in order to
# type check the ported dEQP tests.
#
# Assumes the Closure compiler:
#   https://github.com/google/closure-compiler
# is installed side-by-side with the WebGL repository, for example:
#
#  WebGL/
#    doc/
#    extensions/
#    sdk/
#    ...
#  closure/
#    compiler.jar
#
# The externs.zip file inside the closure compiler needs to be modified
# to support WebGL2.
# and that the shell is cd'd into the directory containing this
# script.
#

: ${JAVA:=java}

$JAVA -jar ../../../../closure/compiler.jar --compilation_level ADVANCED_OPTIMIZATIONS --warning_level VERBOSE --externs compiler_additional_extern.js --js functional/**.js framework/**.js modules/**.js --js_output_file /dev/null --js ../closure-library/closure/**.js

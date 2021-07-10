#!/usr/bin/env python

#
# Copyright (c) 2015 The Khronos Group Inc.
#
# Permission is hereby granted, free of charge, to any person obtaining a
# copy of this software and/or associated documentation files (the
# "Materials"), to deal in the Materials without restriction, including
# without limitation the rights to use, copy, modify, merge, publish,
# distribute, sublicense, and/or sell copies of the Materials, and to
# permit persons to whom the Materials are furnished to do so, subject to
# the following conditions:
#
# The above copyright notice and this permission notice shall be included
# in all copies or substantial portions of the Materials.
#
# THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
# MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
# IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
# CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
# TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
# MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
#

# Author: Mobica LTD

import sys
import re
import os
import subprocess
import threading
from sys import stdout, stderr, argv

# Running this script
# 1. To rebuild all dependencies:
# $ build.py deps
# 2. To build all targets without rebuilding dependencies
# $ build.py build
# 3. To build a single target without rebuilding dependencies
# $ build.py build <target>
# See the table below for available targets
# 4. To rebuild all dependencies and targets
# $ build.py
# 5. To build dependencies for a single target
# $ build.py deps <target>
# 6. To build dependencies for and compile a single target
# $ build.py <target>

# List of targets (short target name, closure namespace)
targets = {
    'textureformat': 'functional.gles3.es3fTextureFormatTests',
    'fboCompletenessTests': 'functional.gles3.es3fFboCompletenessTests',
    'fbomultisampletests': 'functional.gles3.es3fFboMultisampleTests',
    'fbostencilbuffertests': 'functional.gles3.es3fFboStencilbufferTests',
    'fragmentoutput': 'functional.gles3.es3fFragmentOutputTests',
    'framebufferblittests': 'functional.gles3.es3fFramebufferBlitTests',
    'instancedrenderingtests': 'functional.gles3.es3fInstancedRenderingTests',
    'pixelBufferObjectTest': 'functional.gles3.es3fPixelBufferObjectTest',
    'primitiverestarttests': 'functional.gles3.es3fPrimitiveRestartTests',
    'samplerobjecttests': 'functional.gles3.es3fSamplerObjectTests',
    'transformFeedbackTests': 'functional.gles3.es3fTransformFeedbackTests',
    'uniformapi': 'functional.gles3.es3fUniformApiTests',
    'uniformbuffers': 'functional.gles3.es3fUniformBlockTests',
    'vertexarrays': 'functional.gles3.es3fVertexArrayTests',
    'shaderlibrary': 'modules.shared.glsShaderLibrary',
    'negativebuffer': 'functional.gles3.es3fNegativeBufferApiTests',
    'sglrReferenceContextTest': 'framework.opengl.simplereference.sglrReferenceContextTest',
    'lifetime': 'functional.gles3.es3fLifetimeTests',
    'draw': 'functional.gles3.es3fDrawTests',
    'attriblocation': 'functional.gles3.es3fAttribLocationTests',
    'textureShadowTests': 'functional.gles3.es3fTextureShadowTests',
    'texturewrap': 'functional.gles3.es3fTextureWrapTests',
    'negativetextureapi': 'functional.gles3.es3fNegativeTextureApiTests',
    'multisample': 'functional.gles3.es3fMultisampleTests',
    'negativefragmentapi': 'functional.gles3.es3fNegativeFragmentApiTests',
    'negativevertexarrayapi': 'functional.gles3.es3fNegativeVertexArrayApiTests',
    'negativestateapi' : 'functional.gles3.es3fNegativeStateApiTests',
    'negativeshaderapi' : 'functional.gles3.es3fNegativeShaderApiTests',
    'rasterizerdiscard' : 'functional.gles3.es3fRasterizerDiscardTests',
    'buffercopy' : 'functional.gles3.es3fBufferCopyTests',
    'shaderindexing' : 'functional.gles3.es3fShaderIndexingTests',
    'shaderloop' : 'functional.gles3.es3fShaderLoopTests',
    'shaderstruct' : 'functional.gles3.es3fShaderStructTests',
    'shaderswitch' : 'functional.gles3.es3fShaderSwitchTests',
    'fborender' : 'functional.gles3.es3fFboRenderTest',
    'shaderderivate' : 'functional.gles3.es3fShaderDerivateTests',
    'builtinprecision' : 'functional.gles3.es3fBuiltinPrecisionTests',
    'shaderbuiltinvar' : 'functional.gles3.es3fShaderBuiltinVarTests',
    'texturefiltering' : 'functional.gles3.es3fTextureFilteringTests',
    'fbocolor' : 'functional.gles3.es3fFboColorbufferTests',
    'fragdepth' : 'functional.gles3.es3fFragDepthTests',
    'shaderop' : 'functional.gles3.es3fShaderOperatorTests',
    'vao' : 'functional.gles3.es3fVertexArrayObjectTests',
    'clip' : 'functional.gles3.es3fClippingTests',
    'inv' : 'functional.gles3.es3fFboInvalidateTests',
    'defvertattr' : 'functional.gles3.es3fDefaultVertexAttributeTests',
    'occlusion' : 'functional.gles3.es3fOcclusionQueryTests',
    'shaderapi' : 'functional.gles3.es3fShaderApiTests',
    'shaderpackingfunction' : 'functional.gles3.es3fShaderPackingFunctionTests',
    'shadercommonfunction' : 'functional.gles3.es3fShaderCommonFunctionTests',
    'shadermatrix' : 'functional.gles3.es3fShaderMatrixTest',
    'shaderprecision' : 'functional.gles3.es3fShaderPrecisionTests',
    'bstate': 'functional.gles3.es3fBooleanStateQuery',
    'shaderstate': 'functional.gles3.es3fShaderStateQueryTests',
    'fbostate' : 'functional.gles3.es3fFboStateQueryTests',
    'rbostate' : 'functional.gles3.es3fRboStateQueryTests',
    'bufferstate' : 'functional.gles3.es3fBufferObjectQueryTests',
    'samplerstate' : 'functional.gles3.es3fSamplerStateQueryTests',
    'texstate' : 'functional.gles3.es3fTextureStateQuery',
    'internalformatstate' : 'functional.gles3.es3fInternalFormatQueryTests',
    'texturespecification' : 'functional.gles3.es3fTextureSpecificationTests',
    'shadertexturefunction' : 'functional.gles3.es3fShaderTextureFunctionTests',
    'sync' : 'functional.gles3.es3fSyncTests',
    'readpixel' : 'functional.gles3.es3fReadPixelTests',
    'stringquery' : 'functional.gles3.es3fStringQueryTests',
    'indexedstate' : 'functional.gles3.es3fIndexedStateQueryTests',
    'integerstate' : 'functional.gles3.es3fIntegerStateQueryTests',
    'floatstate' : 'functional.gles3.es3fFloatStateQueryTests'
}

total_errors = 0
total_warnings = 0

results = dict()

def dep_filename(target):
    return target + '.dep'

def compiled_filename(target):
    return target + '.compiled'

def write_to_file(outfile, cmdLine, redirect_stderr):
    stderr = None
    if redirect_stderr:
        stderr = subprocess.STDOUT

    with open(outfile, "w") as out_file:
            proc = subprocess.Popen(cmdLine, shell=True, stdout=subprocess.PIPE, stderr=stderr)
            while proc.poll() is None:
                line = proc.stdout.readline()
                out_file.write(line)

            out_file.flush()
            proc.wait()

def read_file(file_path):
    #File exist
    if not file_exists(file_path):
        sys.exit(2)

    fo = open(file_path)
    lines = fo.read()
    fo.close()
    return lines

def file_exists(file_path):
    if not os.path.exists:
        print "The file " + file_name + " doesn't exists"
        return False
    return True

def build_deps(target, namespace):
    cmdLine = 'python ../closure-library/closure/bin/build/closurebuilder.py --root=../closure-library --root=. --namespace=' + namespace
    print cmdLine
    write_to_file(dep_filename(target), cmdLine, False)

def build_all_deps():
    for target in targets.keys():
        build_deps(target, targets[target])

def buildDepsFile():
    # the parameter "--root_with_prefix" is the relative path from the file goog/base.js to the root of the .js files we
    # are working on.
    cmdBuildDeps = 'python ../closure-library/closure/bin/build/depswriter.py --root_with_prefix=". ../../../deqp" > deqp-deps.js'

    # Calls the python program that generates the google closure dependencies
    # write_to_file('deqp-deps.js', cmdBuildDeps, False)
    proc = subprocess.Popen(cmdBuildDeps, shell=True, stdout=subprocess.PIPE, stderr=None)
    proc.wait()

def build_target(target, namespace):
    global total_errors
    global total_warnings
    deps = read_file(dep_filename(target))
    cmdLine = 'java -jar compiler.jar --compilation_level ADVANCED_OPTIMIZATIONS --warning_level VERBOSE --jscomp_warning undefinedVars --externs compiler_additional_extern.js'
    for dep in deps.split('\n'):
        dep = dep.strip()
        if len(dep) > 0:
            cmdLine += ' --js ' + dep
    cmdLine += ' --closure_entry_point=' + namespace
    print cmdLine
    filename = compiled_filename(target)
    write_to_file(filename, cmdLine, True)
    compiled = read_file(filename)
    result = re.search(r'(\d*)\s*error\(s\),\s*(\d*)\s*warning\(s\)', compiled)
    errors = 0
    warnings = 0
    if result:
        print target + ': ' + result.group(0)
        errors = int(result.group(1))
        warnings = int(result.group(2))
        total_errors += errors
        total_warnings += warnings
    results[target] = [errors, warnings]

def build_all_targets():
    for target in targets.keys():
        build_target(target, targets[target])

def format_target(target):
    deps = read_file(dep_filename(target))
    fixjsstyle = 'fixjsstyle.py'
    reformat = 'reformatting_tool.py'
    for dep in deps.split('\n'):
        dep = dep.strip()
        if len(dep) > 0 and not re.search('closure-library.*base\.js', dep):
            print fixjsstyle + ' ' + dep
            subprocess.call(['python', fixjsstyle, dep])
            print reformat + ' -f ' + dep
            subprocess.call(['python', reformat, '-f', dep])

def format_all_targets():
    for target in targets.keys():
        format_target(target)

def pass_or_fail():
    if total_errors + total_warnings == 0:
        print "Passed"
    elif len(results) > 1: #display the summary only when building more than one target
        passed = [k for k, v in results.iteritems() if v[0] + v[1] == 0]
        failed = dict((k, v) for k, v in results.iteritems() if v[0] + v[1] != 0)
        print "\nBuild Summary:"
        # Print first the tests that passed
        for target in passed:
            print "{0:>30}\tPassed".format(target+":")

        # Print tests that failed. Fixed-width to improve readability
        for target in failed:
            errors = failed[target][0]
            warnings = failed[target][1]
            print "{0:>30}\tErrors: {1:4}\tWarnings: {2:4}".format(target+":", errors, warnings)
        print "Compilation failed: {} error(s), {} warning(s).".format(total_errors, total_warnings)

def main(argv):
    if len(argv) == 0:
        build_all_deps()
        build_all_targets()
        buildDepsFile()
        pass_or_fail()
    elif (argv[0] == 'deps'):
        if len(argv) == 2:
            target = argv[1]
            build_deps(target, targets[target])
        else:
            build_all_deps()
    elif (argv[0] == 'format'):
        if len(argv) == 2:
            target = argv[1]
            format_target(target)
        else:
            format_all_targets()
    elif (argv[0] == 'build'):
        if len(argv) == 2:
            target = argv[1]
            build_target(target, targets[target])
        else:
            build_all_targets()
        pass_or_fail()
    elif (argv[0] == 'depfile'):
        buildDepsFile()
    elif (argv[0] == 'list'):
        print "List of available targets:"
        for target in targets.keys():
            print "\t{}".format(target)
    else:
        target = argv[0]
        build_deps(target, targets[target])
        build_target(target, targets[target])
        pass_or_fail()

if __name__ == '__main__':
    main(sys.argv[1:])

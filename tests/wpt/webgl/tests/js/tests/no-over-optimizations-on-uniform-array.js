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
NoOverOptimizeOnUniformArrayTester = (function(){

var vshader = [
    "attribute vec4 a_position;",
    "void main()",
    "{",
    "    gl_Position = a_position;",
    "}"
].join('\n');

var fshader_max = [
    "precision mediump float;",
    "uniform vec4 colora[$(maxUniformVectors)];",
    "void main()",
    "{",
    "    gl_FragColor = vec4(colora[$(usedUniformVector)]);",
    "}"
].join('\n');

var fshader_max_ab_ab = [
    "precision mediump float;",
    "uniform vec4 $(decl1);",
    "uniform vec4 $(decl2);",
    "void main()",
    "{",
    "gl_FragColor = vec4($(usage1) + $(usage2));",
    "}"
].join('\n');

// MaxInt32 is 2^32-1. We need +1 of that to test overflow conditions
var MaxInt32PlusOne = 4294967296;

function setupTests(gl) {
    var tests = [];
    var maxUniformVectors = gl.getParameter(gl.MAX_FRAGMENT_UNIFORM_VECTORS);

    // This test is to test drivers the have bugs related to optimizing
    // an array of uniforms when only 1 of those uniforms is used.
    tests.push({
        desc: "using last element",
        maxUniformVectors: maxUniformVectors,
        usedUniformVector: maxUniformVectors - 1,
        shader: "fshader-max",
        color: [0, 1, 0, 1],
        arrayName: "colora",
        extraName: "colorb",
    });
    tests.push({
        desc: "using first element",
        maxUniformVectors: maxUniformVectors,
        usedUniformVector: 0,
        shader: "fshader-max",
        color: [0, 1, 0, 1],
        arrayName: "colora",
        extraName: "colorb",
    });

    // Generate test shaders. We're trying to force the driver to
    // overflow from 1 array into the next if it optimizes. So for example if it was C
    //
    //   int big[4];
    //   int little[1];
    //   big[5] = 124;
    //
    // Would end up setting little[0] instead of big. Some drivers optimize
    // where if you only use say 'big[3]' it will actually only allocate just 1 element
    // for big.
    //
    // But, some drivers have a bug where the fact that they optimized big to 1 element
    // does not get passed down to glUniform so when setting the uniform 'big[3]' they
    // overwrite memory.
    //
    // If the driver crashes, yea. We found a bug. We can blacklist the driver.
    // Otherwise we try various combinations so that setting 'little[0]' first
    // and then setting all elements of 'big' we hope it will overwrite 'little[0]'
    // which will show the bug and again we can blacklist the driver.
    //
    // We don't know how the driver will order, in memory, the various uniforms
    // or for that matter we don't even know if they will be contiguous in memory
    // but to hopefully expose any bugs we try various combinations.
    //
    //    It could be the compiler orders uniforms alphabetically.
    //    It could be it orders them in order of declaration.
    //    It could be it orders them in order of usage.
    //
    // We also test using only first element of big or just the last element of big.
    //
    for (var nameOrder = 0; nameOrder < 2; ++nameOrder) {
        var name1 = nameOrder ? "colora" : "colorb";
        var name2 = nameOrder ? "colorb" : "colora";
        for (var last = 0; last < 2; ++last) {
            var usedUniformVector = last ? maxUniformVectors - 2 : 0;
            for (var declOrder = 0; declOrder < 2; ++declOrder) {
                var bigName    = declOrder ? name1 : name2;
                var littleName = declOrder ? name2 : name1;
                var decl1 = bigName + "[" + (maxUniformVectors - 1) + "]";
                var decl2 = littleName + "[1]";
                if (declOrder) {
                    var t = decl1;
                    decl1 = decl2;
                    decl2 = t;
                }
                for (var usageOrder = 0; usageOrder < 2; ++usageOrder) {
                    var usage1 = bigName + "[" + usedUniformVector + "]";
                    var usage2 = littleName + "[0]";
                    if (usageOrder) {
                        var t = usage1;
                        usage1 = usage2;
                        usage2 = t;
                    }
                    var fSrc = wtu.replaceParams(fshader_max_ab_ab, {
                        decl1: decl1,
                        decl2: decl2,
                        usage1: usage1,
                        usage2: usage2,
                    });
                    var desc = "testing: " + name1 + ":" + name2 + " using " + (last ? "last" : "first") +
                        " creating uniforms " + decl1 + " " + decl2 + " and accessing " + usage1 + " " + usage2;
                    tests.push({
                        desc: desc,
                        maxUniformVectors: maxUniformVectors - 1,
                        usedUniformVector: usedUniformVector,
                        source: fSrc,
                        color: [0, 0, 0, 1],
                        arrayName: bigName,
                        extraName: littleName,
                    });
                }
            }
        }
    }
    return tests;
};

function testUniformOptimizationIssues(test) {
    debug("");
    debug(test.desc);
    var fshader = test.source;
    if (!fshader) {
        fshader = wtu.replaceParams(fshader_max, test);
    }

    var consoleElem = document.getElementById("console");
    wtu.addShaderSource(
        consoleElem, "vertex shader", vshader);
    wtu.addShaderSource(
        consoleElem, "fragment shader", fshader);

    var program = wtu.loadProgram(gl, vshader, fshader);
    gl.useProgram(program);

    var colorbLocation = gl.getUniformLocation(program, test.extraName + "[0]");
    if (colorbLocation) {
        gl.uniform4fv(colorbLocation, [0, 1, 0, 0]);
    }

    // Ensure that requesting an array uniform past MaxInt32PlusOne returns no uniform
    var nameMaxInt32PlusOne = test.arrayName + "[" + (test.usedUniformVector + MaxInt32PlusOne) + "]";
    assertMsg(gl.getUniformLocation(program, nameMaxInt32PlusOne) === null,
        "Requesting " + nameMaxInt32PlusOne + " uniform should return a null uniform location");

    // Set just the used uniform
    var name = test.arrayName + "[" + test.usedUniformVector + "]";
    var uniformLocation = gl.getUniformLocation(program, name);
    gl.uniform4fv(uniformLocation, test.color);
    wtu.setupIndexedQuad(gl, 1);
    wtu.clearAndDrawIndexedQuad(gl, 1);
    wtu.checkCanvas(gl, [0, 255, 0, 255], "should be green");

    // Set all the unused uniforms
    var locations = [];
    var allRequiredUniformLocationsQueryable = true;
    for (var ii = 0; ii < test.maxUniformVectors; ++ii) {
        var name = test.arrayName + "[" + ii + "]";
        var uniformLocation = gl.getUniformLocation(program, name);
        locations.push(uniformLocation);
        if (ii == test.usedUniformVector) {
            continue;
        }
        // Locations > usedUnformVector may not exist.
        // Locations <= usedUniformVector MUST exist.
        if (ii <= test.usedUniformVector && (uniformLocation === undefined || uniformLocation === null)) {
            allRequiredUniformLocationsQueryable = false;
        }
        gl.uniform4fv(uniformLocation, [1, 0, 0, 1]);
    }
    if (allRequiredUniformLocationsQueryable) {
        testPassed("allRequiredUniformLocationsQueryable is true.");
    }
    else {
        testFailed("allRequiredUniformLocationsQueryable should be true. Was false.");
    }
    var positionLoc = gl.getAttribLocation(program, "a_position");
    wtu.setupIndexedQuad(gl, 1, positionLoc);
    wtu.clearAndDrawIndexedQuad(gl, 1);
    wtu.checkCanvas(gl, [0, 255, 0, 255], "should be green");

    // Check we can read & write each uniform.
    // Note: uniforms past test.usedUniformVector might not exist.
    for (var ii = 0; ii < test.maxUniformVectors; ++ii) {
        gl.uniform4fv(locations[ii], [ii + 4, ii + 2, ii + 3, ii + 1]);
    }

    var kEpsilon = 0.01;
    var isSame = function(v1, v2) {
        return Math.abs(v1 - v2) < kEpsilon;
    };

    for (var ii = 0; ii < test.maxUniformVectors; ++ii) {
        var location = locations[ii];
        if (location) {
            var value = gl.getUniform(program, locations[ii]);
            if (!isSame(value[0], ii + 4) ||
                !isSame(value[1], ii + 2) ||
                !isSame(value[2], ii + 3) ||
                !isSame(value[3], ii + 1)) {
                testFailed("location: " + ii + " was not correct value");
                break;
            }
        }
    }
}

function runOneTest(gl, test) {
    testUniformOptimizationIssues(test);
};

function runTests(gl, tests) {
    debug("");
    debug("Test drivers don't over optimize unused array elements");

    for (var ii = 0; ii < tests.length; ++ii) {
        runOneTest(gl, tests[ii]);
    }
};

return {
    setupTests : setupTests,
    runTests : runTests
};

}());

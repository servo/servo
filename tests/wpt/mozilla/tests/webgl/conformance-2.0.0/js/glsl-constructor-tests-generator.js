/*
** Copyright (c) 2014 The Khronos Group Inc.
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


var GLSLConstructorTestsGenerator = (function() {

var wtu = WebGLTestUtils;

// Shader code templates
var constructorVertexTemplate = [
  "attribute vec4 vPosition;",

  "precision mediump int;",
  "precision mediump float;",

  // Colors used to signal correctness of component values comparison
  "const vec4 green = vec4(0.0, 1.0, 0.0, 1.0);",
  "const vec4 red   = vec4(1.0, 0.0, 0.0, 1.0);",

  // Error bound used in comparison of floating point values
  "$(errorBound)",

  "varying vec4 vColor;",

  "void main() {",
  "  $(argsList)",

  "  $(type) v = $(type)($(argsConstr));",

  "  if ($(checkCompVals))",
  "    vColor = green;",
  "  else",
  "    vColor = red;",

  "  gl_Position = vPosition;",
  "}"
].join("\n");


var passThroughColorFragmentShader = [
  "precision mediump float;",

  "varying vec4 vColor;",

  "void main() {",
  "    gl_FragColor = vColor;",
  "}"
].join('\n');


var constructorFragmentTemplate = [
  "precision mediump int;",
  "precision mediump float;",

  // Colors used to signal correctness of component values comparison
  "const vec4 green = vec4(0.0, 1.0, 0.0, 1.0); ",
  "const vec4 red   = vec4(1.0, 0.0, 0.0, 1.0); ",

  // Error bound used in comparison of floating point values
  "$(errorBound)",

  "void main() {",
  "  $(argsList)",

  "  $(type) v = $(type)($(argsConstr));",

  "  if ($(checkCompVals))",
  "    gl_FragColor = green;",
  "  else",
  "    gl_FragColor = red;",
  "}"
].join("\n");


// Coding of the different argument types
// s  : scalar
// v2 : vec2
// v3 : vec3
// v4 : vec4
// m2 : mat2
// m3 : mat3
// m4 : mat4

// Returns the dimensions of the type
// Count of columns, count of rows
function getTypeCodeDimensions(typeCode) {
  switch (typeCode) {
    case "s":  return [1, 1];
    case "v2": return [1, 2];
    case "v3": return [1, 3];
    case "v4": return [1, 4];
    case "m2": return [2, 2];
    case "m3": return [3, 3];
    case "m4": return [4, 4];

    default:
      wtu.error("GLSLConstructorTestsGenerator.getTypeCodeDimensions(), unknown type code");
      debugger;
  }
};


// Returns the component count for the type code
function getTypeCodeComponentCount(typeCode) {
  var dim = getTypeCodeDimensions(typeCode);

  return dim[0] * dim[1];
}


// Returns glsl name of type code
function getGLSLBaseTypeName(typeCode) {
  switch(typeCode) {
    case "s":  return "";
    case "v2": return "vec2";
    case "v3": return "vec3";
    case "v4": return "vec4";
    case "m2": return "mat2";
    case "m3": return "mat3";
    case "m4": return "mat4";

    default:
      wtu.error("GLSLConstructorTestsGenerator.getGLSLBaseTypeName(), unknown type code");
      debugger;
  }
}


// Returns the scalar glsl type name related to the structured type
function getGLSLScalarType(targetType) {
  switch(targetType[0]) {
    case 'i': return "int";
    case 'b': return "bool";

    case 'v':
    case 'm':
      return "float";

    default:
      wtu.error("GLSLConstructorTestsGenerator.getGLSLScalarType(), unknown target type");
      debugger;
  }
}


// Returns the scalar prefix for the associated scalar type
function getGLSLScalarPrefix(targetType) {
  switch(targetType[0]) {
    case 'i':
    case 'b':
      return targetType[0];

    case 'v':
    case 'm':
      return '';

    default:
      wtu.error("GLSLConstructorTestsGenerator.getGLSLScalarPrefix(), unknown target type");
      debugger;
  }
}


// Returns the type for a specified target type and argument type code
function getGLSLArgumentType(typeCode, targetType) {
  var baseType = getGLSLBaseTypeName(typeCode);
  if (baseType !== "") {
    if (typeCode[0] === "v") {
      // Vectors come in different flavours
      return getGLSLScalarPrefix(targetType) + baseType;
    }
    else
      return baseType;
  }
  else
    return getGLSLScalarType(targetType);
}


// Returns the glsl type of the argument components
function getGLSLArgumentComponentType(argTypeCode, targetType) {
  var scalarType;

  if (argTypeCode[0] === "m") {
    // Matrices are always floats
    scalarType = "float";
  }
  else
    scalarType = getGLSLScalarType(targetType);

  return scalarType;
}


function getGLSLColumnSize(targetType) {
  colSize = parseInt(targetType.slice(-1));

  if (!isNaN(colSize))
    return colSize;

  wtu.error("GLSLConstructorTestsGenerator.getGLSLColumnSize(), invalid target type");
    debugger;
}


// Returns correct string representation of scalar value
function getScalarTypeValStr(val, scalarType) {
  if (val == null)
    debugger;

  switch (scalarType) {
    case "float": return val.toFixed(1);
    case "int":   return val;
    case "bool":  return (val === 0) ? "false" : "true";

    default:
      wtu.error("GLSLConstructorTestsGenerator.getScalarTypeValStr(), unknown scalar type");
      debugger;
  }
}


// Returns true if the glsl type name is a matrix
function isGLSLTypeMatrix(type) {
  return (type.indexOf("mat") !== -1);
}


// Returns true if the glsl type name is a vector
function isGLSLTypeVector(type) {
  return (type.indexOf("vec") !== -1);
}


// Returns the count of components
function getGLSLTypeComponentCount(type) {
  var colSize = getGLSLColumnSize(type);

  if (isGLSLTypeMatrix(type))
    return colSize * colSize;
  else
    return colSize;
}


// Returns the constructor expression with the components set to a sequence of scalar values
// Like vec3(1.0, 2.0, 3.0)
function getComponentSequenceConstructorExpression(typeCode, firstCompValue, targetType) {
  var scalarType = getGLSLArgumentComponentType(typeCode, targetType);

  if (typeCode === "s") {
    // Scalar
    return getScalarTypeValStr(firstCompValue, scalarType) + ";";
  }
  else {
    // Structured typeargTypeCode[0] === "m"
    compCount = getTypeCodeComponentCount(typeCode);
    var constrExpParts = new Array(compCount);
    for (var aa = 0; aa < compCount; ++aa)
        constrExpParts[aa] = getScalarTypeValStr(firstCompValue + aa, scalarType);

    return getGLSLArgumentType(typeCode, targetType) + "(" + constrExpParts.join(", ") + ");";
  }
}


// Returns the expression to select a component of the structured type
function getComponentSelectorExpStr(targetType, compIx) {
  if (isGLSLTypeMatrix(targetType)) {
    var colRowIx = getColRowIndexFromLinearIndex(compIx, getGLSLColumnSize(targetType));
    return "v[" + colRowIx.colIx + "][" + colRowIx.rowIx + "]";
  }
  else
    return "v[" + compIx + "]";
}


// Returns expression which validates the components set by the constructor expression
function getComponentValidationExpression(refCompVals, targetType) {
  // Early out for invalid arguments
  if (refCompVals.length === 0)
    return "false";

  var scalarType = getGLSLScalarType(targetType);
  var checkComponentValueParts = new Array(refCompVals.length);
  for (var cc = 0; cc < refCompVals.length; ++cc) {
    var val_str = getScalarTypeValStr(refCompVals[cc], scalarType);
    var comp_sel_exp = getComponentSelectorExpStr(targetType, cc);
    if (scalarType === "float") {
      // Comparison of floating point values with error bound
      checkComponentValueParts[cc] = "abs(" + comp_sel_exp + " - " + val_str + ") <= errorBound";
    }
    else {
      // Simple comparison to expected value
      checkComponentValueParts[cc] = comp_sel_exp + " == " + val_str;
    }
  }

  return checkComponentValueParts.join(" && ");
}


// Returns substitution parts to turn the shader template into testable shader code
function getTestShaderParts(targetType, argExp, firstCompValue) {
  // glsl code of declarations of arguments
  var argsListParts = new Array(argExp.length);

  // glsl code of constructor expression
  var argsConstrParts = new Array(argExp.length);

  // glsl type expression
  var typeExpParts = new Array(argExp.length);
  for (var aa = 0; aa < argExp.length; ++aa) {
    var typeCode     = argExp[aa];
    var argCompCount = getTypeCodeComponentCount(typeCode);
    var argName      = "a" + aa;
    var argType      = getGLSLArgumentType(typeCode, targetType);
    var argConstrExp = argType + " " + argName + " = " + getComponentSequenceConstructorExpression(typeCode, firstCompValue, targetType);

    // Add construction of one argument
    // Indent if not first argument
    argsListParts[aa] = ((aa > 0) ? "  " : "") + argConstrExp;

    // Add argument name to target type argument list
    argsConstrParts[aa] = argName;

    // Add type name to type expression
    typeExpParts[aa] = argType;

    // Increment argument component value so all argument component arguments have a unique value
    firstCompValue += argCompCount;
  }

  return {
    argsList:   argsListParts.join("\n") + "\n",
    argsConstr: argsConstrParts.join(", "),
    typeExp:    targetType + "(" + typeExpParts.join(", ") + ")"
  };
}


// Utility functions to manipulate the array of reference values

// Returns array filled with identical values
function getArrayWithIdenticalValues(size, val) {
  var matArray = new Array(size);
  for (var aa = 0; aa < size; ++aa)
    matArray[aa] = val;

  return matArray;
}


// Returns array filled with increasing values from a specified start value
function getArrayWithIncreasingValues(size, start) {
  var matArray = new Array(size);
  for (var aa = 0; aa < size; ++aa)
    matArray[aa] = start + aa;

  return matArray;
}


// Utility functions to manipulate the array of reference values if the target type is a matrix

// Returns an array which is the column order layout of a square matrix where the diagonal is set to a specified value
function matCompArraySetDiagonal(matArray, diagVal) {
  // The entries for the diagonal start at array index 0 and increase
  // by column size + 1
  var colSize = Math.round(Math.sqrt(matArray.length));
  var dIx = 0;
  do {
    matArray[dIx] = diagVal;
    dIx += (colSize + 1);
  }
  while (dIx < colSize * colSize);

  return matArray;
}


// Returns an array which contains the values of an identity matrix read out in column order
function matCompArrayCreateDiagonalMatrix(colSize, diagVal) {
  var size = colSize * colSize;
  var matArray = new Array(size);
  for (var aa = 0; aa < size; ++aa)
    matArray[aa] = 0;

  return matCompArraySetDiagonal(matArray, diagVal);
}


// Returns the column and row index from the linear index if the components of the matrix are stored in column order in an array
// in a one dimensional array in column order
function getColRowIndexFromLinearIndex(linIx, colSize) {
  return {
    colIx: Math.floor(linIx / colSize),
    rowIx: linIx % colSize
  };
}


// Returns the linear index for matrix column and row index for a specified matrix size
function getLinearIndexFromColRowIndex(rowColIx, colSize) {
  return rowColIx.colIx * colSize + rowColIx.rowIx;
}


// Returns a matrix set from another matrix
function matCompArraySetMatrixFromMatrix(dstColSize, srcMatArray) {
  // Overwrite components from destination with the source component values at the same col, row coordinates
  var dstMatArray = matCompArrayCreateDiagonalMatrix(dstColSize, 1);

  var srcColSize = Math.round(Math.sqrt(srcMatArray.length));

  for (var c_ix = 0; c_ix < srcMatArray.length; ++c_ix) {
    var srcMatIx = getColRowIndexFromLinearIndex(c_ix, srcColSize);
    if (srcMatIx.colIx < dstColSize && srcMatIx.rowIx < dstColSize) {
      // Source matrix coordinates are valid destination matrix coordinates
      dstMatArray[getLinearIndexFromColRowIndex(srcMatIx, dstColSize)] = srcMatArray[c_ix];
    }
  }

  return dstMatArray;
}


// Returns the glsl code to verify if the components are set correctly
// and the message to display for the test
function getConstructorExpressionInfo(targetType, argExp, firstCompValue) {
  var argCompCountsSum = 0;
  var argCompCounts = new Array(argExp.length);
  for (var aa = 0; aa < argExp.length; ++aa) {
    argCompCounts[aa] = getTypeCodeComponentCount(argExp[aa]);
    argCompCountsSum += argCompCounts[aa];
  }

  var targetCompCount = getGLSLTypeComponentCount(targetType);

  var refCompVals;
  var testMsg;
  var valid;

  if (argCompCountsSum === 0) {
    // A constructor needs at least one argument
    refCompVals = [];
    testMsg     = "invalid (no arguments)";
    valid       = false;
  }
  else {
    if (isGLSLTypeVector(targetType)) {
      if (argCompCountsSum === 1) {
        // One scalar argument
        // Vector constructor with one scalar argument set all components to the same value
        refCompVals = getArrayWithIdenticalValues(targetCompCount, firstCompValue);
        testMsg     = "valid (all components set to the same value)";
        valid       = true;
      }
      else {
        // Not one scalar argument
        if (argCompCountsSum < targetCompCount) {
          // Not all components set
          refCompVals = [];
          testMsg     = "invalid (not enough arguments)";
          valid       = false;
        }
        else {
          // argCompCountsSum >= targetCompCount
          // All components set
          var lastArgFirstCompIx = argCompCountsSum - argCompCounts[argCompCounts.length - 1];

          if (lastArgFirstCompIx < targetCompCount) {
            // First component of last argument is used
            refCompVals = getArrayWithIncreasingValues(targetCompCount, firstCompValue);
            testMsg     = "valid";
            valid       = true;
          }
          else {
            // First component of last argument is not used
            refCompVals = [];
            testMsg     = "invalid (unused argument)";
            valid       = false;
          }
        }
      }
    }
    else {
      // Matrix target type
      if (argCompCountsSum === 1) {
        // One scalar argument
        // Matrix constructors with one scalar set all components on the diagonal to the same value
        // All other components are set to zero
        refCompVals = matCompArrayCreateDiagonalMatrix(Math.round(Math.sqrt(targetCompCount)), firstCompValue);
        testMsg     = "valid (diagonal components set to the same value, off-diagonal components set to zero)";
        valid       = true;
      }
      else {
        // Not one scalar argument
        if (argExp.length === 1 && argExp[0][0] === "m") {
          // One single matrix argument
          var dstColSize = getGLSLColumnSize(targetType);
          refCompVals = matCompArraySetMatrixFromMatrix(dstColSize, getArrayWithIncreasingValues(getTypeCodeComponentCount(argExp[0]), firstCompValue));
          testMsg     = "valid, components at corresponding col, row indices are set from argument, other components are set from identity matrix";
          valid       = true;
        }
        else {
          // More than one argument or one argument not of type matrix
          // Can be treated in the same manner
          // Arguments can not be of type matrix
          var matFound = false;
          for (var aa = 0; aa < argExp.length; ++aa)
            if (argExp[aa][0] === "m")
              matFound = true;

          if (matFound) {
            refCompVals = [];
            testMsg     = "invalid, argument list greater than one contains matrix type";
            valid       = false;
          }
          else {
            if (argCompCountsSum < targetCompCount) {
              refCompVals = [];
              testMsg     = "invalid (not enough arguments)";
              valid       = false;
            }
            else {
              // argCompCountsSum >= targetCompCount
              // All components set
              var lastArgFirstCompIx = argCompCountsSum - argCompCounts[argCompCounts.length - 1];

              if (lastArgFirstCompIx < targetCompCount) {
                // First component of last argument is used
                refCompVals = getArrayWithIncreasingValues(targetCompCount, firstCompValue);
                testMsg     = "valid";
                valid       = true;
              }
              else {
                // First component of last argument is not used
                refCompVals = [];
                testMsg     = "invalid (unused argument)";
                valid       = false;
              }
            }
          }
        }
      }
    }
  }

  // Check if no case is missed
  if (testMsg == null || valid == null) {
    wtu.error("GLSLConstructorTestsGenerator.getConstructorExpressionInfo(), info not set");
    debugger;
  }

  return {
    refCompVals: refCompVals,
    testMsg:     testMsg,
    valid:       valid
  };
}


// Returns a vertex shader testcase and a fragment shader testcase
function getVertexAndFragmentShaderTestCase(targetType, argExp) {
  var firstCompValue = 0;
  if (isGLSLTypeMatrix(targetType)) {
    // Use value different from 0 and 1
    // 0 and 1 are values used by matrix constructed from a matrix or a single scalar
    firstCompValue = 2;
  }

  var argCode = getTestShaderParts          (targetType, argExp, firstCompValue);
  var expInfo = getConstructorExpressionInfo(targetType, argExp, firstCompValue);

  var substitutions = {
    type:          targetType,
    errorBound:    (getGLSLScalarType(targetType) === "float") ? "const float errorBound = 1.0E-5;" : "",
    argsList:      argCode.argsList,
    argsConstr:    argCode.argsConstr,
    checkCompVals: getComponentValidationExpression(expInfo.refCompVals, targetType)
  };

  return [ {
      // Test constructor argument list in vertex shader
      vShaderSource:  wtu.replaceParams(constructorVertexTemplate, substitutions),
      vShaderSuccess: expInfo.valid,
      fShaderSource:  passThroughColorFragmentShader,
      fShaderSuccess: true,
      linkSuccess:    expInfo.valid,
      passMsg:        "Vertex shader : " + argCode.typeExp + ", " + expInfo.testMsg,
      render:         expInfo.valid
    }, {
      // Test constructor argument list in fragment shader
      fShaderSource:  wtu.replaceParams(constructorFragmentTemplate, substitutions),
      fShaderSuccess: expInfo.valid,
      linkSuccess:    expInfo.valid,
      passMsg:        "Fragment shader : " + argCode.typeExp + ", " + expInfo.testMsg,
      render:         expInfo.valid
    }
  ];
}


// Incrementing the argument expressions
// Utility object which defines the order of incrementing the argument types
var typeCodeIncrementer = {
  s:     { typeCode: "v2", order: 0 },
  v2:    { typeCode: "v3", order: 1 },
  v3:    { typeCode: "v4", order: 2 },
  v4:    { typeCode: "m2", order: 3 },
  m2:    { typeCode: "m3", order: 4 },
  m3:    { typeCode: "m4", order: 5 },
  m4:    { typeCode: "s",  order: 6 },
  first: "s"
}


// Returns the next argument sequence
function getNextArgumentSequence(inSeq) {
  var nextSeq;
  if (inSeq.length === 0) {
    // Current argument sequence is empty, add first argument
    nextSeq = [typeCodeIncrementer.first];
  }
  else {
    nextSeq = new Array(inSeq.length);
    var overflow = true;
    for (var aa = 0; aa < inSeq.length; ++aa) {
      var currArg = inSeq[aa];
      if (overflow) {
        // Increment the current argument type
        var nextArg = typeCodeIncrementer[currArg].typeCode;
        nextSeq[aa] = nextArg;
        overflow = (nextArg === typeCodeIncrementer.first);
      }
      else {
        // Copy remainder of sequence
        nextSeq[aa] = currArg;
      }
    }

    if (overflow) {
      nextSeq.push(typeCodeIncrementer.first);
    }
  }

  return nextSeq;
}


// Returns true if two argument expressions are equal
function areArgExpEqual(expA, expB) {
  if (expA.length !== expB.length)
    return false;

  for (var aa = 0; aa < expA.length; ++aa)
    if (expA[aa] !== expB[aa])
      return false;

  return true;
}


// Returns true if first argument expression is smaller
// (comes before the second one in iterating order)
// compared to the second argument expression
function isArgExpSmallerOrEqual(argExpA, argExpB) {
  var aLen = argExpA.length;
  var bLen = argExpB.length;
  if (aLen !== bLen)
    return (aLen < bLen);

  // Argument type expression lengths are equal
  for (var aa = aLen - 1; aa >= 0; --aa) {
    var argA = argExpA[aa];
    var argB = argExpB[aa];

    if (argA !== argB) {
      var aOrder = typeCodeIncrementer[argA].order;
      var bOrder = typeCodeIncrementer[argB].order;
      if (aOrder !== bOrder)
        return (aOrder < bOrder);
    }
  }

  // Argument type expressions are equal
  return true;
}


// Returns the next argument expression from sequence set
// Returns null if end is reached
function getNextArgumentExpression(testExp, testSet) {
  var testInterval = testSet[testExp.ix];

  if (areArgExpEqual(testExp.argExp, testInterval[1])) {
    // End of current interval reached
    if (testExp.ix === testSet.length - 1) {
      // End of set reached
      return null;
    }
    else {
      // Return first argument expression of next interval
      var nextIx = testExp.ix + 1;
      return { ix: nextIx, argExp: testSet[nextIx][0] };
    }
  }
  else {
    // Return next expression in current interval
    return { ix: testExp.ix, argExp: getNextArgumentSequence(testExp.argExp) };
  }
}


// Returns an array of the parts in the string separated by commas and with the white space trimmed
function convertCsvToArray(str) {
  // Checks type codes in input
  function checkInput(el, ix, arr) {
    var typeCode = el.trim();
    if (!(typeCode in typeCodeIncrementer) && typeCode !== "first") {
      wtu.error("GLSLConstructorTestsGenerator.convertCsvToArray(), unknown type code" + typeCode);
      debugger;
    }

    arr[ix] = typeCode;
  }

  var spArr = str.split(",");

  // Convert empty string to empty array
  if (spArr.length === 1 && spArr[0].trim() === "")
    spArr = [];

  spArr.forEach(checkInput);

  return spArr;
}


// Processes the set of specified test sequences
function processInputs(testSequences) {
  var testSet = new Array(testSequences.length);
  for (var tt = 0; tt < testSequences.length; ++tt) {
    var interval = testSequences[tt];
    var bounds = interval.split("-");
    var begin = convertCsvToArray(bounds[0]);
    var end   = convertCsvToArray(bounds[bounds.length - 1]);

    // Check if interval is valid
    if (!isArgExpSmallerOrEqual(begin, end)) {
      wtu.error("GLSLConstructorTestsGenerator.processInputs(), interval not valid");
      debugger;
    }

    testSet[tt] = [ begin, end ];
  }

  return testSet;
}


/**
 * Returns list of test cases for vector types
 * All combinations of arguments up to one unused argument of one component are tested
 * @param {targetType} Name of target type to test the constructor expressions on
 * @param {testSet}    Set of intervals of argument sequences to test
 */
function getConstructorTests(targetType, testSequences) {
  // List of tests to return
  var testInfos = [];

  // List of argument types
  var testSet = processInputs(testSequences);
  var testExp = { ix: 0, argExp: testSet[0][0] };

  do {
    // Add one vertex shader test case and one fragment shader test case
    testInfos = testInfos.concat(getVertexAndFragmentShaderTestCase(targetType, testExp.argExp));

    // Generate next argument expression
    testExp = getNextArgumentExpression(testExp, testSet);
  }
  while (testExp != null);

  return testInfos;
}


// Returns default test argument expression set
// For details on input format : see bottom of file
function getDefaultTestSet(targetType) {
  switch(targetType) {
    case "vec2":
    case "ivec2":
    case "bvec2":
      return [
        // No arguments and all single argument expressions
        " - m4",

        // All two argument expressions with a scalar as second argument
        "s, s - m4, s",

        // All two arguments expressions with a scalar as first argument
        "s, v2", "s, v3", "s, v4", "s, m2", "s, m3", "s, m4",

        // Three argument expression
        "s, s, s"
      ];

    case "vec3":
    case "ivec3":
    case "bvec3":
      return [
        // No arguments and all single argument expressions
        " - m4",

        // All two argument expressions with a scalar as second argument
        "s, s - m4, s",

        // All two argument expressions with a scalar as first argument
        "s, v2", "s, v3", "s, v4", "s, m2", "s, m3", "s, m4",

        // All three argument expressions with two scalars as second and third argument
        "s, s, s - m4, s, s",

        // All three argument expressions with two scalars as first and second argument
        "s, s, v2", "s, s, v3", "s, s, v4", "s, s, m2", "s, s, m3", "s, s, m4",

        // Four argument expression
        "s, s, s, s"
      ];

    case "vec4":
    case "ivec4":
    case "bvec4":
    case "mat2":
      return [
        // No arguments and all single argument expressions
        " - m4",

        // All two argument expressions with a scalar as second argument
        "s, s - m4, s",

        // All two argument expressions with a scalar as first argument
        "s, v2", "s, v3", "s, v4", "s, m2", "s, m3", "s, m4",

        // All three argument expressions with two scalars as second and third argument
        "s, s, s - m4, s, s",

        // All three argument expressions with two scalars as first and second argument
        "s, s, v2", "s, s, v3", "s, s, v4", "s, s, m2", "s, s, m3", "s, s, m4",

        // All four argument expressions with three scalars as second, third and fourth argument
        "s, s, s, s - m4, s, s, s",

        // All four argument expressions with three scalars as first, second and third argument
        "s, s, s, v2", "s, s, s, v3", "s, s, s, v4", "s, s, s, m2", "s, s, s, m3", "s, s, s, m4",

        // Five argument expression
        "s, s, s, s, s"
      ];

    case "mat3":
    case "mat4":
      return [
        // No arguments and all single argument expressions
        " - m4",

        // All two argument expressions with a scalar as second argument
        "s, s - m4, s",

        // All two argument expressions with a scalar as first argument
        "s, v2", "s, v3", "s, v4", "s, m2", "s, m3", "s, m4",

        // Several argument sequences
        "v4, s, v4", "v4, s, v3, v2", "v4, v4, v3, v2", "v4, v4, v4, v4", "v2, v2, v2, v2, v2", "v2, v2, v2, v2, v2, v2, v2, v2",
        "v3, v3, v3", "v3, v3, v3, s", "v3, v3, v3, v3, v3, s", "v3, v3, v3, v3, v3, s, s",
      ];
  }
}


// Return publics
return {
  getConstructorTests: getConstructorTests,
  getDefaultTestSet:   getDefaultTestSet
};

}());


// Input is an array of intervals of argument types
// The generated test argument sequences are from (including) the lower interval boundary
// until (including) the upper boundary
// Coding and order of the different argument types :
// s  : scalar
// v2 : vec2
// v3 : vec3
// v4 : vec4
// m2 : mat2
// m3 : mat3
// m4 : mat4

// One interval is put in one string
// Low and high bound are separated by a dash.
// If there is no dash it is regarded as an interval of one expression
// The individual argument codes are separated by commas
// The individual arguments are incremented from left to right
// The left most argument is the one which is incremented first
// Once the left most arguments wraps the second argument is increased
// Examples :
// "s - m4"        : All single arguments from scalar up to (including) mat4
// "m2, s - m4, s" : All two argument expressions with a matrix argument as first argument and a scalar as second argument
// " - m4, m4"     : The empty argument, all one arguments and all two argument expressions
// "m2, s, v3, m4" : One 4 argument expression : mat2, scalar, vec3, mat4

var msg = decodeURIComponent(location.hash.substr(1));

var log = [];
function check_true(actual, msg) {
  if (actual !== true) {
    log.push(msg);
    return false;
  }
  return true;
}

function check_Blob(msg, input, port, expect_File, orig_input) {
  expect_File = !!expect_File;
  orig_input = orig_input || input;
  try {
    var expected;
    switch (msg) {
      case 'Blob basic':
      case 'File basic':
        expected = [0x66, 0x6F, 0x6F];
        expected.type = 'text/x-bar';
        if (expect_File) {
          expected.name = 'bar';
          expected.lastModified = 42;
        }
        break;
      case 'Blob unpaired high surrogate (invalid utf-8)':
        expected = [0xED, 0xA0, 0x80];
        expected.type = '';
        break;
      case 'Blob unpaired low surrogate (invalid utf-8)':
        expected = [0xED, 0xB0, 0x80];
        expected.type = '';
        break;
      case 'Blob paired surrogates (invalid utf-8)':
        expected = [0xED, 0xA0, 0x80, 0xED, 0xB0, 0x80];
        expected.type = '';
        break;
      case 'Blob empty':
        expected = [];
        expected.type = '';
        break;
      case 'Blob NUL':
        var expected = [0x00];
        expected.type = '';
        break;
      default:
        check_true(false, 'check_Blob: unknown test');
        return;
        break;
    }
    if (check_true(input instanceof Blob, 'input instanceof Blob') &&
        check_true((input instanceof File) == expect_File, '(input instanceof File) == expect_File') &&
        check_true(input.size === expected.length, 'input.size === expected.length') &&
        check_true(input.type === expected.type, 'input.type === expected.type')) {
      if (!expect_File || (check_true(input.name === expected.name, 'input.name === expected.name') &&
                           check_true(input.lastModified === expected.lastModified))) {
        var reader = new FileReader();
        var read_done = function() {
          try {
            var result = reader.result;
            check_true(result.byteLength === expected.length, 'result.byteLength === expected.length')
            var view = new DataView(result);
            for (var i = 0; i < result.byteLength; ++i) {
              check_true(view.getUint8(i) === expected[i], 'view.getUint8('+i+') === expected['+i+']')
            }
            if (log.length === 0) {
              port.postMessage(orig_input);
            } else {
              port.postMessage('FAIL '+log);
            }
            close();
          } catch(ex) {
            postMessage('FAIL '+ex);
            close();
          }
        }
        var read_error = function() { port.postMessage('FAIL (got FileReader error)'); close(); };
        reader.readAsArrayBuffer(input);
        reader.onload = read_done;
        reader.onabort = reader.onerror = read_error;
      }
    } else {
      port.postMessage('FAIL '+log);
      close();
    }
  } catch(ex) {
    postMessage('FAIL '+ex);
    close();
  }
}

function check_ImageData(input, expected) {
  if (check_true(input instanceof ImageData, 'input instanceof ImageData') &&
      check_true(input.width === expected.width, 'input.width === '+expected.width) &&
      check_true(input.height === expected.height, 'input.height === '+expected.height) &&
      check_true(input.data instanceof Uint8ClampedArray, 'input.data instanceof Uint8ClampedArray') &&
      check_true(input.data.length === expected.data.length, 'input.data.length === '+expected.data.length) &&
      check_true(!('CanvasPixelArray' in self), "!('CanvasPixelArray' in self)")) {
    for (var i = 0; i < input.length; ++i) {
      if (!(check_true(input.data[i] === expected.data[i], 'input.data['+i+'] === '+expected.data[i]))) {
        return false;
      }
    }
    return true;
  }
  return false;
}

function check_ImageBitmap(input, expected) {
  return check_true(input instanceof ImageBitmap, 'input instanceof ImageBitmap');
  // XXX paint it on a proxy canvas and check the data
}

function check_RegExp(msg, input) {
  // XXX ES6 spec doesn't define exact serialization for `source` (it allows several ways to escape)
  switch (msg) {
    case 'RegExp flags and lastIndex':
      return check_true(input instanceof RegExp, "input instanceof RegExp") &&
             check_true(input.source === 'foo', "input.source === 'foo'") &&
             check_true(input.global === true, "input.global === true") &&
             check_true(input.ignoreCase === true, "input.ignoreCase === true") &&
             check_true(input.multiline === true, "input.multiline === true") &&
             check_true(input.lastIndex === 0, "input.lastIndex === 0");
      break;
    case 'RegExp sticky flag':
      return check_true(input instanceof RegExp, "input instanceof RegExp") &&
             check_true(input.source === 'foo', "input.source === 'foo'") &&
             check_true(input.global === false, "input.global === false") &&
             check_true(input.ignoreCase === false, "input.ignoreCase === false") &&
             check_true(input.multiline === false, "input.multiline === false") &&
             check_true(input.sticky === true, "input.sticky === true") &&
             check_true(input.unicode === false, "input.unicode === false") &&
             check_true(input.lastIndex === 0, "input.lastIndex === 0");
      break;
    case 'RegExp unicode flag':
      return check_true(input instanceof RegExp, "input instanceof RegExp") &&
             check_true(input.source === 'foo', "input.source === 'foo'") &&
             check_true(input.global === false, "input.global === false") &&
             check_true(input.ignoreCase === false, "input.ignoreCase === false") &&
             check_true(input.multiline === false, "input.multiline === false") &&
             check_true(input.sticky === false, "input.sticky === false") &&
             check_true(input.unicode === true, "input.unicode === true") &&
             check_true(input.lastIndex === 0, "input.lastIndex === 0");
      break;
    case 'RegExp empty':
      return check_true(input instanceof RegExp, "input instanceof RegExp") &&
             check_true(input.source === '(?:)', "input.source === '(?:)'") &&
             check_true(input.global === false, "input.global === false") &&
             check_true(input.ignoreCase === false, "input.ignoreCase === false") &&
             check_true(input.multiline === false, "input.multiline === false") &&
             check_true(input.lastIndex === 0, "input.lastIndex === 0");
      break;
    case 'RegExp slash':
      return check_true(input instanceof RegExp, "input instanceof RegExp") &&
             check_true(input.source === '\\/', "input.source === '\\\\/'") &&
             check_true(input.global === false, "input.global === false") &&
             check_true(input.ignoreCase === false, "input.ignoreCase === false") &&
             check_true(input.multiline === false, "input.multiline === false") &&
             check_true(input.lastIndex === 0, "input.lastIndex === 0");
      break;
    case 'RegExp new line':
      return check_true(input instanceof RegExp, "input instanceof RegExp") &&
             check_true(input.source === '\\n', "input.source === '\\\\n'") &&
             check_true(input.global === false, "input.global === false") &&
             check_true(input.ignoreCase === false, "input.ignoreCase === false") &&
             check_true(input.multiline === false, "input.multiline === false") &&
             check_true(input.lastIndex === 0, "input.lastIndex === 0");
      break;
    default:
      check_true(false, 'check_RegExp: unknown test');
      return false;
      break;
  }
}

function check_FileList(msg, input) {
  try {
    return check_true(input instanceof FileList, 'input instanceof FileList') &&
           check_true(input.length === 0, 'input.length === 0');
  } catch(ex) {
    return check_true(false, ex);
  }
}

function check(input, port) {
  try {
    switch (msg) {
      case 'primitive undefined':
        if (check_true(input === undefined, 'input === undefined')) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive null':
        if (check_true(input === null, 'input === null')) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive true':
        if (check_true(input === true, 'input === true')) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive false':
        if (check_true(input === false, 'input === false')) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive string, empty string':
        if (check_true(input === '', "input === ''")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive string, lone high surrogate':
        if (check_true(input === '\uD800', "input === '\uD800'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive string, lone low surrogate':
        if (check_true(input === '\uDC00', "input === '\uDC00'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive string, NUL':
        if (check_true(input === '\u0000', "input === '\u0000'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive string, astral character':
        if (check_true(input === '\uDBFF\uDFFD', "input === '\uDBFF\uDFFD'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, 0.2':
        if (check_true(input === 0.2, "input === 0.2")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, 0':
        if (check_true(input === 0, "input === 0") &&
            check_true(1/input === Infinity, "1/input === Infinity")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, -0':
        if (check_true(input === 0, "input === 0") &&
            check_true(1/input === -Infinity, "1/input === -Infinity")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, NaN':
        if (check_true(input !== input, "input !== input")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, Infinity':
        if (check_true(input === Infinity, "input === Infinity")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, -Infinity':
        if (check_true(input === -Infinity, "input === -Infinity")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, 9007199254740992':
        if (check_true(input === 9007199254740992, "input === 9007199254740992")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, -9007199254740992':
        if (check_true(input === -9007199254740992, "input === -9007199254740992")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, 9007199254740994':
        if (check_true(input === 9007199254740994, "input === 9007199254740994")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'primitive number, -9007199254740994':
        if (check_true(input === -9007199254740994, "input === -9007199254740994")) {
          port.postMessage(input);
          close();
          break;
        }
      case 'Array primitives':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 19, 'input.length === 19') &&
            check_true(input[0] === undefined, 'input[0] === undefined') &&
            check_true(input[1] === null, 'input[1] === null') &&
            check_true(input[2] === true, 'input[2] === true') &&
            check_true(input[3] === false, 'input[3] === false') &&
            check_true(input[4] === '', "input[4] === ''") &&
            check_true(input[5] === '\uD800', "input[5] === '\\uD800'") &&
            check_true(input[6] === '\uDC00', "input[6] === '\\uDC00'") &&
            check_true(input[7] === '\u0000', "input[7] === '\\u0000'") &&
            check_true(input[8] === '\uDBFF\uDFFD', "input[8] === '\\uDBFF\\uDFFD'") &&
            check_true(input[9] === 0.2, "input[9] === 0.2") &&
            check_true(1/input[10] === Infinity, "1/input[10] === Infinity") &&
            check_true(1/input[11] === -Infinity, "1/input[11] === -Infinity") &&
            check_true(input[12] !== input[11], "input[12] !== input[11]") &&
            check_true(input[13] === Infinity, "input[13] === Infinity") &&
            check_true(input[14] === -Infinity, "input[14] === -Infinity") &&
            check_true(input[15] === 9007199254740992, "input[15] === 9007199254740992") &&
            check_true(input[16] === -9007199254740992, "input[16] === -9007199254740992") &&
            check_true(input[17] === 9007199254740994, "input[17] === 9007199254740994") &&
            check_true(input[18] === -9007199254740994, "input[18] === -9007199254740994")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object primitives':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_true(input['undefined'] === undefined, "input['undefined'] === undefined") &&
              check_true(input['null'] === null, "input['null'] === null") &&
              check_true(input['true'] === true, "input['true'] === true") &&
              check_true(input['false'] === false, "input['false'] === false") &&
              check_true(input['empty'] === '', "input['empty'] === ''") &&
              check_true(input['high surrogate'] === '\uD800', "input['high surrogate'] === '\uD800'") &&
              check_true(input['low surrogate'] === '\uDC00', "input['low surrogate'] === '\uDC00'") &&
              check_true(input['nul'] === '\u0000', "input['nul'] === '\u0000'") &&
              check_true(input['astral'] === '\uDBFF\uDFFD', "input['astral'] === '\uDBFF\uDFFD'") &&
              check_true(input['0.2'] === 0.2, "input['0.2'] === 0.2") &&
              check_true(1/input['0'] === Infinity, "1/input['0'] === Infinity") &&
              check_true(1/input['-0'] === -Infinity, "1/input['-0'] === -Infinity") &&
              check_true(input['NaN'] !== input['NaN'], "input['NaN'] !== input['NaN']") &&
              check_true(input['Infinity'] === Infinity, "input['Infinity'] === Infinity") &&
              check_true(input['-Infinity'] === -Infinity, "input['-Infinity'] === -Infinity") &&
              check_true(input['9007199254740992'] === 9007199254740992, "input['9007199254740992'] === 9007199254740992") &&
              check_true(input['-9007199254740992'] === -9007199254740992, "input['-9007199254740992'] === -9007199254740992") &&
              check_true(input['9007199254740994'] === 9007199254740994, "input['9007199254740994'] === 9007199254740994") &&
              check_true(input['-9007199254740994'] === -9007199254740994, "input['9007199254740994'] === -9007199254740994")) {
            var i = 0;
            for (var x in input) {
              i++;
            }
            if (check_true(i === 19, 'i === 19')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'Boolean true':
        if (check_true(input instanceof Boolean, "input instanceof Boolean") &&
            check_true(String(input) === 'true', "String(input) === 'true'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Boolean false':
        if (check_true(input instanceof Boolean, "input instanceof Boolean") &&
            check_true(String(input) === 'false', "String(input) === 'false'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array Boolean objects':
        (function() {
          if (check_true(input instanceof Array, 'input instanceof Array') &&
              check_true(input.length === 2, 'input.length === 2') &&
              check_true(String(input[0]) === 'true', "String(input[0]) === 'true'") &&
              check_true(String(input[1]) === 'false', "String(input[1]) === 'false'")) {
            for (var i = 0; i < input.length; ++i) {
              if (!check_true(input[i] instanceof Boolean, 'input['+i+'] instanceof Boolean'))
                return;
            }
            port.postMessage(input);
            close();
          }
        })();
        break;
      case 'Object Boolean objects':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_true(String(input['true']) === 'true', "String(input['true']) === 'true'") &&
              check_true(String(input['false']) === 'false', "String(input['false']) === 'false'")) {
            var i = 0;
            for (var x in input) {
              i++;
              if (!check_true(input[x] instanceof Boolean, 'input['+x+'] instanceof Boolean'))
                return;
            }
            if (check_true(i === 2, 'i === 2')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'String empty string':
        if (check_true(input instanceof String, "input instanceof String") &&
            check_true(String(input) === '', "String(input) === ''")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'String lone high surrogate':
        if (check_true(input instanceof String, "input instanceof String") &&
            check_true(String(input) === '\uD800', "String(input) === '\\uD800'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'String lone low surrogate':
        if (check_true(input instanceof String, "input instanceof String") &&
            check_true(String(input) === '\uDC00', "String(input) === '\\uDC00'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'String NUL':
        if (check_true(input instanceof String, "input instanceof String") &&
            check_true(String(input) === '\u0000', "String(input) === '\\u0000'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'String astral character':
        if (check_true(input instanceof String, "input instanceof String") &&
            check_true(String(input) === '\uDBFF\uDFFD', "String(input) === '\\uDBFF\\uDFFD'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array String objects':
        (function() {
          if (check_true(input instanceof Array, 'input instanceof Array') &&
              check_true(input.length === 5, 'input.length === 5') &&
              check_true(String(input[0]) === '', "String(input[0]) === ''") &&
              check_true(String(input[1]) === '\uD800', "String(input[1]) === '\\uD800'") &&
              check_true(String(input[2]) === '\uDC00', "String(input[1]) === '\\uDC00'") &&
              check_true(String(input[3]) === '\u0000', "String(input[2]) === '\\u0000'") &&
              check_true(String(input[4]) === '\uDBFF\uDFFD', "String(input[3]) === '\\uDBFF\\uDFFD'")) {
            for (var i = 0; i < input.length; ++i) {
              if (!check_true(input[i] instanceof String, 'input['+i+'] instanceof String'))
                return;
            }
            port.postMessage(input);
            close();
          }
        })();
        break;
      case 'Object String objects':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_true(String(input['empty']) === '', "String(input['empty']) === ''") &&
              check_true(String(input['high surrogate']) === '\uD800', "String(input['high surrogate']) === '\\uD800'") &&
              check_true(String(input['low surrogate']) === '\uDC00', "String(input['low surrogate']) === '\\uDC00'") &&
              check_true(String(input['nul']) === '\u0000', "String(input['nul']) === '\\u0000'") &&
              check_true(String(input['astral']) === '\uDBFF\uDFFD', "String(input['astral']) === '\\uDBFF\\uDFFD'")) {
            var i = 0;
            for (var x in input) {
              i++;
              if (!check_true(input[x] instanceof String, 'input['+x+'] instanceof Boolean'))
                return;
            }
            if (check_true(i === 5, 'i === 5')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'Number 0.2':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(Number(input) === 0.2, "Number(input) === 0.2")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number 0':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(1/Number(input) === Infinity, "1/Number(input) === Infinity")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number -0':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(1/Number(input) === -Infinity, "1/Number(input) === -Infinity")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number NaN':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(Number(input) !== Number(input), "Number(input) !== Number(input)")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number Infinity':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(Number(input) === Infinity, "Number(input) === Infinity")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number -Infinity':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(Number(input) === -Infinity, "Number(input) === -Infinity")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number 9007199254740992':
        if (check_true(input instanceof Number) &&
            check_true(Number(input) === 9007199254740992, "Number(input) === 9007199254740992")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number -9007199254740992':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(Number(input) === -9007199254740992, "Number(input) === -9007199254740992")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number 9007199254740994':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(Number(input) === 9007199254740994, "Number(input) === 9007199254740994")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Number -9007199254740994':
        if (check_true(input instanceof Number, "input instanceof Number") &&
            check_true(Number(input) === -9007199254740994, "Number(input) === -9007199254740994")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array Number objects':
        (function() {
          if (check_true(input instanceof Array, 'input instanceof Array') &&
              check_true(input.length === 10, 'input.length === 10') &&
              check_true(Number(input[0]) === 0.2, "Number(input[0]) === 0.2") &&
              check_true(1/Number(input[1]) === Infinity, "1/Number(input[1]) === Infinity") &&
              check_true(1/Number(input[2]) === -Infinity, "1/Number(input[2]) === -Infinity") &&
              check_true(Number(input[3]) !== Number(input[3]), "Number(input[3]) !== Number(input[3])") &&
              check_true(Number(input[4]) === Infinity, "Number(input[4]) === Infinity") &&
              check_true(Number(input[5]) === -Infinity, "Number(input[5]) === -Infinity") &&
              check_true(Number(input[6]) === 9007199254740992, "Number(input[6]) === 9007199254740992") &&
              check_true(Number(input[7]) === -9007199254740992, "Number(input[7]) === -9007199254740992") &&
              check_true(Number(input[8]) === 9007199254740994, "Number(input[8]) === 9007199254740994") &&
              check_true(Number(input[9]) === -9007199254740994, "Number(input[9]) === -9007199254740994")) {
            for (var i = 0; i < input.length; ++i) {
              if (!check_true(input[i] instanceof Number, 'input['+i+'] instanceof Number'))
                return;
            }
            port.postMessage(input);
            close();
          }
        })();
        break;
      case 'Object Number objects':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_true(Number(input['0.2']) === 0.2, "Number(input['0.2']) === 0.2") &&
              check_true(1/Number(input['0']) === Infinity, "1/Number(input['0']) === Infinity") &&
              check_true(1/Number(input['-0']) === -Infinity, "1/Number(input['-0']) === -Infinity") &&
              check_true(Number(input['NaN']) !== Number(input['NaN']), "Number(input['NaN']) !== Number(input['NaN'])") &&
              check_true(Number(input['Infinity']) === Infinity, "Number(input['Infinity']) === Infinity") &&
              check_true(Number(input['-Infinity']) === -Infinity, "Number(input['-Infinity']) === -Infinity") &&
              check_true(Number(input['9007199254740992']) === 9007199254740992, "Number(input['9007199254740992']) === 9007199254740992") &&
              check_true(Number(input['-9007199254740992']) === -9007199254740992, "Number(input['-9007199254740992']) === -9007199254740992") &&
              check_true(Number(input['9007199254740994']) === 9007199254740994, "Number(input['9007199254740994']) === 9007199254740994") &&
              check_true(Number(input['-9007199254740994']) === -9007199254740994, "Number(input['-9007199254740994']) === -9007199254740994")) {
            var i = 0;
            for (var x in input) {
              i++;
              if (!check_true(input[x] instanceof Number, 'input['+x+'] instanceof Number'))
                return;
            }
            if (check_true(i === 10, 'i === 10')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'Date 0':
        if (check_true(input instanceof Date, "input instanceof Date") &&
            check_true(1/Number(input) === 1/Number(new Date(0)), "1/Number(input) === 1/Number(new Date(0))")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Date -0':
        if (check_true(input instanceof Date, "input instanceof Date") &&
            check_true(1/Number(input) === 1/Number(new Date(-0)), "1/Number(input) === 1/Number(new Date(-0))")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Date -8.64e15':
        if (check_true(input instanceof Date, "input instanceof Date") &&
            check_true(Number(input) === -8.64e15, "Number(input) === -8.64e15")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Date 8.64e15':
        if (check_true(input instanceof Date, "input instanceof Date") &&
            check_true(Number(input) === 8.64e15, "Number(input) === 8.64e15")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array Date objects':
        (function() {
          if (check_true(input instanceof Array, 'input instanceof Array') &&
              check_true(input.length === 4, 'input.length === 4') &&
              check_true(1/Number(input[0]) === 1/new Date(0), '1/Number(input[0]) === 1/new Date(0)') &&
              check_true(1/Number(input[1]) === 1/new Date(-0), '1/Number(input[1]) === 1/new Date(-0)') &&
              check_true(Number(input[2]) === -8.64e15, 'Number(input[2]) === -8.64e15') &&
              check_true(Number(input[3]) === 8.64e15, 'Number(input[3]) === 8.64e15')) {
            for (var i = 0; i < input.length; ++i) {
              if (!check_true(input[i] instanceof Date, 'input['+i+'] instanceof Date'))
                return;
            }
            port.postMessage(input);
            close();
          }
        })();
        break;
      case 'Object Date objects':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_true(1/Number(input['0']) === 1/new Date(0), "1/Number(input['0']) === 1/new Date(0)") &&
              check_true(1/Number(input['-0']) === 1/new Date(-0), "1/Number(input[1]) === 1/new Date(-0)") &&
              check_true(Number(input['-8.64e15']) === -8.64e15, "Number(input['-8.64e15']) === -8.64e15") &&
              check_true(Number(input['8.64e15']) === 8.64e15, "Number(input['8.64e15']) === 8.64e15")) {
            var i = 0;
            for (var x in input) {
              i++;
              if (!check_true(input[x] instanceof Date, 'input['+x+'] instanceof Date'))
                return;
            }
            port.postMessage(input);
            close();
          }
        })();
        break;
      case 'RegExp flags and lastIndex':
      case 'RegExp empty':
      case 'RegExp slash':
      case 'RegExp new line':
        if (check_RegExp(msg, input)) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array RegExp object, RegExp flags and lastIndex':
      case 'Array RegExp object, RegExp empty':
      case 'Array RegExp object, RegExp slash':
      case 'Array RegExp object, RegExp new line':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 1, 'input.length === 1') &&
            check_RegExp(msg.substr('Array RegExp object, '.length), input[0])) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object RegExp object, RegExp flags and lastIndex':
      case 'Object RegExp object, RegExp empty':
      case 'Object RegExp object, RegExp slash':
      case 'Object RegExp object, RegExp new line':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_RegExp(msg.substr('Object RegExp object, '.length), input['x'])) {
            var i = 0;
            for (var x in input) {
              i++;
            }
            if (check_true(i === 1, 'i === 1')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'Blob basic':
      case 'Blob unpaired high surrogate (invalid utf-8)':
      case 'Blob unpaired low surrogate (invalid utf-8)':
      case 'Blob paired surrogates (invalid utf-8)':
      case 'Blob empty':
      case 'Blob NUL':
        check_Blob(msg, input, port);
        // no postMessage or close here, check_Blob takes care of that
        break;
      case 'Array Blob object, Blob basic':
      case 'Array Blob object, Blob unpaired high surrogate (invalid utf-8)':
      case 'Array Blob object, Blob unpaired low surrogate (invalid utf-8)':
      case 'Array Blob object, Blob paired surrogates (invalid utf-8)':
      case 'Array Blob object, Blob empty':
      case 'Array Blob object, Blob NUL':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 1, 'input.length === 1')) {
          check_Blob(msg.substr('Array Blob object, '.length), input[0], port, false, input);
          // no postMessage or close here, check_Blob takes care of that
        }
        break;
      case 'Object Blob object, Blob basic':
      case 'Object Blob object, Blob unpaired high surrogate (invalid utf-8)':
      case 'Object Blob object, Blob unpaired low surrogate (invalid utf-8)':
      case 'Object Blob object, Blob paired surrogates (invalid utf-8)':
      case 'Object Blob object, Blob empty':
      case 'Object Blob object, Blob NUL':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)')) {
            var i = 0;
            for (var x in input) {
              i++;
            }
            if (check_true(i === 1, 'i === 1')) {
              check_Blob(msg.substr('Object Blob object, '.length), input['x'], port, false, input);
              // no postMessage or close here, check_Blob takes care of that
            }
          }
        })();
        break;
      case 'File basic':
        check_Blob(msg, input, port, true);
        // no postMessage or close here, check_Blob takes care of that
        break;
      case 'FileList empty':
        if (check_FileList(msg, input)) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array FileList object, FileList empty':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 1, 'input.length === 1') &&
            check_FileList(msg.substr('Array FileList object, '.length), input[0])) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object FileList object, FileList empty':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_FileList(msg.substr('Array FileList object, '.length), input['x'])) {
            var i = 0;
            for (var x in input) {
              i++;
            }
            if (check_true(i === 1, 'i === 1')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'ImageData 1x1 transparent black':
        if (check_ImageData(input, {width:1, height:1, data:[0,0,0,0]})) {
          port.postMessage(input);
          close();
        }
        break;
      case 'ImageData 1x1 non-transparent non-black':
        if (check_ImageData(input, {width:1, height:1, data:[100, 101, 102, 103]})) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array ImageData object, ImageData 1x1 transparent black':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 1, 'input.length === 1') &&
            check_ImageData(input[0], {width:1, height:1, data:[0,0,0,0]})) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array ImageData object, ImageData 1x1 non-transparent non-black':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 1, 'input.length === 1') &&
            check_ImageData(input[0], {width:1, height:1, data:[100, 101, 102, 103]})) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object ImageData object, ImageData 1x1 transparent black':
        (function(){
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_ImageData(input['x'], {width:1, height:1, data:[0,0,0,0]})) {
            var i = 0;
            for (var x in input) {
              i++;
            }
            if (check_true(i === 1, 'i === 1')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'Object ImageData object, ImageData 1x1 non-transparent non-black':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_ImageData(input['x'], {width:1, height:1, data:[100, 101, 102, 103]})) {
            var i = 0;
            for (var x in input) {
              i++;
            }
            if (check_true(i === 1, 'i === 1')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'ImageBitmap 1x1 transparent black':
        if (check_ImageBitmap(input, {width:1, height:1, data:[0, 0, 0, 0]})) {
          port.postMessage(input);
          close();
        }
        break;
      case 'ImageBitmap 1x1 non-transparent non-black':
        if (check_ImageBitmap(input, {width:1, height:1, data:[100, 101, 102, 103]})) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array ImageBitmap object, ImageBitmap 1x1 transparent black':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 1, 'input.length === 1') &&
            check_ImageBitmap(input[0], {width:1, height:1, data:[0, 0, 0, 0]})) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array ImageBitmap object, ImageBitmap 1x1 non-transparent non-black':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 1, 'input.length === 1') &&
            check_ImageBitmap(input[0], {width:1, height:1, data:[100, 101, 102, 103]})) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object ImageBitmap object, ImageBitmap 1x1 transparent black':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_ImageBitmap(input['x'], {width:1, height:1, data:[0, 0, 0, 0]})) {
            var i = 0;
            for (var x in input) {
              i++;
            }
            if (check_true(i === 1, 'i === 1')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'Object ImageBitmap object, ImageBitmap 1x1 non-transparent non-black':
        (function() {
          if (check_true(input instanceof Object, 'input instanceof Object') &&
              check_true(!(input instanceof Array), '!(input instanceof Array)') &&
              check_ImageBitmap(input['x'], {width:1, height:1, data:[100, 101, 102, 103]})) {
            var i = 0;
            for (var x in input) {
              i++;
            }
            if (check_true(i === 1, 'i === 1')) {
              port.postMessage(input);
              close();
            }
          }
        })();
        break;
      case 'Array sparse':
        (function() {
          if (check_true(input instanceof Array, 'input instanceof Array') &&
              check_true(input.length === 10, 'input.length === 10')) {
            for (var x in input) {
              check_true(false, 'unexpected enumerable property '+x);
              return;
            }
            port.postMessage(input);
            close();
          }
        })();
        break;
      case 'Array with non-index property':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 0, 'input.length === 0') &&
            check_true(input.foo === 'bar', "input.foo === 'bar'")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object with index property and length':
        if (check_true(input instanceof Object, 'input instanceof Object') &&
            check_true(!(input instanceof Array), '!(input instanceof Array)') &&
            check_true(input[0] === 'foo', "input[0] === 'foo'") &&
            check_true(input.length === 1, 'input.length === 1')) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array with circular reference':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 1, 'input.length === 1') &&
            check_true(input[0] === input, "input[0] === input")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object with circular reference':
        if (check_true(input instanceof Object, 'input instanceof Object') &&
            check_true(!(input instanceof Array), '!(input instanceof Array)') &&
            check_true(input['x'] === input, "input['x'] === input")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Array with identical property values':
        if (check_true(input instanceof Array, 'input instanceof Array') &&
            check_true(input.length === 2, 'input.length === 2') &&
            check_true(input[0] === input[1], "input[0] === input[1]")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object with identical property values':
        if (check_true(input instanceof Object, 'input instanceof Object') &&
            check_true(!(input instanceof Array), '!(input instanceof Array)') &&
            check_true(input['x'] === input['y'], "input['x'] === input['y']")) {
          port.postMessage(input);
          close();
        }
        break;
      case 'Object with property on prototype':
      case 'Object with non-enumerable property':
        if (check_true(input instanceof Object, 'input instanceof Object') &&
            check_true(!(input instanceof Array), '!(input instanceof Array)') &&
            check_true(!('foo' in input), "!('foo' in input)")) {
          input = {};
          Object.defineProperty(input, 'foo', {value:'bar', enumerable:false, writable:true, configurable:true});
          port.postMessage(input);
          close();
        }
        break;
      case 'Object with non-writable property':
        if (check_true(input instanceof Object, 'input instanceof Object') &&
            check_true(!(input instanceof Array), '!(input instanceof Array)') &&
            check_true(input.foo === 'bar', "input.foo === bar")) {
          input.foo += ' baz';
          if (check_true(input.foo === 'bar baz', "input.foo === 'bar baz'")) {
            input = {};
            Object.defineProperty(input, 'foo', {value:'bar', enumerable:true, writable:false, configurable:true});
            port.postMessage(input);
            close();
          }
        }
        break;
      case 'Object with non-configurable property':
        if (check_true(input instanceof Object, 'input instanceof Object') &&
            check_true(!(input instanceof Array), '!(input instanceof Array)') &&
            check_true(input.foo === 'bar', "input.foo === bar")) {
          delete input.foo;
          if (check_true(!('foo' in input), "!('foo' in input)")) {
            input = {};
            Object.defineProperty(input, 'foo', {value:'bar', enumerable:true, writable:true, configurable:false});
            port.postMessage(input);
            close();
          }
        }
        break;

      default:
        port.postMessage('FAIL: unknown test');
        close();
    }
    if (log.length > 0) {
      port.postMessage('FAIL '+log);
      close();
    }
  } catch (ex) {
    port.postMessage('FAIL '+ex);
    close();
  }
}

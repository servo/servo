function createWorker(msg) {
  // `type` is defined in the test case itself
  if (type == 'dedicated')
    return new Worker('dedicated.js#'+encodeURIComponent(msg));
  else if (type == 'shared')
    return (new SharedWorker('shared.js#'+encodeURIComponent(msg))).port;
  else
    assert_unreached('invalid or missing `type`');
}

function check(msg, input, callback, test_obj) {
  if (!test_obj)
    test_obj = async_test(msg);
  test_obj.step(function() {
    var w = createWorker(msg);
    if (typeof input === 'function')
      input = this.step(input);
    w.postMessage(input);
    w.onmessage = this.step_func(function(ev) { callback(ev.data, input, this); });
  });
}

function compare_primitive(actual, input, test_obj) {
  assert_equals(actual, input);
  if (test_obj)
    test_obj.done();
}
function compare_Array(callback, callback_is_async) {
  return function(actual, input, test_obj) {
    if (typeof actual === 'string')
      assert_unreached(actual);
    assert_true(actual instanceof Array, 'instanceof Array');
    assert_not_equals(actual, input);
    assert_equals(actual.length, input.length, 'length');
    callback(actual, input);
    if (test_obj && !callback_is_async)
      test_obj.done();
  }
}

function compare_Object(callback, callback_is_async) {
  return function(actual, input, test_obj) {
    if (typeof actual === 'string')
      assert_unreached(actual);
    assert_true(actual instanceof Object, 'instanceof Object');
    assert_false(actual instanceof Array, 'instanceof Array');
    assert_not_equals(actual, input);
    callback(actual, input);
    if (test_obj && !callback_is_async)
      test_obj.done();
  }
}

function enumerate_props(compare_func, test_obj) {
  return function(actual, input) {
    for (var x in input) {
      compare_func(actual[x], input[x], test_obj);
    }
  };
}

check('primitive undefined', undefined, compare_primitive);
check('primitive null', null, compare_primitive);
check('primitive true', true, compare_primitive);
check('primitive false', false, compare_primitive);
check('primitive string, empty string', '', compare_primitive);
check('primitive string, lone high surrogate', '\uD800', compare_primitive);
check('primitive string, lone low surrogate', '\uDC00', compare_primitive);
check('primitive string, NUL', '\u0000', compare_primitive);
check('primitive string, astral character', '\uDBFF\uDFFD', compare_primitive);
check('primitive number, 0.2', 0.2, compare_primitive);
check('primitive number, 0', 0, compare_primitive);
check('primitive number, -0', -0, compare_primitive);
check('primitive number, NaN', NaN, compare_primitive);
check('primitive number, Infinity', Infinity, compare_primitive);
check('primitive number, -Infinity', -Infinity, compare_primitive);
check('primitive number, 9007199254740992', 9007199254740992, compare_primitive);
check('primitive number, -9007199254740992', -9007199254740992, compare_primitive);
check('primitive number, 9007199254740994', 9007199254740994, compare_primitive);
check('primitive number, -9007199254740994', -9007199254740994, compare_primitive);

check('Array primitives', [undefined,
                           null,
                           true,
                           false,
                           '',
                           '\uD800',
                           '\uDC00',
                           '\u0000',
                           '\uDBFF\uDFFD',
                           0.2,
                           0,
                           -0,
                           NaN,
                           Infinity,
                           -Infinity,
                           9007199254740992,
                           -9007199254740992,
                           9007199254740994,
                           -9007199254740994], compare_Array(enumerate_props(compare_primitive)));
check('Object primitives', {'undefined':undefined,
                           'null':null,
                           'true':true,
                           'false':false,
                           'empty':'',
                           'high surrogate':'\uD800',
                           'low surrogate':'\uDC00',
                           'nul':'\u0000',
                           'astral':'\uDBFF\uDFFD',
                           '0.2':0.2,
                           '0':0,
                           '-0':-0,
                           'NaN':NaN,
                           'Infinity':Infinity,
                           '-Infinity':-Infinity,
                           '9007199254740992':9007199254740992,
                           '-9007199254740992':-9007199254740992,
                           '9007199254740994':9007199254740994,
                           '-9007199254740994':-9007199254740994}, compare_Object(enumerate_props(compare_primitive)));

function compare_Boolean(actual, input, test_obj) {
  if (typeof actual === 'string')
    assert_unreached(actual);
  assert_true(actual instanceof Boolean, 'instanceof Boolean');
  assert_equals(String(actual), String(input), 'converted to primitive');
  assert_not_equals(actual, input);
  if (test_obj)
    test_obj.done();
}
check('Boolean true', new Boolean(true), compare_Boolean);
check('Boolean false', new Boolean(false), compare_Boolean);
check('Array Boolean objects', [new Boolean(true), new Boolean(false)], compare_Array(enumerate_props(compare_Boolean)));
check('Object Boolean objects', {'true':new Boolean(true), 'false':new Boolean(false)}, compare_Object(enumerate_props(compare_Boolean)));

function compare_obj(what) {
  var Type = window[what];
  return function(actual, input, test_obj) {
    if (typeof actual === 'string')
      assert_unreached(actual);
    assert_true(actual instanceof Type, 'instanceof '+what);
    assert_equals(Type(actual), Type(input), 'converted to primitive');
    assert_not_equals(actual, input);
    if (test_obj)
      test_obj.done();
  };
}
check('String empty string', new String(''), compare_obj('String'));
check('String lone high surrogate', new String('\uD800'), compare_obj('String'));
check('String lone low surrogate', new String('\uDC00'), compare_obj('String'));
check('String NUL', new String('\u0000'), compare_obj('String'));
check('String astral character', new String('\uDBFF\uDFFD'), compare_obj('String'));
check('Array String objects', [new String(''),
                               new String('\uD800'),
                               new String('\uDC00'),
                               new String('\u0000'),
                               new String('\uDBFF\uDFFD')], compare_Array(enumerate_props(compare_obj('String'))));
check('Object String objects', {'empty':new String(''),
                               'high surrogate':new String('\uD800'),
                               'low surrogate':new String('\uDC00'),
                               'nul':new String('\u0000'),
                               'astral':new String('\uDBFF\uDFFD')}, compare_Object(enumerate_props(compare_obj('String'))));

check('Number 0.2', new Number(0.2), compare_obj('Number'));
check('Number 0', new Number(0), compare_obj('Number'));
check('Number -0', new Number(-0), compare_obj('Number'));
check('Number NaN', new Number(NaN), compare_obj('Number'));
check('Number Infinity', new Number(Infinity), compare_obj('Number'));
check('Number -Infinity', new Number(-Infinity), compare_obj('Number'));
check('Number 9007199254740992', new Number(9007199254740992), compare_obj('Number'));
check('Number -9007199254740992', new Number(-9007199254740992), compare_obj('Number'));
check('Number 9007199254740994', new Number(9007199254740994), compare_obj('Number'));
check('Number -9007199254740994', new Number(-9007199254740994), compare_obj('Number'));
check('Array Number objects', [new Number(0.2),
                               new Number(0),
                               new Number(-0),
                               new Number(NaN),
                               new Number(Infinity),
                               new Number(-Infinity),
                               new Number(9007199254740992),
                               new Number(-9007199254740992),
                               new Number(9007199254740994),
                               new Number(-9007199254740994)], compare_Array(enumerate_props(compare_obj('Number'))));
check('Object Number objects', {'0.2':new Number(0.2),
                               '0':new Number(0),
                               '-0':new Number(-0),
                               'NaN':new Number(NaN),
                               'Infinity':new Number(Infinity),
                               '-Infinity':new Number(-Infinity),
                               '9007199254740992':new Number(9007199254740992),
                               '-9007199254740992':new Number(-9007199254740992),
                               '9007199254740994':new Number(9007199254740994),
                               '-9007199254740994':new Number(-9007199254740994)}, compare_Object(enumerate_props(compare_obj('Number'))));

function compare_Date(actual, input, test_obj) {
  if (typeof actual === 'string')
    assert_unreached(actual);
  assert_true(actual instanceof Date, 'instanceof Date');
  assert_equals(Number(actual), Number(input), 'converted to primitive');
  assert_not_equals(actual, input);
  if (test_obj)
    test_obj.done();
}
check('Date 0', new Date(0), compare_Date);
check('Date -0', new Date(-0), compare_Date);
check('Date -8.64e15', new Date(-8.64e15), compare_Date);
check('Date 8.64e15', new Date(8.64e15), compare_Date);
check('Array Date objects', [new Date(0),
                             new Date(-0),
                             new Date(-8.64e15),
                             new Date(8.64e15)], compare_Array(enumerate_props(compare_Date)));
check('Object Date objects', {'0':new Date(0),
                              '-0':new Date(-0),
                              '-8.64e15':new Date(-8.64e15),
                              '8.64e15':new Date(8.64e15)}, compare_Object(enumerate_props(compare_Date)));

function compare_RegExp(expected_source) {
  // XXX ES6 spec doesn't define exact serialization for `source` (it allows several ways to escape)
  return function(actual, input, test_obj) {
    if (typeof actual === 'string')
      assert_unreached(actual);
    assert_true(actual instanceof RegExp, 'instanceof RegExp');
    assert_equals(actual.global, input.global, 'global');
    assert_equals(actual.ignoreCase, input.ignoreCase, 'ignoreCase');
    assert_equals(actual.multiline, input.multiline, 'multiline');
    assert_equals(actual.source, expected_source, 'source');
    assert_equals(actual.sticky, input.sticky, 'sticky');
    assert_equals(actual.unicode, input.unicode, 'unicode');
    assert_equals(actual.lastIndex, 0, 'lastIndex');
    assert_not_equals(actual, input);
    if (test_obj)
      test_obj.done();
  }
}
function func_RegExp_flags_lastIndex() {
  var r = /foo/gim;
  r.lastIndex = 2;
  return r;
}
function func_RegExp_sticky() {
  return new RegExp('foo', 'y');
}
function func_RegExp_unicode() {
  return new RegExp('foo', 'u');
}
check('RegExp flags and lastIndex', func_RegExp_flags_lastIndex, compare_RegExp('foo'));
check('RegExp sticky flag', func_RegExp_sticky, compare_RegExp('foo'));
check('RegExp unicode flag', func_RegExp_unicode, compare_RegExp('foo'));
check('RegExp empty', new RegExp(''), compare_RegExp('(?:)'));
check('RegExp slash', new RegExp('/'), compare_RegExp('\\/'));
check('RegExp new line', new RegExp('\n'), compare_RegExp('\\n'));
check('Array RegExp object, RegExp flags and lastIndex', [func_RegExp_flags_lastIndex()], compare_Array(enumerate_props(compare_RegExp('foo'))));
check('Array RegExp object, RegExp sticky flag', function() { return [func_RegExp_sticky()]; }, compare_Array(enumerate_props(compare_RegExp('foo'))));
check('Array RegExp object, RegExp unicode flag', function() { return [func_RegExp_unicode()]; }, compare_Array(enumerate_props(compare_RegExp('foo'))));
check('Array RegExp object, RegExp empty', [new RegExp('')], compare_Array(enumerate_props(compare_RegExp('(?:)'))));
check('Array RegExp object, RegExp slash', [new RegExp('/')], compare_Array(enumerate_props(compare_RegExp('\\/'))));
check('Array RegExp object, RegExp new line', [new RegExp('\n')], compare_Array(enumerate_props(compare_RegExp('\\n'))));
check('Object RegExp object, RegExp flags and lastIndex', {'x':func_RegExp_flags_lastIndex()}, compare_Object(enumerate_props(compare_RegExp('foo'))));
check('Object RegExp object, RegExp sticky flag', function() { return {'x':func_RegExp_sticky()}; }, compare_Object(enumerate_props(compare_RegExp('foo'))));
check('Object RegExp object, RegExp unicode flag', function() { return {'x':func_RegExp_unicode()}; }, compare_Object(enumerate_props(compare_RegExp('foo'))));
check('Object RegExp object, RegExp empty', {'x':new RegExp('')}, compare_Object(enumerate_props(compare_RegExp('(?:)'))));
check('Object RegExp object, RegExp slash', {'x':new RegExp('/')}, compare_Object(enumerate_props(compare_RegExp('\\/'))));
check('Object RegExp object, RegExp new line', {'x':new RegExp('\n')}, compare_Object(enumerate_props(compare_RegExp('\\n'))));

function compare_Blob(actual, input, test_obj, expect_File) {
  if (typeof actual === 'string')
    assert_unreached(actual);
  assert_true(actual instanceof Blob, 'instanceof Blob');
  if (!expect_File)
    assert_false(actual instanceof File, 'instanceof File');
  assert_equals(actual.size, input.size, 'size');
  assert_equals(actual.type, input.type, 'type');
  assert_not_equals(actual, input);
  var ev_reader = new FileReader();
  var input_reader = new FileReader();
  var read_count = 0;
  var read_done = test_obj.step_func(function() {
    read_count++;
    if (read_count == 2) {
      var ev_result = ev_reader.result;
      var input_result = input_reader.result;
      assert_equals(ev_result.byteLength, input_result.byteLength, 'byteLength');
      var ev_view = new DataView(ev_result);
      var input_view = new DataView(input_result);
      for (var i = 0; i < ev_result.byteLength; ++i) {
        assert_equals(ev_view.getUint8(i), input_view.getUint8(i), 'getUint8('+i+')');
      }
      if (test_obj)
        test_obj.done();
    }
  });
  var read_error = test_obj.step_func(function() { assert_unreached('FileReader error'); });
  ev_reader.readAsArrayBuffer(actual);
  ev_reader.onload = read_done;
  ev_reader.onabort = ev_reader.onerror = read_error;
  input_reader.readAsArrayBuffer(input);
  input_reader.onload = read_done;
  input_reader.onabort = input_reader.onerror = read_error;
}
function func_Blob_basic() {
  return new Blob(['foo'], {type:'text/x-bar'});
}
check('Blob basic', func_Blob_basic, compare_Blob);

function b(str) {
  return parseInt(str, 2);
}
function encode_cesu8(codeunits) {
  // http://www.unicode.org/reports/tr26/ section 2.2
  // only the 3-byte form is supported
  var rv = [];
  codeunits.forEach(function(codeunit) {
    rv.push(b('11100000') + ((codeunit & b('1111000000000000')) >> 12));
    rv.push(b('10000000') + ((codeunit & b('0000111111000000')) >> 6));
    rv.push(b('10000000') +  (codeunit & b('0000000000111111')));
  });
  return rv;
}
function func_Blob_bytes(arr) {
  return function() {
    var buffer = new ArrayBuffer(arr.length);
    var view = new DataView(buffer);
    for (var i = 0; i < arr.length; ++i) {
      view.setUint8(i, arr[i]);
    }
    return new Blob([view]);
  };
}
check('Blob unpaired high surrogate (invalid utf-8)', func_Blob_bytes(encode_cesu8([0xD800])), compare_Blob);
check('Blob unpaired low surrogate (invalid utf-8)', func_Blob_bytes(encode_cesu8([0xDC00])), compare_Blob);
check('Blob paired surrogates (invalid utf-8)', func_Blob_bytes(encode_cesu8([0xD800, 0xDC00])), compare_Blob);

function func_Blob_empty() {
  return new Blob(['']);
}
check('Blob empty', func_Blob_empty , compare_Blob);
function func_Blob_NUL() {
  return new Blob(['\u0000']);
}
check('Blob NUL', func_Blob_NUL, compare_Blob);

async_test(function(test_obj) {
  check(test_obj.name, [test_obj.step(func_Blob_basic)], compare_Array(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Array Blob object, Blob basic');
async_test(function(test_obj) {
  check(test_obj.name, [test_obj.step(func_Blob_bytes([0xD800]))], compare_Array(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Array Blob object, Blob unpaired high surrogate (invalid utf-8)');
async_test(function(test_obj) {
  check(test_obj.name, [test_obj.step(func_Blob_bytes([0xDC00]))], compare_Array(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Array Blob object, Blob unpaired low surrogate (invalid utf-8)');
async_test(function(test_obj) {
  check(test_obj.name, [test_obj.step(func_Blob_bytes([0xD800, 0xDC00]))], compare_Array(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Array Blob object, Blob paired surrogates (invalid utf-8)');
async_test(function(test_obj) {
  check(test_obj.name, [test_obj.step(func_Blob_empty)], compare_Array(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Array Blob object, Blob empty');
async_test(function(test_obj) {
  check(test_obj.name, [test_obj.step(func_Blob_NUL)], compare_Array(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Array Blob object, Blob NUL');

async_test(function(test_obj) {
  check(test_obj.name, {'x':test_obj.step(func_Blob_basic)}, compare_Object(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Object Blob object, Blob basic');
async_test(function(test_obj) {
  check(test_obj.name, {'x':test_obj.step(func_Blob_bytes([0xD800]))}, compare_Object(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Object Blob object, Blob unpaired high surrogate (invalid utf-8)');
async_test(function(test_obj) {
  check(test_obj.name, {'x':test_obj.step(func_Blob_bytes([0xDC00]))}, compare_Object(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Object Blob object, Blob unpaired low surrogate (invalid utf-8)');
async_test(function(test_obj) {
  check(test_obj.name, {'x':test_obj.step(func_Blob_bytes([0xD800, 0xDC00]))}, compare_Object(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Object Blob object, Blob paired surrogates (invalid utf-8)');
async_test(function(test_obj) {
  check(test_obj.name, {'x':test_obj.step(func_Blob_empty)}, compare_Object(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Object Blob object, Blob empty');
async_test(function(test_obj) {
  check(test_obj.name, {'x':test_obj.step(func_Blob_NUL)}, compare_Object(enumerate_props(compare_Blob, test_obj), true), test_obj);
}, 'Object Blob object, Blob NUL');

function compare_File(actual, input, test_obj) {
  assert_true(actual instanceof File, 'instanceof File');
  assert_equals(actual.name, input.name, 'name');
  assert_equals(actual.lastModified, input.lastModified, 'lastModified');
  compare_Blob(actual, input, test_obj, true);
}
function func_File_basic() {
  return new File(['foo'], 'bar', {type:'text/x-bar', lastModified:42});
}
check('File basic', func_File_basic, compare_File);

function compare_FileList(actual, input, test_obj) {
  if (typeof actual === 'string')
    assert_unreached(actual);
  assert_true(actual instanceof FileList, 'instanceof FileList');
  assert_equals(actual.length, input.length, 'length');
  assert_not_equals(actual, input);
  // XXX when there's a way to populate or construct a FileList,
  // check the items in the FileList
  if (test_obj)
    test_obj.done();
}
function func_FileList_empty() {
  var input = document.createElement('input');
  input.type = 'file';
  return input.files;
}
check('FileList empty', func_FileList_empty, compare_FileList);
check('Array FileList object, FileList empty', [func_FileList_empty], compare_Array(enumerate_props(compare_FileList)));
check('Object FileList object, FileList empty', {'x':func_FileList_empty}, compare_Object(enumerate_props(compare_FileList)));

function compare_ArrayBufferView(view) {
  var Type = window[view];
  return function(actual, input, test_obj) {
    if (typeof actual === 'string')
      assert_unreached(actual);
    assert_true(actual instanceof Type, 'instanceof '+view);
    assert_equals(actual.length, input.length, 'length');
    assert_not_equals(actual.buffer, input.buffer, 'buffer');
    for (var i = 0; i < actual.length; ++i) {
      assert_equals(actual[i], input[i], 'actual['+i+']');
    }
    if (test_obj)
      test_obj.done();
  };
}
function compare_ImageData(actual, input, test_obj) {
  if (typeof actual === 'string')
    assert_unreached(actual);
  assert_equals(actual.width, input.width, 'width');
  assert_equals(actual.height, input.height, 'height');
  assert_not_equals(actual.data, input.data, 'data');
  compare_ArrayBufferView('Uint8ClampedArray')(actual.data, input.data, null);
  if (test_obj)
    test_obj.done();
}
function func_ImageData_1x1_transparent_black() {
  var canvas = document.createElement('canvas');
  var ctx = canvas.getContext('2d');
  return ctx.createImageData(1, 1);
}
check('ImageData 1x1 transparent black', func_ImageData_1x1_transparent_black, compare_ImageData);
function func_ImageData_1x1_non_transparent_non_black() {
  var canvas = document.createElement('canvas');
  var ctx = canvas.getContext('2d');
  var imagedata = ctx.createImageData(1, 1);
  imagedata.data[0] = 100;
  imagedata.data[1] = 101;
  imagedata.data[2] = 102;
  imagedata.data[3] = 103;
  return imagedata;
}
check('ImageData 1x1 non-transparent non-black', func_ImageData_1x1_non_transparent_non_black, compare_ImageData);
async_test(function(test_obj) {
  check(test_obj.name, [test_obj.step(func_ImageData_1x1_transparent_black)], compare_Array(enumerate_props(compare_ImageData)), test_obj);
}, 'Array ImageData object, ImageData 1x1 transparent black');
async_test(function(test_obj) {
  check(test_obj.name, [test_obj.step(func_ImageData_1x1_non_transparent_non_black)], compare_Array(enumerate_props(compare_ImageData)), test_obj);
}, 'Array ImageData object, ImageData 1x1 non-transparent non-black');
async_test(function(test_obj) {
  check(test_obj.name, {'x':test_obj.step(func_ImageData_1x1_transparent_black)}, compare_Object(enumerate_props(compare_ImageData)), test_obj);
}, 'Object ImageData object, ImageData 1x1 transparent black');
async_test(function(test_obj) {
  check(test_obj.name, {'x':test_obj.step(func_ImageData_1x1_non_transparent_non_black)}, compare_Object(enumerate_props(compare_ImageData)), test_obj);
}, 'Object ImageData object, ImageData 1x1 non-transparent non-black');

function compare_ImageBitmap(actual, input, test_obj) {
  if (typeof actual === 'string')
    assert_unreached(actual);
  assert_equals(actual instanceof ImageBitmap, 'instanceof ImageBitmap');
  assert_not_equals(actual, input);
  // XXX paint the ImageBitmap on a canvas and check the data
  if (test_obj)
    test_obj.done();
}
function get_canvas_1x1_transparent_black() {
  var canvas = document.createElement('canvas');
  canvas.width = 1;
  canvas.height = 1;
  return canvas;
}
async_test(function(test_obj) {
  var canvas = get_canvas_1x1_transparent_black();
  createImageBitmap(canvas, function(image) { check(test_obj.name, image, compare_ImageBitmap, test_obj); });
}, 'ImageBitmap 1x1 transparent black');
function get_canvas_1x1_non_transparent_non_black() {
  var canvas = document.createElement('canvas');
  canvas.width = 1;
  canvas.height = 1;
  var ctx = canvas.getContext('2d');
  var imagedata = ctx.getImageData(0, 0, 1, 1);
  imagedata.data[0] = 100;
  imagedata.data[1] = 101;
  imagedata.data[2] = 102;
  imagedata.data[3] = 103;
  return canvas;
}
async_test(function(test_obj) {
  var canvas = get_canvas_1x1_non_transparent_non_black();
  createImageBitmap(canvas, function(image) { check(test_obj.name, image, compare_ImageBitmap, test_obj); });
}, 'ImageBitmap 1x1 non-transparent non-black');

async_test(function(test_obj) {
  var canvas = get_canvas_1x1_transparent_black();
  createImageBitmap(canvas, function(image) { check(test_obj.name, [image], compare_Array(enumerate_props(compare_ImageBitmap)), test_obj); });
}, 'Array ImageBitmap object, ImageBitmap 1x1 transparent black');
async_test(function(test_obj) {
  var canvas = get_canvas_1x1_non_transparent_non_black();
  createImageBitmap(canvas, function(image) { check(test_obj.name, [image], compare_Array(enumerate_props(compare_ImageBitmap)), test_obj); });
}, 'Array ImageBitmap object, ImageBitmap 1x1 non-transparent non-black');

async_test(function(test_obj) {
  var canvas = get_canvas_1x1_transparent_black();
  createImageBitmap(canvas, function(image) { check(test_obj.name, {'x':image}, compare_Object(enumerate_props(compare_ImageBitmap)), test_obj); });
}, 'Object ImageBitmap object, ImageBitmap 1x1 transparent black');
async_test(function(test_obj) {
  var canvas = get_canvas_1x1_non_transparent_non_black();
  createImageBitmap(canvas, function(image) { check(test_obj.name, {'x':image}, compare_Object(enumerate_props(compare_ImageBitmap)), test_obj); });
}, 'Object ImageBitmap object, ImageBitmap 1x1 non-transparent non-black');

check('Array sparse', new Array(10), compare_Array(enumerate_props(compare_primitive)));
check('Array with non-index property', function() {
  var rv = [];
  rv.foo = 'bar';
  return rv;
}, compare_Array(enumerate_props(compare_primitive)));
check('Object with index property and length', {'0':'foo', 'length':1}, compare_Object(enumerate_props(compare_primitive)));
function check_circular_property(prop) {
  return function(actual) {
    assert_equals(actual[prop], actual);
  };
}
check('Array with circular reference', function() {
  var rv = [];
  rv[0] = rv;
  return rv;
}, compare_Array(check_circular_property('0')));
check('Object with circular reference', function() {
  var rv = {};
  rv['x'] = rv;
  return rv;
}, compare_Object(check_circular_property('x')));
function check_identical_property_values(prop1, prop2) {
  return function(actual) {
    assert_equals(actual[prop1], actual[prop2]);
  };
}
check('Array with identical property values', function() {
  var obj = {}
  return [obj, obj];
}, compare_Array(check_identical_property_values('0', '1')));
check('Object with identical property values', function() {
  var obj = {}
  return {'x':obj, 'y':obj};
}, compare_Object(check_identical_property_values('x', 'y')));

function check_absent_property(prop) {
  return function(actual) {
    assert_false(prop in actual);
  };
}
check('Object with property on prototype', function() {
  var Foo = function() {};
  Foo.prototype = {'foo':'bar'};
  return new Foo();
}, compare_Object(check_absent_property('foo')));

check('Object with non-enumerable property', function() {
  var rv = {};
  Object.defineProperty(rv, 'foo', {value:'bar', enumerable:false, writable:true, configurable:true});
  return rv;
}, compare_Object(check_absent_property('foo')));

function check_writable_property(prop) {
  return function(actual, input) {
    assert_equals(actual[prop], input[prop]);
    actual[prop] += ' baz';
    assert_equals(actual[prop], input[prop] + ' baz');
  };
}
check('Object with non-writable property', function() {
  var rv = {};
  Object.defineProperty(rv, 'foo', {value:'bar', enumerable:true, writable:false, configurable:true});
  return rv;
}, compare_Object(check_writable_property('foo')));

function check_configurable_property(prop) {
  return function(actual, input) {
    assert_equals(actual[prop], input[prop]);
    delete actual[prop];
    assert_false('prop' in actual);
  };
}
check('Object with non-configurable property', function() {
  var rv = {};
  Object.defineProperty(rv, 'foo', {value:'bar', enumerable:true, writable:true, configurable:false});
  return rv;
}, compare_Object(check_configurable_property('foo')));

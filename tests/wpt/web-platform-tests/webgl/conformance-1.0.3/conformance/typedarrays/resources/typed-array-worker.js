/*
** Copyright (c) 2013 The Khronos Group Inc.
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

function constructTypedArray(type, data) {
  if (type == 'Int8Array') {
    return new Int8Array(data);
  } else if (type == 'Uint8Array') {
    return new Uint8Array(data);
  } else if (type == 'Uint8ClampedArray') {
    return new Uint8ClampedArray(data);
  } else if (type == 'Int16Array') {
    return new Int16Array(data);
  } else if (type == 'Uint16Array') {
    return new Uint16Array(data);
  } else if (type == 'Int32Array') {
    return new Int32Array(data);
  } else if (type == 'Uint32Array') {
    return new Uint32Array(data);
  } else if (type == 'Float32Array') {
    return new Float32Array(data);
  } else if (type == 'Float64Array') {
    return new Float64Array(data);
  }
}

function constructDataView(subType, elementSizeInBytes, data) {
  var setter = "set" + subType;
  var byteOffset = 0;
  var buffer = new ArrayBuffer(elementSizeInBytes * data.length);
  var dataView = new DataView(buffer);
  for (var ii = 0; ii < data.length; ++ii) {
    dataView[setter](byteOffset, data[ii]);
    byteOffset += elementSizeInBytes;
  }
  return dataView;
}

onmessage = function(event) {
  var message = event.data;
  if (message.command == 'copy' ||
      message.command == 'transfer' ||
      message.command == 'copyBuffer' ||
      message.command == 'transferBuffer') {
    var view;
    if (message.type != 'DataView') {
      view = constructTypedArray(message.type, message.data);
    } else {
      view = constructDataView(message.subType, message.elementSizeInBytes, message.data);
    }
    var valueToSend;
    if (message.command == 'copy' ||
        message.command == 'transfer') {
      valueToSend = view;
    } else {
      valueToSend = view.buffer;
    }
    var transferablesToSend = undefined;
    if (message.command == 'transfer' ||
        message.command == 'transferBuffer') {
      transferablesToSend = [ view.buffer ];
    }
    postMessage(valueToSend, transferablesToSend);
  } else if (message.command == 'pong') {
    postMessage(message.data, message.transferables);
  } else if (message.command == 'ignore') {
  } else {
    postMessage('error: unknown message');
  }
};

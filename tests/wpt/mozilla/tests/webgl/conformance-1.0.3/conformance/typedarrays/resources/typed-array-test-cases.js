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

// The "name" attribute is a concession to browsers which don't
// implement the "name" property on function objects.
var testCases =
  [ {name: "Float32Array",
     unsigned: false,
     integral: false,
     elementSizeInBytes: 4,
     testValues:     [ -500.5, 500.5 ],
     expectedValues: [ -500.5, 500.5 ]
    },
    {name: "Float64Array",
     unsigned: false,
     integral: false,
     elementSizeInBytes: 8,
     testValues:     [ -500.5, 500.5 ],
     expectedValues: [ -500.5, 500.5 ]
    },
    {name: "Int8Array",
     unsigned: false,
     integral: true,
     elementSizeInBytes: 1,
     testValues:     [ -128, 127, -129,  128 ],
     expectedValues: [ -128, 127,  127, -128 ]
    },
    {name: "Int16Array",
     unsigned: false,
     integral: true,
     elementSizeInBytes: 2,
     testValues:     [ -32768, 32767, -32769,  32768 ],
     expectedValues: [ -32768, 32767,  32767, -32768 ]
    },
    {name: "Int32Array",
     unsigned: false,
     integral: true,
     elementSizeInBytes: 4,
     testValues:     [ -2147483648, 2147483647, -2147483649,  2147483648 ],
     expectedValues: [ -2147483648, 2147483647,  2147483647, -2147483648 ]
    },
    {name: "Uint8Array",
     unsigned: true,
     integral: true,
     elementSizeInBytes: 1,
     testValues:     [ 0, 255,  -1, 256 ],
     expectedValues: [ 0, 255, 255,   0 ]
    },
    {name: "Uint8ClampedArray",
     unsigned: true,
     integral: true,
     elementSizeInBytes: 1,
     testValues:     [ 0, 255,  -1, 256 ],
     expectedValues: [ 0, 255,   0, 255 ]
    },
    {name: "Uint16Array",
     unsigned: true,
     integral: true,
     elementSizeInBytes: 2,
     testValues:     [ 0, 65535,    -1, 65536 ],
     expectedValues: [ 0, 65535, 65535,     0 ]
    },
    {name: "Uint32Array",
     unsigned: true,
     integral: true,
     elementSizeInBytes: 4,
     testValues:     [ 0, 4294967295,         -1, 4294967296 ],
     expectedValues: [ 0, 4294967295, 4294967295,          0 ]
    }
  ];

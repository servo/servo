/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  DataView tests
info: bugzilla.mozilla.org/show_bug.cgi?id=575688
esid: pending
features: [host-gc-required]
---*/

function test(sharedMem) {
    function checkThrow(fun, type) {
        assert.throws(type, fun);
    }

    function bufferize(u8array) {
	if (!sharedMem)
	    return u8array.buffer;

	let v = new Uint8Array(new SharedArrayBuffer(u8array.buffer.byteLength));
	v.set(u8array);

	// Sanity checking
	assert.sameValue(v.buffer instanceof SharedArrayBuffer, true);
	assert.sameValue(v.buffer.byteLength, u8array.buffer.byteLength);
	assert.sameValue(u8array[0], v[0]);

	return v.buffer;
    }

    // testConstructor
    var buffer = bufferize(new Uint8Array([1, 2]));
    checkThrow(() => new DataView(buffer, 0, 3), RangeError);
    checkThrow(() => new DataView(buffer, 1, 2), RangeError);
    checkThrow(() => new DataView(buffer, 2, 1), RangeError);
    checkThrow(() => new DataView(buffer, 2147483649, 0), RangeError);
    checkThrow(() => new DataView(buffer, 0, 2147483649), RangeError);
    checkThrow(() => new DataView(), TypeError);
    checkThrow(() => new DataView(Object.create(new ArrayBuffer(5))), TypeError);

    // testGetMethods

    // testIntegerGets(start=0, length=16)
    var data1 = [0,1,2,3,0x64,0x65,0x66,0x67,0x80,0x81,0x82,0x83,252,253,254,255];
    var data1_r = data1.slice().reverse();
    var buffer1 = bufferize(new Uint8Array(data1));
    var view1 = new DataView(buffer1, 0, 16);
    var view = view1;
    assert.sameValue(view.getInt8(0), 0);
    assert.sameValue(view.getInt8(8), -128);
    assert.sameValue(view.getInt8(15), -1);
    assert.sameValue(view.getUint8(0), 0);
    assert.sameValue(view.getUint8(8), 128);
    assert.sameValue(view.getUint8(15), 255);
    //   Little endian.
    assert.sameValue(view.getInt16(0, true), 256);
    assert.sameValue(view.getInt16(5, true), 0x6665);
    assert.sameValue(view.getInt16(9, true), -32127);
    assert.sameValue(view.getInt16(14, true), -2);
    // Big endian.
    assert.sameValue(view.getInt16(0), 1);
    assert.sameValue(view.getInt16(5), 0x6566);
    assert.sameValue(view.getInt16(9), -32382);
    assert.sameValue(view.getInt16(14), -257);
    // Little endian.
    assert.sameValue(view.getUint16(0, true), 256);
    assert.sameValue(view.getUint16(5, true), 0x6665);
    assert.sameValue(view.getUint16(9, true), 0x8281);
    assert.sameValue(view.getUint16(14, true), 0xfffe);
    // Big endian.
    assert.sameValue(view.getUint16(0), 1);
    assert.sameValue(view.getUint16(5), 0x6566);
    assert.sameValue(view.getUint16(9), 0x8182);
    assert.sameValue(view.getUint16(14), 0xfeff);
    // Little endian.
    assert.sameValue(view.getInt32(0, true), 0x3020100);
    assert.sameValue(view.getInt32(3, true), 0x66656403);
    assert.sameValue(view.getInt32(6, true), -2122291354);
    assert.sameValue(view.getInt32(9, true), -58490239);
    assert.sameValue(view.getInt32(12, true), -66052);
    // Big endian.
    assert.sameValue(view.getInt32(0), 0x10203);
    assert.sameValue(view.getInt32(3), 0x3646566);
    assert.sameValue(view.getInt32(6), 0x66678081);
    assert.sameValue(view.getInt32(9), -2122152964);
    assert.sameValue(view.getInt32(12), -50462977);
    // Little endian.
    assert.sameValue(view.getUint32(0, true), 0x3020100);
    assert.sameValue(view.getUint32(3, true), 0x66656403);
    assert.sameValue(view.getUint32(6, true), 0x81806766);
    assert.sameValue(view.getUint32(9, true), 0xfc838281);
    assert.sameValue(view.getUint32(12, true), 0xfffefdfc);
    // Big endian.
    assert.sameValue(view.getUint32(0), 0x10203);
    assert.sameValue(view.getUint32(3), 0x3646566);
    assert.sameValue(view.getUint32(6), 0x66678081);
    assert.sameValue(view.getUint32(9), 0x818283fc);
    assert.sameValue(view.getUint32(12), 0xfcfdfeff);

    // testFloatGets(start=0)

    // testFloatGet expected=10
    //   Little endian
    var data2 = [0,0,32,65];
    var data2_r = data2.slice().reverse();
    var buffer2 = bufferize(new Uint8Array(data2));
    view = new DataView(buffer2, 0, 4);
    assert.sameValue(view.getFloat32(0, true), 10);
    var buffer2_pad3 = bufferize(new Uint8Array(Array(3).concat(data2)));
    view = new DataView(buffer2_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, true), 10);
    var buffer2_pad7 = bufferize(new Uint8Array(Array(7).concat(data2)));
    view = new DataView(buffer2_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, true), 10);
    var buffer2_pad10 = bufferize(new Uint8Array(Array(10).concat(data2)));
    view = new DataView(buffer2_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, true), 10);
    //   Big endian.
    var buffer2_r = bufferize(new Uint8Array(data2_r));
    view = new DataView(buffer2_r, 0, 4);
    assert.sameValue(view.getFloat32(0, false), 10);
    var buffer2_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data2_r)));
    view = new DataView(buffer2_r_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, false), 10);
    var buffer2_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data2_r)));
    view = new DataView(buffer2_r_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, false), 10);
    var buffer2_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data2_r)));
    view = new DataView(buffer2_r_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, false), 10);

    // testFloatGet expected=1.2300000190734863
    //   Little endian
    var data3 = [164,112,157,63];
    var data3_r = data3.slice().reverse();
    var buffer3 = bufferize(new Uint8Array(data3));
    view = new DataView(buffer3, 0, 4);
    assert.sameValue(view.getFloat32(0, true), 1.2300000190734863);
    var buffer3_pad3 = bufferize(new Uint8Array(Array(3).concat(data3)));
    view = new DataView(buffer3_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, true), 1.2300000190734863);
    var buffer3_pad7 = bufferize(new Uint8Array(Array(7).concat(data3)));
    view = new DataView(buffer3_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, true), 1.2300000190734863);
    var buffer3_pad10 = bufferize(new Uint8Array(Array(10).concat(data3)));
    view = new DataView(buffer3_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, true), 1.2300000190734863);
    //   Big endian.
    var buffer3_r = bufferize(new Uint8Array(data3_r));
    view = new DataView(buffer3_r, 0, 4);
    assert.sameValue(view.getFloat32(0, false), 1.2300000190734863);
    var buffer3_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data3_r)));
    view = new DataView(buffer3_r_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, false), 1.2300000190734863);
    var buffer3_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data3_r)));
    view = new DataView(buffer3_r_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, false), 1.2300000190734863);
    var buffer3_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data3_r)));
    view = new DataView(buffer3_r_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, false), 1.2300000190734863);

    // testFloatGet expected=-45621.37109375
    //   Little endian
    var data4 = [95,53,50,199];
    var data4_r = data4.slice().reverse();
    var buffer4 = bufferize(new Uint8Array(data4));
    view = new DataView(buffer4, 0, 4);
    assert.sameValue(view.getFloat32(0, true), -45621.37109375);
    var buffer4_pad3 = bufferize(new Uint8Array(Array(3).concat(data4)));
    view = new DataView(buffer4_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, true), -45621.37109375);
    var buffer4_pad7 = bufferize(new Uint8Array(Array(7).concat(data4)));
    view = new DataView(buffer4_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, true), -45621.37109375);
    var buffer4_pad10 = bufferize(new Uint8Array(Array(10).concat(data4)));
    view = new DataView(buffer4_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, true), -45621.37109375);
    //   Big endian.
    var buffer4_r = bufferize(new Uint8Array(data4_r));
    view = new DataView(buffer4_r, 0, 4);
    assert.sameValue(view.getFloat32(0, false), -45621.37109375);
    var buffer4_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data4_r)));
    view = new DataView(buffer4_r_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, false), -45621.37109375);
    var buffer4_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data4_r)));
    view = new DataView(buffer4_r_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, false), -45621.37109375);
    var buffer4_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data4_r)));
    view = new DataView(buffer4_r_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, false), -45621.37109375);

    // testFloatGet expected=NaN
    //   Little endian
    var data5 = [255,255,255,127];
    var data5_r = data5.slice().reverse();
    var buffer5 = bufferize(new Uint8Array(data5));
    view = new DataView(buffer5, 0, 4);
    assert.sameValue(view.getFloat32(0, true), NaN);
    var buffer5_pad3 = bufferize(new Uint8Array(Array(3).concat(data5)));
    view = new DataView(buffer5_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, true), NaN);
    var buffer5_pad7 = bufferize(new Uint8Array(Array(7).concat(data5)));
    view = new DataView(buffer5_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, true), NaN);
    var buffer5_pad10 = bufferize(new Uint8Array(Array(10).concat(data5)));
    view = new DataView(buffer5_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, true), NaN);
    //   Big endian.
    var buffer5_r = bufferize(new Uint8Array(data5_r));
    view = new DataView(buffer5_r, 0, 4);
    assert.sameValue(view.getFloat32(0, false), NaN);
    var buffer5_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data5_r)));
    view = new DataView(buffer5_r_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, false), NaN);
    var buffer5_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data5_r)));
    view = new DataView(buffer5_r_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, false), NaN);
    var buffer5_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data5_r)));
    view = new DataView(buffer5_r_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, false), NaN);

    // testFloatGet expected=NaN
    //   Little endian
    var data6 = [255,255,255,255];
    var data6_r = data6.slice().reverse();
    var buffer6 = bufferize(new Uint8Array(data6));
    view = new DataView(buffer6, 0, 4);
    assert.sameValue(view.getFloat32(0, true), NaN);
    var buffer6_pad3 = bufferize(new Uint8Array(Array(3).concat(data6)));
    view = new DataView(buffer6_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, true), NaN);
    var buffer6_pad7 = bufferize(new Uint8Array(Array(7).concat(data6)));
    view = new DataView(buffer6_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, true), NaN);
    var buffer6_pad10 = bufferize(new Uint8Array(Array(10).concat(data6)));
    view = new DataView(buffer6_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, true), NaN);
    //   Big endian.
    var buffer6_r = bufferize(new Uint8Array(data6_r));
    view = new DataView(buffer6_r, 0, 4);
    assert.sameValue(view.getFloat32(0, false), NaN);
    var buffer6_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data6_r)));
    view = new DataView(buffer6_r_pad3, 0, 7);
    assert.sameValue(view.getFloat32(3, false), NaN);
    var buffer6_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data6_r)));
    view = new DataView(buffer6_r_pad7, 0, 11);
    assert.sameValue(view.getFloat32(7, false), NaN);
    var buffer6_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data6_r)));
    view = new DataView(buffer6_r_pad10, 0, 14);
    assert.sameValue(view.getFloat32(10, false), NaN);

    // testFloatGet expected=10
    //   Little endian
    var data7 = [0,0,0,0,0,0,36,64];
    var data7_r = data7.slice().reverse();
    var buffer7 = bufferize(new Uint8Array(data7));
    view = new DataView(buffer7, 0, 8);
    assert.sameValue(view.getFloat64(0, true), 10);
    var buffer7_pad3 = bufferize(new Uint8Array(Array(3).concat(data7)));
    view = new DataView(buffer7_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, true), 10);
    var buffer7_pad7 = bufferize(new Uint8Array(Array(7).concat(data7)));
    view = new DataView(buffer7_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, true), 10);
    var buffer7_pad10 = bufferize(new Uint8Array(Array(10).concat(data7)));
    view = new DataView(buffer7_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, true), 10);
    //   Big endian.
    var buffer7_r = bufferize(new Uint8Array(data7_r));
    view = new DataView(buffer7_r, 0, 8);
    assert.sameValue(view.getFloat64(0, false), 10);
    var buffer7_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data7_r)));
    view = new DataView(buffer7_r_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, false), 10);
    var buffer7_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data7_r)));
    view = new DataView(buffer7_r_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, false), 10);
    var buffer7_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data7_r)));
    view = new DataView(buffer7_r_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, false), 10);

    // testFloatGet expected=1.23
    //   Little endian
    var data8 = [174,71,225,122,20,174,243,63];
    var data8_r = data8.slice().reverse();
    var buffer8 = bufferize(new Uint8Array(data8));
    view = new DataView(buffer8, 0, 8);
    assert.sameValue(view.getFloat64(0, true), 1.23);
    var buffer8_pad3 = bufferize(new Uint8Array(Array(3).concat(data8)));
    view = new DataView(buffer8_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, true), 1.23);
    var buffer8_pad7 = bufferize(new Uint8Array(Array(7).concat(data8)));
    view = new DataView(buffer8_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, true), 1.23);
    var buffer8_pad10 = bufferize(new Uint8Array(Array(10).concat(data8)));
    view = new DataView(buffer8_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, true), 1.23);
    //   Big endian.
    var buffer8_r = bufferize(new Uint8Array(data8_r));
    view = new DataView(buffer8_r, 0, 8);
    assert.sameValue(view.getFloat64(0, false), 1.23);
    var buffer8_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data8_r)));
    view = new DataView(buffer8_r_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, false), 1.23);
    var buffer8_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data8_r)));
    view = new DataView(buffer8_r_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, false), 1.23);
    var buffer8_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data8_r)));
    view = new DataView(buffer8_r_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, false), 1.23);

    // testFloatGet expected=-6213576.4839
    //   Little endian
    var data9 = [181,55,248,30,242,179,87,193];
    var data9_r = data9.slice().reverse();
    var buffer9 = bufferize(new Uint8Array(data9));
    view = new DataView(buffer9, 0, 8);
    assert.sameValue(view.getFloat64(0, true), -6213576.4839);
    var buffer9_pad3 = bufferize(new Uint8Array(Array(3).concat(data9)));
    view = new DataView(buffer9_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, true), -6213576.4839);
    var buffer9_pad7 = bufferize(new Uint8Array(Array(7).concat(data9)));
    view = new DataView(buffer9_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, true), -6213576.4839);
    var buffer9_pad10 = bufferize(new Uint8Array(Array(10).concat(data9)));
    view = new DataView(buffer9_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, true), -6213576.4839);
    //   Big endian.
    var buffer9_r = bufferize(new Uint8Array(data9_r));
    view = new DataView(buffer9_r, 0, 8);
    assert.sameValue(view.getFloat64(0, false), -6213576.4839);
    var buffer9_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data9_r)));
    view = new DataView(buffer9_r_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, false), -6213576.4839);
    var buffer9_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data9_r)));
    view = new DataView(buffer9_r_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, false), -6213576.4839);
    var buffer9_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data9_r)));
    view = new DataView(buffer9_r_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, false), -6213576.4839);

    // testFloatGet expected=NaN
    //   Little endian
    var data10 = [255,255,255,255,255,255,255,127];
    var data10_r = data10.slice().reverse();
    var buffer10 = bufferize(new Uint8Array(data10));
    view = new DataView(buffer10, 0, 8);
    assert.sameValue(view.getFloat64(0, true), NaN);
    var buffer10_pad3 = bufferize(new Uint8Array(Array(3).concat(data10)));
    view = new DataView(buffer10_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, true), NaN);
    var buffer10_pad7 = bufferize(new Uint8Array(Array(7).concat(data10)));
    view = new DataView(buffer10_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, true), NaN);
    var buffer10_pad10 = bufferize(new Uint8Array(Array(10).concat(data10)));
    view = new DataView(buffer10_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, true), NaN);
    //   Big endian.
    var buffer10_r = bufferize(new Uint8Array(data10_r));
    view = new DataView(buffer10_r, 0, 8);
    assert.sameValue(view.getFloat64(0, false), NaN);
    var buffer10_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data10_r)));
    view = new DataView(buffer10_r_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, false), NaN);
    var buffer10_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data10_r)));
    view = new DataView(buffer10_r_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, false), NaN);
    var buffer10_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data10_r)));
    view = new DataView(buffer10_r_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, false), NaN);

    // testFloatGet expected=NaN
    //   Little endian
    var data11 = [255,255,255,255,255,255,255,255];
    var data11_r = data11.slice().reverse();
    var buffer11 = bufferize(new Uint8Array(data11));
    view = new DataView(buffer11, 0, 8);
    assert.sameValue(view.getFloat64(0, true), NaN);
    var buffer11_pad3 = bufferize(new Uint8Array(Array(3).concat(data11)));
    view = new DataView(buffer11_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, true), NaN);
    var buffer11_pad7 = bufferize(new Uint8Array(Array(7).concat(data11)));
    view = new DataView(buffer11_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, true), NaN);
    var buffer11_pad10 = bufferize(new Uint8Array(Array(10).concat(data11)));
    view = new DataView(buffer11_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, true), NaN);
    //   Big endian.
    var buffer11_r = bufferize(new Uint8Array(data11_r));
    view = new DataView(buffer11_r, 0, 8);
    assert.sameValue(view.getFloat64(0, false), NaN);
    var buffer11_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data11_r)));
    view = new DataView(buffer11_r_pad3, 0, 11);
    assert.sameValue(view.getFloat64(3, false), NaN);
    var buffer11_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data11_r)));
    view = new DataView(buffer11_r_pad7, 0, 15);
    assert.sameValue(view.getFloat64(7, false), NaN);
    var buffer11_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data11_r)));
    view = new DataView(buffer11_r_pad10, 0, 18);
    assert.sameValue(view.getFloat64(10, false), NaN);

    // testIntegerGets(start=3, length=2)
    var data12 = [31,32,33,0,1,2,3,100,101,102,103,128,129,130,131,252,253,254,255];
    var data12_r = data12.slice().reverse();
    var buffer12 = bufferize(new Uint8Array(data12));
    view = new DataView(buffer12, 3, 2);
    assert.sameValue(view.getInt8(0), 0);
    checkThrow(() => view.getInt8(8), RangeError);
    checkThrow(() => view.getInt8(15), RangeError);
    assert.sameValue(view.getUint8(0), 0);
    checkThrow(() => view.getUint8(8), RangeError);
    checkThrow(() => view.getUint8(15), RangeError);
    //   Little endian.
    assert.sameValue(view.getInt16(0, true), 256);
    checkThrow(() => view.getInt16(5, true), RangeError);
    checkThrow(() => view.getInt16(9, true), RangeError);
    checkThrow(() => view.getInt16(14, true), RangeError);
    // Big endian.
    assert.sameValue(view.getInt16(0), 1);
    checkThrow(() => view.getInt16(5), RangeError);
    checkThrow(() => view.getInt16(9), RangeError);
    checkThrow(() => view.getInt16(14), RangeError);
    // Little endian.
    assert.sameValue(view.getUint16(0, true), 256);
    checkThrow(() => view.getUint16(5, true), RangeError);
    checkThrow(() => view.getUint16(9, true), RangeError);
    checkThrow(() => view.getUint16(14, true), RangeError);
    // Big endian.
    assert.sameValue(view.getUint16(0), 1);
    checkThrow(() => view.getUint16(5), RangeError);
    checkThrow(() => view.getUint16(9), RangeError);
    checkThrow(() => view.getUint16(14), RangeError);
    // Little endian.
    checkThrow(() => view.getInt32(0, true), RangeError);
    checkThrow(() => view.getInt32(3, true), RangeError);
    checkThrow(() => view.getInt32(6, true), RangeError);
    checkThrow(() => view.getInt32(9, true), RangeError);
    checkThrow(() => view.getInt32(12, true), RangeError);
    // Big endian.
    checkThrow(() => view.getInt32(0), RangeError);
    checkThrow(() => view.getInt32(3), RangeError);
    checkThrow(() => view.getInt32(6), RangeError);
    checkThrow(() => view.getInt32(9), RangeError);
    checkThrow(() => view.getInt32(12), RangeError);
    // Little endian.
    checkThrow(() => view.getUint32(0, true), RangeError);
    checkThrow(() => view.getUint32(3, true), RangeError);
    checkThrow(() => view.getUint32(6, true), RangeError);
    checkThrow(() => view.getUint32(9, true), RangeError);
    checkThrow(() => view.getUint32(12, true), RangeError);
    // Big endian.
    checkThrow(() => view.getUint32(0), RangeError);
    checkThrow(() => view.getUint32(3), RangeError);
    checkThrow(() => view.getUint32(6), RangeError);
    checkThrow(() => view.getUint32(9), RangeError);
    checkThrow(() => view.getUint32(12), RangeError);

    // testFloatGets(start=3)

    // testFloatGet expected=10
    //   Little endian
    view = new DataView(buffer2, 3, 1);
    checkThrow(() => view.getFloat32(0, true), RangeError);
    view = new DataView(buffer2_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, true), RangeError);
    view = new DataView(buffer2_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, true), RangeError);
    view = new DataView(buffer2_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer2_r, 3, 1);
    checkThrow(() => view.getFloat32(0, false), RangeError);
    view = new DataView(buffer2_r_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, false), RangeError);
    view = new DataView(buffer2_r_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, false), RangeError);
    view = new DataView(buffer2_r_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, false), RangeError);

    // testFloatGet expected=1.2300000190734863
    //   Little endian
    view = new DataView(buffer3, 3, 1);
    checkThrow(() => view.getFloat32(0, true), RangeError);
    view = new DataView(buffer3_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, true), RangeError);
    view = new DataView(buffer3_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, true), RangeError);
    view = new DataView(buffer3_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer3_r, 3, 1);
    checkThrow(() => view.getFloat32(0, false), RangeError);
    view = new DataView(buffer3_r_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, false), RangeError);
    view = new DataView(buffer3_r_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, false), RangeError);
    view = new DataView(buffer3_r_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, false), RangeError);

    // testFloatGet expected=-45621.37109375
    //   Little endian
    view = new DataView(buffer4, 3, 1);
    checkThrow(() => view.getFloat32(0, true), RangeError);
    view = new DataView(buffer4_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, true), RangeError);
    view = new DataView(buffer4_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, true), RangeError);
    view = new DataView(buffer4_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer4_r, 3, 1);
    checkThrow(() => view.getFloat32(0, false), RangeError);
    view = new DataView(buffer4_r_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, false), RangeError);
    view = new DataView(buffer4_r_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, false), RangeError);
    view = new DataView(buffer4_r_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, false), RangeError);

    // testFloatGet expected=NaN
    //   Little endian
    view = new DataView(buffer5, 3, 1);
    checkThrow(() => view.getFloat32(0, true), RangeError);
    view = new DataView(buffer5_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, true), RangeError);
    view = new DataView(buffer5_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, true), RangeError);
    view = new DataView(buffer5_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer5_r, 3, 1);
    checkThrow(() => view.getFloat32(0, false), RangeError);
    view = new DataView(buffer5_r_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, false), RangeError);
    view = new DataView(buffer5_r_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, false), RangeError);
    view = new DataView(buffer5_r_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, false), RangeError);

    // testFloatGet expected=NaN
    //   Little endian
    view = new DataView(buffer6, 3, 1);
    checkThrow(() => view.getFloat32(0, true), RangeError);
    view = new DataView(buffer6_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, true), RangeError);
    view = new DataView(buffer6_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, true), RangeError);
    view = new DataView(buffer6_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer6_r, 3, 1);
    checkThrow(() => view.getFloat32(0, false), RangeError);
    view = new DataView(buffer6_r_pad3, 3, 4);
    checkThrow(() => view.getFloat32(3, false), RangeError);
    view = new DataView(buffer6_r_pad7, 3, 8);
    checkThrow(() => view.getFloat32(7, false), RangeError);
    view = new DataView(buffer6_r_pad10, 3, 11);
    checkThrow(() => view.getFloat32(10, false), RangeError);

    // testFloatGet expected=10
    //   Little endian
    view = new DataView(buffer7, 3, 5);
    checkThrow(() => view.getFloat64(0, true), RangeError);
    view = new DataView(buffer7_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, true), RangeError);
    view = new DataView(buffer7_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, true), RangeError);
    view = new DataView(buffer7_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer7_r, 3, 5);
    checkThrow(() => view.getFloat64(0, false), RangeError);
    view = new DataView(buffer7_r_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, false), RangeError);
    view = new DataView(buffer7_r_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, false), RangeError);
    view = new DataView(buffer7_r_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, false), RangeError);

    // testFloatGet expected=1.23
    //   Little endian
    view = new DataView(buffer8, 3, 5);
    checkThrow(() => view.getFloat64(0, true), RangeError);
    view = new DataView(buffer8_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, true), RangeError);
    view = new DataView(buffer8_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, true), RangeError);
    view = new DataView(buffer8_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer8_r, 3, 5);
    checkThrow(() => view.getFloat64(0, false), RangeError);
    view = new DataView(buffer8_r_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, false), RangeError);
    view = new DataView(buffer8_r_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, false), RangeError);
    view = new DataView(buffer8_r_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, false), RangeError);

    // testFloatGet expected=-6213576.4839
    //   Little endian
    view = new DataView(buffer9, 3, 5);
    checkThrow(() => view.getFloat64(0, true), RangeError);
    view = new DataView(buffer9_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, true), RangeError);
    view = new DataView(buffer9_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, true), RangeError);
    view = new DataView(buffer9_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer9_r, 3, 5);
    checkThrow(() => view.getFloat64(0, false), RangeError);
    view = new DataView(buffer9_r_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, false), RangeError);
    view = new DataView(buffer9_r_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, false), RangeError);
    view = new DataView(buffer9_r_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, false), RangeError);

    // testFloatGet expected=NaN
    //   Little endian
    view = new DataView(buffer10, 3, 5);
    checkThrow(() => view.getFloat64(0, true), RangeError);
    view = new DataView(buffer10_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, true), RangeError);
    view = new DataView(buffer10_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, true), RangeError);
    view = new DataView(buffer10_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer10_r, 3, 5);
    checkThrow(() => view.getFloat64(0, false), RangeError);
    view = new DataView(buffer10_r_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, false), RangeError);
    view = new DataView(buffer10_r_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, false), RangeError);
    view = new DataView(buffer10_r_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, false), RangeError);

    // testFloatGet expected=NaN
    //   Little endian
    view = new DataView(buffer11, 3, 5);
    checkThrow(() => view.getFloat64(0, true), RangeError);
    view = new DataView(buffer11_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, true), RangeError);
    view = new DataView(buffer11_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, true), RangeError);
    view = new DataView(buffer11_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, true), RangeError);
    //   Big endian.
    view = new DataView(buffer11_r, 3, 5);
    checkThrow(() => view.getFloat64(0, false), RangeError);
    view = new DataView(buffer11_r_pad3, 3, 8);
    checkThrow(() => view.getFloat64(3, false), RangeError);
    view = new DataView(buffer11_r_pad7, 3, 12);
    checkThrow(() => view.getFloat64(7, false), RangeError);
    view = new DataView(buffer11_r_pad10, 3, 15);
    checkThrow(() => view.getFloat64(10, false), RangeError);

    // testGetNegativeIndexes
    view = new DataView(buffer1, 0, 16);
    checkThrow(() => view.getInt8(-1), RangeError);
    checkThrow(() => view.getInt8(-2), RangeError);
    checkThrow(() => view.getUint8(-1), RangeError);
    checkThrow(() => view.getUint8(-2), RangeError);
    checkThrow(() => view.getInt16(-1), RangeError);
    checkThrow(() => view.getInt16(-2), RangeError);
    checkThrow(() => view.getInt16(-3), RangeError);
    checkThrow(() => view.getUint16(-1), RangeError);
    checkThrow(() => view.getUint16(-2), RangeError);
    checkThrow(() => view.getUint16(-3), RangeError);
    checkThrow(() => view.getInt32(-1), RangeError);
    checkThrow(() => view.getInt32(-3), RangeError);
    checkThrow(() => view.getInt32(-5), RangeError);
    checkThrow(() => view.getUint32(-1), RangeError);
    checkThrow(() => view.getUint32(-3), RangeError);
    checkThrow(() => view.getUint32(-5), RangeError);
    view = new DataView(buffer7, 0, 8);
    checkThrow(() => view.getFloat32(-1), RangeError);
    checkThrow(() => view.getFloat32(-3), RangeError);
    checkThrow(() => view.getFloat32(-5), RangeError);
    checkThrow(() => view.getFloat64(-1), RangeError);
    checkThrow(() => view.getFloat64(-5), RangeError);
    checkThrow(() => view.getFloat64(-9), RangeError);

    // Too large for signed 32 bit integer index
    checkThrow(() => view.getInt8(2147483648), RangeError);
    checkThrow(() => view.getInt8(2147483649), RangeError);
    checkThrow(() => view.getUint8(2147483648), RangeError);
    checkThrow(() => view.getUint8(2147483649), RangeError);
    checkThrow(() => view.getInt16(2147483648), RangeError);
    checkThrow(() => view.getInt16(2147483649), RangeError);
    checkThrow(() => view.getUint16(2147483648), RangeError);
    checkThrow(() => view.getUint16(2147483649), RangeError);
    checkThrow(() => view.getInt32(2147483648), RangeError);
    checkThrow(() => view.getInt32(2147483649), RangeError);
    checkThrow(() => view.getUint32(2147483648), RangeError);
    checkThrow(() => view.getUint32(2147483649), RangeError);
    checkThrow(() => view.getFloat32(2147483648), RangeError);
    checkThrow(() => view.getFloat32(2147483649), RangeError);
    checkThrow(() => view.getFloat64(2147483648), RangeError);
    checkThrow(() => view.getFloat64(2147483649), RangeError);

    // Test for wrong type of |this|
    checkThrow(() => view.getInt8.apply("dead", [0]), TypeError);
    checkThrow(() => view.getUint8.apply("puppies", [0]), TypeError);
    checkThrow(() => view.getInt16.apply("aren", [0]), TypeError);
    checkThrow(() => view.getUint16.apply("t", [0]), TypeError);
    checkThrow(() => view.getInt32.apply("much", [0]), TypeError);
    checkThrow(() => view.getUint32.apply("fun", [0]), TypeError);
    checkThrow(() => view.getFloat32.apply("(in", [0]), TypeError);
    checkThrow(() => view.getFloat64.apply("bed)", [0]), TypeError);
    checkThrow(() => view.setInt8.apply("dead", [0, 0]), TypeError);
    checkThrow(() => view.setUint8.apply("puppies", [0, 0]), TypeError);
    checkThrow(() => view.setInt16.apply("aren", [0, 0]), TypeError);
    checkThrow(() => view.setUint16.apply("t", [0, 0]), TypeError);
    checkThrow(() => view.setInt32.apply("much", [0, 0]), TypeError);
    checkThrow(() => view.setUint32.apply("fun", [0, 0]), TypeError);
    checkThrow(() => view.setFloat32.apply("(in", [0, 0]), TypeError);
    checkThrow(() => view.setFloat64.apply("bed)", [0, 0]), TypeError);

    // testSetMethods

    //   Test for set methods that work

    // testIntegerSets(start=0, length=16)
    var data13 = [204,204,204,204,204,204,204,204,204,204,204,204,204,204,204,204];
    var data13_r = data13.slice().reverse();
    var buffer13 = bufferize(new Uint8Array(data13));
    view = new DataView(buffer13, 0, 16);
    view.setInt8(0, 0);
    assert.sameValue(view.getInt8(0), 0);
    view.setInt8(8, -128);
    assert.sameValue(view.getInt8(8), -128);
    view.setInt8(15, -1);
    assert.sameValue(view.getInt8(15), -1);
    view.setUint8(0, 0);
    assert.sameValue(view.getUint8(0), 0);
    view.setUint8(8, 128);
    assert.sameValue(view.getUint8(8), 128);
    view.setUint8(15, 255);
    assert.sameValue(view.getUint8(15), 255);
    view.setInt16(0, 256, true);
    assert.sameValue(view.getInt16(0, true), 256);
    view.setInt16(5, 26213, true);
    assert.sameValue(view.getInt16(5, true), 26213);
    view.setInt16(9, -32127, true);
    assert.sameValue(view.getInt16(9, true), -32127);
    view.setInt16(14, -2, true);
    assert.sameValue(view.getInt16(14, true), -2);
    view.setInt16(0, 1);
    assert.sameValue(view.getInt16(0), 1);
    view.setInt16(5, 25958);
    assert.sameValue(view.getInt16(5), 25958);
    view.setInt16(9, -32382);
    assert.sameValue(view.getInt16(9), -32382);
    view.setInt16(14, -257);
    assert.sameValue(view.getInt16(14), -257);
    view.setUint16(0, 256, true);
    assert.sameValue(view.getUint16(0, true), 256);
    view.setUint16(5, 26213, true);
    assert.sameValue(view.getUint16(5, true), 26213);
    view.setUint16(9, 33409, true);
    assert.sameValue(view.getUint16(9, true), 33409);
    view.setUint16(14, 65534, true);
    assert.sameValue(view.getUint16(14, true), 65534);
    view.setUint16(0, 1);
    assert.sameValue(view.getUint16(0), 1);
    view.setUint16(5, 25958);
    assert.sameValue(view.getUint16(5), 25958);
    view.setUint16(9, 33154);
    assert.sameValue(view.getUint16(9), 33154);
    view.setUint16(14, 65279);
    assert.sameValue(view.getUint16(14), 65279);
    view.setInt32(0, 50462976, true);
    assert.sameValue(view.getInt32(0, true), 50462976);
    view.setInt32(3, 1717920771, true);
    assert.sameValue(view.getInt32(3, true), 1717920771);
    view.setInt32(6, -2122291354, true);
    assert.sameValue(view.getInt32(6, true), -2122291354);
    view.setInt32(9, -58490239, true);
    assert.sameValue(view.getInt32(9, true), -58490239);
    view.setInt32(12, -66052, true);
    assert.sameValue(view.getInt32(12, true), -66052);
    view.setInt32(0, 66051);
    assert.sameValue(view.getInt32(0), 66051);
    view.setInt32(3, 56911206);
    assert.sameValue(view.getInt32(3), 56911206);
    view.setInt32(6, 1718059137);
    assert.sameValue(view.getInt32(6), 1718059137);
    view.setInt32(9, -2122152964);
    assert.sameValue(view.getInt32(9), -2122152964);
    view.setInt32(12, -50462977);
    assert.sameValue(view.getInt32(12), -50462977);
    view.setUint32(0, 50462976, true);
    assert.sameValue(view.getUint32(0, true), 50462976);
    view.setUint32(3, 1717920771, true);
    assert.sameValue(view.getUint32(3, true), 1717920771);
    view.setUint32(6, 2172675942, true);
    assert.sameValue(view.getUint32(6, true), 2172675942);
    view.setUint32(9, 4236477057, true);
    assert.sameValue(view.getUint32(9, true), 4236477057);
    view.setUint32(12, 4294901244, true);
    assert.sameValue(view.getUint32(12, true), 4294901244);
    view.setUint32(0, 66051);
    assert.sameValue(view.getUint32(0), 66051);
    view.setUint32(3, 56911206);
    assert.sameValue(view.getUint32(3), 56911206);
    view.setUint32(6, 1718059137);
    assert.sameValue(view.getUint32(6), 1718059137);
    view.setUint32(9, 2172814332);
    assert.sameValue(view.getUint32(9), 2172814332);
    view.setUint32(12, 4244504319);
    assert.sameValue(view.getUint32(12), 4244504319);

    // testFloatSets(start=undefined)

    // testFloatSet expected=10
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat32(0, 10, true);
    assert.sameValue(view.getFloat32(0, true), 10);
    var buffer13_pad3 = bufferize(new Uint8Array(Array(3).concat(data13)));
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat32(3, 10, true);
    assert.sameValue(view.getFloat32(3, true), 10);
    var buffer13_pad7 = bufferize(new Uint8Array(Array(7).concat(data13)));
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat32(7, 10, true);
    assert.sameValue(view.getFloat32(7, true), 10);
    var buffer13_pad10 = bufferize(new Uint8Array(Array(10).concat(data13)));
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat32(10, 10, true);
    assert.sameValue(view.getFloat32(10, true), 10);
    //   Big endian.
    var buffer13_r = bufferize(new Uint8Array(data13_r));
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat32(0, 10, false);
    assert.sameValue(view.getFloat32(0, false), 10);
    var buffer13_r_pad3 = bufferize(new Uint8Array(Array(3).concat(data13_r)));
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat32(3, 10, false);
    assert.sameValue(view.getFloat32(3, false), 10);
    var buffer13_r_pad7 = bufferize(new Uint8Array(Array(7).concat(data13_r)));
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat32(7, 10, false);
    assert.sameValue(view.getFloat32(7, false), 10);
    var buffer13_r_pad10 = bufferize(new Uint8Array(Array(10).concat(data13_r)));
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat32(10, 10, false);
    assert.sameValue(view.getFloat32(10, false), 10);

    // testFloatSet expected=1.2300000190734863
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat32(0, 1.2300000190734863, true);
    assert.sameValue(view.getFloat32(0, true), 1.2300000190734863);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat32(3, 1.2300000190734863, true);
    assert.sameValue(view.getFloat32(3, true), 1.2300000190734863);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat32(7, 1.2300000190734863, true);
    assert.sameValue(view.getFloat32(7, true), 1.2300000190734863);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat32(10, 1.2300000190734863, true);
    assert.sameValue(view.getFloat32(10, true), 1.2300000190734863);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat32(0, 1.2300000190734863, false);
    assert.sameValue(view.getFloat32(0, false), 1.2300000190734863);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat32(3, 1.2300000190734863, false);
    assert.sameValue(view.getFloat32(3, false), 1.2300000190734863);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat32(7, 1.2300000190734863, false);
    assert.sameValue(view.getFloat32(7, false), 1.2300000190734863);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat32(10, 1.2300000190734863, false);
    assert.sameValue(view.getFloat32(10, false), 1.2300000190734863);

    // testFloatSet expected=-45621.37109375
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat32(0, -45621.37109375, true);
    assert.sameValue(view.getFloat32(0, true), -45621.37109375);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat32(3, -45621.37109375, true);
    assert.sameValue(view.getFloat32(3, true), -45621.37109375);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat32(7, -45621.37109375, true);
    assert.sameValue(view.getFloat32(7, true), -45621.37109375);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat32(10, -45621.37109375, true);
    assert.sameValue(view.getFloat32(10, true), -45621.37109375);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat32(0, -45621.37109375, false);
    assert.sameValue(view.getFloat32(0, false), -45621.37109375);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat32(3, -45621.37109375, false);
    assert.sameValue(view.getFloat32(3, false), -45621.37109375);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat32(7, -45621.37109375, false);
    assert.sameValue(view.getFloat32(7, false), -45621.37109375);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat32(10, -45621.37109375, false);
    assert.sameValue(view.getFloat32(10, false), -45621.37109375);

    // testFloatSet expected=NaN
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat32(0, NaN, true);
    assert.sameValue(view.getFloat32(0, true), NaN);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat32(3, NaN, true);
    assert.sameValue(view.getFloat32(3, true), NaN);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat32(7, NaN, true);
    assert.sameValue(view.getFloat32(7, true), NaN);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat32(10, NaN, true);
    assert.sameValue(view.getFloat32(10, true), NaN);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat32(0, NaN, false);
    assert.sameValue(view.getFloat32(0, false), NaN);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat32(3, NaN, false);
    assert.sameValue(view.getFloat32(3, false), NaN);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat32(7, NaN, false);
    assert.sameValue(view.getFloat32(7, false), NaN);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat32(10, NaN, false);
    assert.sameValue(view.getFloat32(10, false), NaN);

    // testFloatSet expected=-NaN
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat32(0, -NaN, true);
    assert.sameValue(view.getFloat32(0, true), -NaN);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat32(3, -NaN, true);
    assert.sameValue(view.getFloat32(3, true), -NaN);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat32(7, -NaN, true);
    assert.sameValue(view.getFloat32(7, true), -NaN);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat32(10, -NaN, true);
    assert.sameValue(view.getFloat32(10, true), -NaN);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat32(0, -NaN, false);
    assert.sameValue(view.getFloat32(0, false), -NaN);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat32(3, -NaN, false);
    assert.sameValue(view.getFloat32(3, false), -NaN);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat32(7, -NaN, false);
    assert.sameValue(view.getFloat32(7, false), -NaN);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat32(10, -NaN, false);
    assert.sameValue(view.getFloat32(10, false), -NaN);

    // testFloatSet expected=10
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat64(0, 10, true);
    assert.sameValue(view.getFloat64(0, true), 10);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat64(3, 10, true);
    assert.sameValue(view.getFloat64(3, true), 10);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat64(7, 10, true);
    assert.sameValue(view.getFloat64(7, true), 10);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat64(10, 10, true);
    assert.sameValue(view.getFloat64(10, true), 10);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat64(0, 10, false);
    assert.sameValue(view.getFloat64(0, false), 10);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat64(3, 10, false);
    assert.sameValue(view.getFloat64(3, false), 10);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat64(7, 10, false);
    assert.sameValue(view.getFloat64(7, false), 10);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat64(10, 10, false);
    assert.sameValue(view.getFloat64(10, false), 10);

    // testFloatSet expected=1.23
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat64(0, 1.23, true);
    assert.sameValue(view.getFloat64(0, true), 1.23);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat64(3, 1.23, true);
    assert.sameValue(view.getFloat64(3, true), 1.23);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat64(7, 1.23, true);
    assert.sameValue(view.getFloat64(7, true), 1.23);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat64(10, 1.23, true);
    assert.sameValue(view.getFloat64(10, true), 1.23);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat64(0, 1.23, false);
    assert.sameValue(view.getFloat64(0, false), 1.23);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat64(3, 1.23, false);
    assert.sameValue(view.getFloat64(3, false), 1.23);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat64(7, 1.23, false);
    assert.sameValue(view.getFloat64(7, false), 1.23);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat64(10, 1.23, false);
    assert.sameValue(view.getFloat64(10, false), 1.23);

    // testFloatSet expected=-6213576.4839
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat64(0, -6213576.4839, true);
    assert.sameValue(view.getFloat64(0, true), -6213576.4839);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat64(3, -6213576.4839, true);
    assert.sameValue(view.getFloat64(3, true), -6213576.4839);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat64(7, -6213576.4839, true);
    assert.sameValue(view.getFloat64(7, true), -6213576.4839);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat64(10, -6213576.4839, true);
    assert.sameValue(view.getFloat64(10, true), -6213576.4839);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat64(0, -6213576.4839, false);
    assert.sameValue(view.getFloat64(0, false), -6213576.4839);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat64(3, -6213576.4839, false);
    assert.sameValue(view.getFloat64(3, false), -6213576.4839);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat64(7, -6213576.4839, false);
    assert.sameValue(view.getFloat64(7, false), -6213576.4839);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat64(10, -6213576.4839, false);
    assert.sameValue(view.getFloat64(10, false), -6213576.4839);

    // testFloatSet expected=NaN
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat64(0, NaN, true);
    assert.sameValue(view.getFloat64(0, true), NaN);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat64(3, NaN, true);
    assert.sameValue(view.getFloat64(3, true), NaN);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat64(7, NaN, true);
    assert.sameValue(view.getFloat64(7, true), NaN);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat64(10, NaN, true);
    assert.sameValue(view.getFloat64(10, true), NaN);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat64(0, NaN, false);
    assert.sameValue(view.getFloat64(0, false), NaN);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat64(3, NaN, false);
    assert.sameValue(view.getFloat64(3, false), NaN);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat64(7, NaN, false);
    assert.sameValue(view.getFloat64(7, false), NaN);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat64(10, NaN, false);
    assert.sameValue(view.getFloat64(10, false), NaN);

    // testFloatSet expected=-NaN
    //   Little endian
    view = new DataView(buffer13, 0, 16);
    view.setFloat64(0, -NaN, true);
    assert.sameValue(view.getFloat64(0, true), -NaN);
    view = new DataView(buffer13_pad3, 0, 19);
    view.setFloat64(3, -NaN, true);
    assert.sameValue(view.getFloat64(3, true), -NaN);
    view = new DataView(buffer13_pad7, 0, 23);
    view.setFloat64(7, -NaN, true);
    assert.sameValue(view.getFloat64(7, true), -NaN);
    view = new DataView(buffer13_pad10, 0, 26);
    view.setFloat64(10, -NaN, true);
    assert.sameValue(view.getFloat64(10, true), -NaN);
    //   Big endian.
    view = new DataView(buffer13_r, 0, 16);
    view.setFloat64(0, -NaN, false);
    assert.sameValue(view.getFloat64(0, false), -NaN);
    view = new DataView(buffer13_r_pad3, 0, 19);
    view.setFloat64(3, -NaN, false);
    assert.sameValue(view.getFloat64(3, false), -NaN);
    view = new DataView(buffer13_r_pad7, 0, 23);
    view.setFloat64(7, -NaN, false);
    assert.sameValue(view.getFloat64(7, false), -NaN);
    view = new DataView(buffer13_r_pad10, 0, 26);
    view.setFloat64(10, -NaN, false);
    assert.sameValue(view.getFloat64(10, false), -NaN);

    // Test for set methods that might write beyond the range

    // testIntegerSets(start=3, length=2)
    view = new DataView(buffer13, 3, 2);
    view.setInt8(0, 0);
    assert.sameValue(view.getInt8(0), 0);
    checkThrow(() => view.setInt8(8, -128), RangeError);
    checkThrow(() => view.setInt8(15, -1), RangeError);
    view.setUint8(0, 0);
    assert.sameValue(view.getUint8(0), 0);
    checkThrow(() => view.setUint8(8, 128), RangeError);
    checkThrow(() => view.setUint8(15, 255), RangeError);
    view.setInt16(0, 256, true);
    assert.sameValue(view.getInt16(0, true), 256);
    checkThrow(() => view.setInt16(5, 26213, true), RangeError);
    checkThrow(() => view.setInt16(9, -32127, true), RangeError);
    checkThrow(() => view.setInt16(14, -2, true), RangeError);
    view.setInt16(0, 1);
    assert.sameValue(view.getInt16(0), 1);
    checkThrow(() => view.setInt16(5, 25958), RangeError);
    checkThrow(() => view.setInt16(9, -32382), RangeError);
    checkThrow(() => view.setInt16(14, -257), RangeError);
    view.setUint16(0, 256, true);
    assert.sameValue(view.getUint16(0, true), 256);
    checkThrow(() => view.setUint16(5, 26213, true), RangeError);
    checkThrow(() => view.setUint16(9, 33409, true), RangeError);
    checkThrow(() => view.setUint16(14, 65534, true), RangeError);
    view.setUint16(0, 1);
    assert.sameValue(view.getUint16(0), 1);
    checkThrow(() => view.setUint16(5, 25958), RangeError);
    checkThrow(() => view.setUint16(9, 33154), RangeError);
    checkThrow(() => view.setUint16(14, 65279), RangeError);
    checkThrow(() => view.setInt32(0, 50462976, true), RangeError);
    checkThrow(() => view.setInt32(3, 1717920771, true), RangeError);
    checkThrow(() => view.setInt32(6, -2122291354, true), RangeError);
    checkThrow(() => view.setInt32(9, -58490239, true), RangeError);
    checkThrow(() => view.setInt32(12, -66052, true), RangeError);
    checkThrow(() => view.setInt32(0, 66051), RangeError);
    checkThrow(() => view.setInt32(3, 56911206), RangeError);
    checkThrow(() => view.setInt32(6, 1718059137), RangeError);
    checkThrow(() => view.setInt32(9, -2122152964), RangeError);
    checkThrow(() => view.setInt32(12, -50462977), RangeError);
    checkThrow(() => view.setUint32(0, 50462976, true), RangeError);
    checkThrow(() => view.setUint32(3, 1717920771, true), RangeError);
    checkThrow(() => view.setUint32(6, 2172675942, true), RangeError);
    checkThrow(() => view.setUint32(9, 4236477057, true), RangeError);
    checkThrow(() => view.setUint32(12, 4294901244, true), RangeError);
    checkThrow(() => view.setUint32(0, 66051), RangeError);
    checkThrow(() => view.setUint32(3, 56911206), RangeError);
    checkThrow(() => view.setUint32(6, 1718059137), RangeError);
    checkThrow(() => view.setUint32(9, 2172814332), RangeError);
    checkThrow(() => view.setUint32(12, 4244504319), RangeError);

    // testFloatSets(start=7)

    // testFloatSet expected=10
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat32(0, 10, true);
    assert.sameValue(view.getFloat32(0, true), 10);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat32(3, 10, true);
    assert.sameValue(view.getFloat32(3, true), 10);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat32(7, 10, true);
    assert.sameValue(view.getFloat32(7, true), 10);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat32(10, 10, true);
    assert.sameValue(view.getFloat32(10, true), 10);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat32(0, 10, false);
    assert.sameValue(view.getFloat32(0, false), 10);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat32(3, 10, false);
    assert.sameValue(view.getFloat32(3, false), 10);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat32(7, 10, false);
    assert.sameValue(view.getFloat32(7, false), 10);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat32(10, 10, false);
    assert.sameValue(view.getFloat32(10, false), 10);

    // testFloatSet expected=1.2300000190734863
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat32(0, 1.2300000190734863, true);
    assert.sameValue(view.getFloat32(0, true), 1.2300000190734863);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat32(3, 1.2300000190734863, true);
    assert.sameValue(view.getFloat32(3, true), 1.2300000190734863);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat32(7, 1.2300000190734863, true);
    assert.sameValue(view.getFloat32(7, true), 1.2300000190734863);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat32(10, 1.2300000190734863, true);
    assert.sameValue(view.getFloat32(10, true), 1.2300000190734863);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat32(0, 1.2300000190734863, false);
    assert.sameValue(view.getFloat32(0, false), 1.2300000190734863);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat32(3, 1.2300000190734863, false);
    assert.sameValue(view.getFloat32(3, false), 1.2300000190734863);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat32(7, 1.2300000190734863, false);
    assert.sameValue(view.getFloat32(7, false), 1.2300000190734863);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat32(10, 1.2300000190734863, false);
    assert.sameValue(view.getFloat32(10, false), 1.2300000190734863);

    // testFloatSet expected=-45621.37109375
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat32(0, -45621.37109375, true);
    assert.sameValue(view.getFloat32(0, true), -45621.37109375);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat32(3, -45621.37109375, true);
    assert.sameValue(view.getFloat32(3, true), -45621.37109375);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat32(7, -45621.37109375, true);
    assert.sameValue(view.getFloat32(7, true), -45621.37109375);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat32(10, -45621.37109375, true);
    assert.sameValue(view.getFloat32(10, true), -45621.37109375);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat32(0, -45621.37109375, false);
    assert.sameValue(view.getFloat32(0, false), -45621.37109375);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat32(3, -45621.37109375, false);
    assert.sameValue(view.getFloat32(3, false), -45621.37109375);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat32(7, -45621.37109375, false);
    assert.sameValue(view.getFloat32(7, false), -45621.37109375);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat32(10, -45621.37109375, false);
    assert.sameValue(view.getFloat32(10, false), -45621.37109375);

    // testFloatSet expected=NaN
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat32(0, NaN, true);
    assert.sameValue(view.getFloat32(0, true), NaN);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat32(3, NaN, true);
    assert.sameValue(view.getFloat32(3, true), NaN);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat32(7, NaN, true);
    assert.sameValue(view.getFloat32(7, true), NaN);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat32(10, NaN, true);
    assert.sameValue(view.getFloat32(10, true), NaN);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat32(0, NaN, false);
    assert.sameValue(view.getFloat32(0, false), NaN);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat32(3, NaN, false);
    assert.sameValue(view.getFloat32(3, false), NaN);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat32(7, NaN, false);
    assert.sameValue(view.getFloat32(7, false), NaN);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat32(10, NaN, false);
    assert.sameValue(view.getFloat32(10, false), NaN);

    // testFloatSet expected=-NaN
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat32(0, -NaN, true);
    assert.sameValue(view.getFloat32(0, true), -NaN);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat32(3, -NaN, true);
    assert.sameValue(view.getFloat32(3, true), -NaN);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat32(7, -NaN, true);
    assert.sameValue(view.getFloat32(7, true), -NaN);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat32(10, -NaN, true);
    assert.sameValue(view.getFloat32(10, true), -NaN);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat32(0, -NaN, false);
    assert.sameValue(view.getFloat32(0, false), -NaN);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat32(3, -NaN, false);
    assert.sameValue(view.getFloat32(3, false), -NaN);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat32(7, -NaN, false);
    assert.sameValue(view.getFloat32(7, false), -NaN);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat32(10, -NaN, false);
    assert.sameValue(view.getFloat32(10, false), -NaN);

    // testFloatSet expected=10
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat64(0, 10, true);
    assert.sameValue(view.getFloat64(0, true), 10);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat64(3, 10, true);
    assert.sameValue(view.getFloat64(3, true), 10);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat64(7, 10, true);
    assert.sameValue(view.getFloat64(7, true), 10);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat64(10, 10, true);
    assert.sameValue(view.getFloat64(10, true), 10);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat64(0, 10, false);
    assert.sameValue(view.getFloat64(0, false), 10);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat64(3, 10, false);
    assert.sameValue(view.getFloat64(3, false), 10);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat64(7, 10, false);
    assert.sameValue(view.getFloat64(7, false), 10);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat64(10, 10, false);
    assert.sameValue(view.getFloat64(10, false), 10);

    // testFloatSet expected=1.23
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat64(0, 1.23, true);
    assert.sameValue(view.getFloat64(0, true), 1.23);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat64(3, 1.23, true);
    assert.sameValue(view.getFloat64(3, true), 1.23);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat64(7, 1.23, true);
    assert.sameValue(view.getFloat64(7, true), 1.23);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat64(10, 1.23, true);
    assert.sameValue(view.getFloat64(10, true), 1.23);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat64(0, 1.23, false);
    assert.sameValue(view.getFloat64(0, false), 1.23);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat64(3, 1.23, false);
    assert.sameValue(view.getFloat64(3, false), 1.23);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat64(7, 1.23, false);
    assert.sameValue(view.getFloat64(7, false), 1.23);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat64(10, 1.23, false);
    assert.sameValue(view.getFloat64(10, false), 1.23);

    // testFloatSet expected=-6213576.4839
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat64(0, -6213576.4839, true);
    assert.sameValue(view.getFloat64(0, true), -6213576.4839);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat64(3, -6213576.4839, true);
    assert.sameValue(view.getFloat64(3, true), -6213576.4839);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat64(7, -6213576.4839, true);
    assert.sameValue(view.getFloat64(7, true), -6213576.4839);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat64(10, -6213576.4839, true);
    assert.sameValue(view.getFloat64(10, true), -6213576.4839);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat64(0, -6213576.4839, false);
    assert.sameValue(view.getFloat64(0, false), -6213576.4839);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat64(3, -6213576.4839, false);
    assert.sameValue(view.getFloat64(3, false), -6213576.4839);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat64(7, -6213576.4839, false);
    assert.sameValue(view.getFloat64(7, false), -6213576.4839);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat64(10, -6213576.4839, false);
    assert.sameValue(view.getFloat64(10, false), -6213576.4839);

    // testFloatSet expected=NaN
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat64(0, NaN, true);
    assert.sameValue(view.getFloat64(0, true), NaN);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat64(3, NaN, true);
    assert.sameValue(view.getFloat64(3, true), NaN);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat64(7, NaN, true);
    assert.sameValue(view.getFloat64(7, true), NaN);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat64(10, NaN, true);
    assert.sameValue(view.getFloat64(10, true), NaN);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat64(0, NaN, false);
    assert.sameValue(view.getFloat64(0, false), NaN);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat64(3, NaN, false);
    assert.sameValue(view.getFloat64(3, false), NaN);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat64(7, NaN, false);
    assert.sameValue(view.getFloat64(7, false), NaN);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat64(10, NaN, false);
    assert.sameValue(view.getFloat64(10, false), NaN);

    // testFloatSet expected=-NaN
    //   Little endian
    view = new DataView(buffer13, 7, 9);
    view.setFloat64(0, -NaN, true);
    assert.sameValue(view.getFloat64(0, true), -NaN);
    view = new DataView(buffer13_pad3, 7, 12);
    view.setFloat64(3, -NaN, true);
    assert.sameValue(view.getFloat64(3, true), -NaN);
    view = new DataView(buffer13_pad7, 7, 16);
    view.setFloat64(7, -NaN, true);
    assert.sameValue(view.getFloat64(7, true), -NaN);
    view = new DataView(buffer13_pad10, 7, 19);
    view.setFloat64(10, -NaN, true);
    assert.sameValue(view.getFloat64(10, true), -NaN);
    //   Big endian.
    view = new DataView(buffer13_r, 7, 9);
    view.setFloat64(0, -NaN, false);
    assert.sameValue(view.getFloat64(0, false), -NaN);
    view = new DataView(buffer13_r_pad3, 7, 12);
    view.setFloat64(3, -NaN, false);
    assert.sameValue(view.getFloat64(3, false), -NaN);
    view = new DataView(buffer13_r_pad7, 7, 16);
    view.setFloat64(7, -NaN, false);
    assert.sameValue(view.getFloat64(7, false), -NaN);
    view = new DataView(buffer13_r_pad10, 7, 19);
    view.setFloat64(10, -NaN, false);
    assert.sameValue(view.getFloat64(10, false), -NaN);

    // Test for set methods that write to negative index

    // testSetNegativeIndexes
    view = new DataView(buffer1, 0, 16);
    checkThrow(() => view.setInt8(-1, 0), RangeError);
    checkThrow(() => view.setInt8(-2, 0), RangeError);
    checkThrow(() => view.setUint8(-1, 0), RangeError);
    checkThrow(() => view.setUint8(-2, 0), RangeError);
    checkThrow(() => view.setInt16(-1, 0), RangeError);
    checkThrow(() => view.setInt16(-2, 0), RangeError);
    checkThrow(() => view.setInt16(-3, 0), RangeError);
    checkThrow(() => view.setUint16(-1, 0), RangeError);
    checkThrow(() => view.setUint16(-2, 0), RangeError);
    checkThrow(() => view.setUint16(-3, 0), RangeError);
    checkThrow(() => view.setInt32(-1, 0), RangeError);
    checkThrow(() => view.setInt32(-3, 0), RangeError);
    checkThrow(() => view.setInt32(-5, 0), RangeError);
    checkThrow(() => view.setUint32(-1, 0), RangeError);
    checkThrow(() => view.setUint32(-3, 0), RangeError);
    checkThrow(() => view.setUint32(-5, 0), RangeError);
    view = new DataView(buffer7, 0, 8);
    checkThrow(() => view.setFloat32(-1, 0), RangeError);
    checkThrow(() => view.setFloat32(-3, 0), RangeError);
    checkThrow(() => view.setFloat32(-5, 0), RangeError);
    checkThrow(() => view.setFloat64(-1, 0), RangeError);
    checkThrow(() => view.setFloat64(-5, 0), RangeError);
    checkThrow(() => view.setFloat64(-9, 0), RangeError);

    // Too large for signed 32 bit integer index
    checkThrow(() => view.setInt8(2147483649, 1), RangeError);
    checkThrow(() => view.setUint8(2147483649, 1), RangeError);
    checkThrow(() => view.setInt16(2147483649, 1), RangeError);
    checkThrow(() => view.setUint16(2147483649, 1), RangeError);
    checkThrow(() => view.setInt32(2147483649, 1), RangeError);
    checkThrow(() => view.setUint32(2147483649, 1), RangeError);
    checkThrow(() => view.setFloat32(2147483649, 1), RangeError);
    checkThrow(() => view.setFloat64(2147483649, 1), RangeError);

    // testAlignment
    var intArray1 = [0, 1, 2, 3, 100, 101, 102, 103, 128, 129, 130, 131, 252, 253, 254, 255];
    view = new DataView(bufferize(new Uint8Array(intArray1)), 0);
    assert.sameValue(view.getUint32(0, false), 0x00010203);
    view = new DataView(bufferize(new Uint8Array(intArray1)), 1);
    assert.sameValue(view.getUint32(0, false), 0x01020364);
    view = new DataView(bufferize(new Uint8Array(intArray1)), 2);
    assert.sameValue(view.getUint32(0, false), 0x02036465);
    view = new DataView(bufferize(new Uint8Array(intArray1)), 3);
    assert.sameValue(view.getUint32(0, false), 0x03646566);
    view = new DataView(bufferize(new Uint8Array(intArray1)), 4);
    assert.sameValue(view.getUint32(0, false), 0x64656667);
    view = new DataView(bufferize(new Uint8Array(intArray1)), 5);
    assert.sameValue(view.getUint32(0, false), 0x65666780);
    view = new DataView(bufferize(new Uint8Array(intArray1)), 6);
    assert.sameValue(view.getUint32(0, false), 0x66678081);
    view = new DataView(bufferize(new Uint8Array(intArray1)), 7);
    assert.sameValue(view.getUint32(0, false), 0x67808182);

    // Test for indexing into a DataView, which should not use the ArrayBuffer storage
    view = new DataView(bufferize(new Uint8Array([1, 2])));
    assert.sameValue(view[0], undefined);
    view[0] = 3;
    assert.sameValue(view[0], 3);
    assert.sameValue(view.getUint8(0), 1);

    // Test WebIDL-specific class and prototype class names
    assert.sameValue(Object.prototype.toString.apply(new Uint8Array(0)), "[object Uint8Array]");
    assert.sameValue(Object.prototype.toString.apply(new Float32Array(0)), "[object Float32Array]");
    assert.sameValue(Object.prototype.toString.apply(new ArrayBuffer()), "[object ArrayBuffer]");
    assert.sameValue(Object.prototype.toString.apply(new DataView(view.buffer)), "[object DataView]");
    assert.sameValue(Object.prototype.toString.apply(DataView.prototype), "[object DataView]");

    // get %TypedArray%.prototype[@@toStringTag] returns undefined thus
    // Object.prototype.toString falls back to [object Object]
    assert.sameValue(Object.prototype.toString.apply(Uint8Array.prototype), "[object Object]");
    assert.sameValue(Object.prototype.toString.apply(Float32Array.prototype), "[object Object]");
    var typedArrayPrototype = Object.getPrototypeOf(Float32Array.prototype);
    assert.sameValue(Object.prototype.toString.apply(typedArrayPrototype), "[object Object]");

    // Accessing DataView fields on DataView.prototype should crash
    checkThrow(() => DataView.prototype.byteLength, TypeError);
    checkThrow(() => DataView.prototype.byteOffset, TypeError);
    checkThrow(() => DataView.prototype.buffer, TypeError);

    // Protos and proxies, oh my!
    var alien = $262.createRealm().global;
    var alien_data = alien.eval('data = ' + JSON.stringify(data1));
    var alien_buffer = alien.eval(`buffer = new ${sharedMem ? 'Shared' : ''}ArrayBuffer(data.length)`);
    alien.eval('new Uint8Array(buffer).set(data)');
    var alien_view = alien.eval('view = new DataView(buffer, 0, 16)');

    // proto is view of buffer: should throw
    var o = Object.create(view1);
    checkThrow(() => o.getUint8(4), TypeError); // WebIDL 4.4.7: Operations
    checkThrow(() => o.buffer, TypeError); // WebIDL 4.4.6: Attributes, section 2
    checkThrow(() => o.byteOffset, TypeError);
    checkThrow(() => o.byteLength, TypeError);

    // proxy for view of buffer: should work
    assert.sameValue(alien_view.buffer.byteLength > 0, true);
    assert.sameValue(alien_view.getUint8(4), 100);

    // Bug 753996: when throwing an Error whose message contains the name of a
    // function, the JS engine calls js_ValueToFunction to produce the function
    // name to include in the message. js_ValueToFunction does not unwrap its
    // argument. So if the function is actually a wrapper, then
    // js_ValueToFunction will throw a TypeError ("X is not a function").
    // Confusingly, this TypeError uses the decompiler, which *will* unwrap the
    // object to get the wrapped function name out, so the final error will
    // look something like "SomeFunction() is not a function".)
    var weirdo = Object.create(alien.eval("new Date"));
    var e = null;
    try {
        weirdo.getTime();
    } catch (exc) {
        e = exc;
    }
    assert.notSameValue(e, null);

    // proto is proxy for view of buffer: should throw TypeError
    //
    // As of this writing, bug 753996 causes this to throw the *wrong*
    // TypeError, and in fact it throws a (thisglobal).TypeError instead of
    // alien.TypeError.
    var av = Object.create(alien_view);
    checkThrow(() => av.getUint8(4), alien.TypeError);
    checkThrow(() => av.buffer, alien.TypeError);

    // view of object whose proto is buffer. This should not work per dherman.
    // Note that DataView throws a TypeError while TypedArrays create a
    // zero-length view. Odd though it seems, this is correct: TypedArrays have
    // a constructor that takes a length argument; DataViews do not. So a
    // TypedArray will do ToUint32 and end up passing a zero as the
    // constructor's length argument.
    buffer = Object.create(buffer1);
    checkThrow(() => new DataView(buffer), TypeError);

    // view of proxy for buffer
    av = new DataView(alien_buffer);
    assert.sameValue(av.getUint8(4), 100);
    assert.sameValue(Object.getPrototypeOf(av), DataView.prototype);

    // Bug 760904: call another compartment's constructor with an ArrayBuffer
    // from this compartment.
    var alien_constructor = alien.DataView;
    var local_buffer = (new Int8Array(3)).buffer;
    var foreign_exchange_student = new alien_constructor(local_buffer);

    // gc bug 787775
    var ab = new ArrayBuffer(4);
    var dv = new DataView(ab);
    dv = 1;
    $262.gc();

    // Bug 1438569.
    dv = new DataView(new ArrayBuffer(20 * 1024 * 1024));
    dv.setInt8(dv.byteLength - 10, 99);
    assert.sameValue(dv.getInt8(dv.byteLength - 10), 99);

    // get/setFloat16
    dv = new DataView(new ArrayBuffer(4));
    if (DataView.prototype.getFloat16) {
        dv.setInt16(0, 18688);
        assert.sameValue(dv.getFloat16(0), 10);
        dv.setFloat16(1, 10);
        assert.sameValue(dv.getFloat16(1), 10);
        assert.sameValue(dv.getInt16(1), 18688);
    }

}

test(false);

if (this.SharedArrayBuffer)
    test(true);

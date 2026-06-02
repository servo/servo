/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../common/util/util.js';import { float16ToUint16, uint16ToFloat16 } from './conversion.js';
import { align } from './math.js';

/**
 * BinaryStream is a utility to efficiently encode and decode numbers to / from a Uint8Array.
 * BinaryStream uses a number of internal typed arrays to avoid small array allocations when reading
 * and writing.
 */
export default class BinaryStream {
  /**
   * Constructor
   * @param buffer the buffer to read from / write to. Array length must be a multiple of 8 bytes.
   */
  constructor(buffer) {
    this.offset = 0;
    this.view = new DataView(buffer);
  }

  /** buffer() returns the stream's buffer sliced to the 8-byte rounded read or write offset */
  buffer() {
    return new Uint8Array(this.view.buffer, 0, align(this.offset, 8));
  }

  /** writeBool() writes a boolean as 255 or 0 to the buffer at the next byte offset */
  writeBool(value) {
    this.view.setUint8(this.offset++, value ? 255 : 0);
  }

  /** readBool() reads a boolean from the buffer at the next byte offset */
  readBool() {
    const val = this.view.getUint8(this.offset++);
    assert(val === 0 || val === 255);
    return val !== 0;
  }

  /** writeU8() writes a uint8 to the buffer at the next byte offset */
  writeU8(value) {
    this.view.setUint8(this.offset++, value);
  }

  /** readU8() reads a uint8 from the buffer at the next byte offset */
  readU8() {
    return this.view.getUint8(this.offset++);
  }

  /** writeU16() writes a uint16 to the buffer at the next 16-bit aligned offset */
  writeU16(value) {
    this.view.setUint16(this.alignedOffset(2), value, /* littleEndian */true);
  }

  /** readU16() reads a uint16 from the buffer at the next 16-bit aligned offset */
  readU16() {
    return this.view.getUint16(this.alignedOffset(2), /* littleEndian */true);
  }

  /** writeU32() writes a uint32 to the buffer at the next 32-bit aligned offset */
  writeU32(value) {
    this.view.setUint32(this.alignedOffset(4), value, /* littleEndian */true);
  }

  /** readU32() reads a uint32 from the buffer at the next 32-bit aligned offset */
  readU32() {
    return this.view.getUint32(this.alignedOffset(4), /* littleEndian */true);
  }

  /** writeI8() writes a int8 to the buffer at the next byte offset */
  writeI8(value) {
    this.view.setInt8(this.offset++, value);
  }

  /** readI8() reads a int8 from the buffer at the next byte offset */
  readI8() {
    return this.view.getInt8(this.offset++);
  }

  /** writeI16() writes a int16 to the buffer at the next 16-bit aligned offset */
  writeI16(value) {
    this.view.setInt16(this.alignedOffset(2), value, /* littleEndian */true);
  }

  /** readI16() reads a int16 from the buffer at the next 16-bit aligned offset */
  readI16() {
    return this.view.getInt16(this.alignedOffset(2), /* littleEndian */true);
  }

  /** writeI64() writes a bitint to the buffer at the next 64-bit aligned offset */
  writeI64(value) {
    this.view.setBigInt64(this.alignedOffset(8), value, /* littleEndian */true);
  }

  /** readI64() reads a bigint from the buffer at the next 64-bit aligned offset */
  readI64() {
    return this.view.getBigInt64(this.alignedOffset(8), /* littleEndian */true);
  }

  /** writeI32() writes a int32 to the buffer at the next 32-bit aligned offset */
  writeI32(value) {
    this.view.setInt32(this.alignedOffset(4), value, /* littleEndian */true);
  }

  /** readI32() reads a int32 from the buffer at the next 32-bit aligned offset */
  readI32() {
    return this.view.getInt32(this.alignedOffset(4), /* littleEndian */true);
  }

  /** writeF16() writes a float16 to the buffer at the next 16-bit aligned offset */
  writeF16(value) {
    this.writeU16(float16ToUint16(value));
  }

  /** readF16() reads a float16 from the buffer at the next 16-bit aligned offset */
  readF16() {
    return uint16ToFloat16(this.readU16());
  }

  /** writeF32() writes a float32 to the buffer at the next 32-bit aligned offset */
  writeF32(value) {
    this.view.setFloat32(this.alignedOffset(4), value, /* littleEndian */true);
  }

  /** readF32() reads a float32 from the buffer at the next 32-bit aligned offset */
  readF32() {
    return this.view.getFloat32(this.alignedOffset(4), /* littleEndian */true);
  }

  /** writeF64() writes a float64 to the buffer at the next 64-bit aligned offset */
  writeF64(value) {
    this.view.setFloat64(this.alignedOffset(8), value, /* littleEndian */true);
  }

  /** readF64() reads a float64 from the buffer at the next 64-bit aligned offset */
  readF64() {
    return this.view.getFloat64(this.alignedOffset(8), /* littleEndian */true);
  }

  /**
   * writeString() writes a length-prefixed UTF-16 string to the buffer at the next 32-bit aligned
   * offset
   */
  writeString(value) {
    this.writeU32(value.length);
    for (let i = 0; i < value.length; i++) {
      this.writeU16(value.charCodeAt(i));
    }
  }

  /**
   * readString() writes a length-prefixed UTF-16 string from the buffer at the next 32-bit aligned
   * offset
   */
  readString() {
    const len = this.readU32();
    const codes = new Array(len);
    for (let i = 0; i < len; i++) {
      codes[i] = this.readU16();
    }
    return String.fromCharCode(...codes);
  }

  /**
   * writeArray() writes a length-prefixed array of T elements to the buffer at the next 32-bit
   * aligned offset, using the provided callback to write the individual elements
   */
  writeArray(value, writeElement) {
    this.writeU32(value.length);
    for (const element of value) {
      writeElement(this, element);
    }
  }

  /**
   * readArray() reads a length-prefixed array of T elements from the buffer at the next 32-bit
   * aligned offset, using the provided callback to read the individual elements
   */
  readArray(readElement) {
    const len = this.readU32();
    const array = new Array(len);
    for (let i = 0; i < len; i++) {
      array[i] = readElement(this);
    }
    return array;
  }

  /**
   * writeCond() writes the boolean condition `cond` to the buffer, then either calls if_true if
   * `cond` is true, otherwise if_false
   */
  writeCond(cond, fns) {
    this.writeBool(cond);
    if (cond) {
      return fns.if_true();
    } else {
      return fns.if_false();
    }
  }

  /**
   * readCond() reads a boolean condition from the buffer, then either calls if_true if
   * the condition was is true, otherwise if_false
   */
  readCond(fns) {
    if (this.readBool()) {
      return fns.if_true();
    } else {
      return fns.if_false();
    }
  }

  /**
   * alignedOffset() aligns this.offset to `bytes`, then increments this.offset by `bytes`.
   * @returns the old offset aligned to the next multiple of `bytes`.
   */
  alignedOffset(bytes) {
    const aligned = align(this.offset, bytes);
    this.offset = aligned + bytes;
    return aligned;
  }



}
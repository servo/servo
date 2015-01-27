/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The Servo Binary Serialization Format: an extremely simple serialization format that is
//! optimized for speed above all else.

use serialize::{Decoder, Encoder};
use std::char;
use std::io::{IoError, Reader, Writer};

pub struct ServoEncoder<'a> {
    pub writer: &'a mut (Writer + 'a),
}

impl<'a> Encoder<IoError> for ServoEncoder<'a> {
    #[inline]
    fn emit_nil(&mut self) -> Result<(),IoError> {
        Ok(())
    }
    #[inline]
    fn emit_uint(&mut self, value: uint) -> Result<(),IoError> {
        self.writer.write_le_uint(value)
    }
    #[inline]
    fn emit_u64(&mut self, value: u64) -> Result<(),IoError> {
        self.writer.write_le_u64(value)
    }
    #[inline]
    fn emit_u32(&mut self, value: u32) -> Result<(),IoError> {
        self.writer.write_le_u32(value)
    }
    #[inline]
    fn emit_u16(&mut self, value: u16) -> Result<(),IoError> {
        self.writer.write_le_u16(value)
    }
    #[inline]
    fn emit_u8(&mut self, value: u8) -> Result<(),IoError> {
        self.writer.write_u8(value)
    }
    #[inline]
    fn emit_int(&mut self, value: int) -> Result<(),IoError> {
        self.writer.write_le_int(value)
    }
    #[inline]
    fn emit_i64(&mut self, value: i64) -> Result<(),IoError> {
        self.writer.write_le_i64(value)
    }
    #[inline]
    fn emit_i32(&mut self, value: i32) -> Result<(),IoError> {
        self.writer.write_le_i32(value)
    }
    #[inline]
    fn emit_i16(&mut self, value: i16) -> Result<(),IoError> {
        self.writer.write_le_i16(value)
    }
    #[inline]
    fn emit_i8(&mut self, value: i8) -> Result<(),IoError> {
        self.writer.write_i8(value)
    }
    #[inline]
    fn emit_bool(&mut self, value: bool) -> Result<(),IoError> {
        self.writer.write_u8(value as u8)
    }
    #[inline]
    fn emit_f64(&mut self, value: f64) -> Result<(),IoError> {
        self.writer.write_le_f64(value)
    }
    #[inline]
    fn emit_f32(&mut self, value: f32) -> Result<(),IoError> {
        self.writer.write_le_f32(value)
    }
    #[inline]
    fn emit_char(&mut self, value: char) -> Result<(),IoError> {
        self.writer.write_le_u32(value as u32)
    }
    #[inline]
    fn emit_str(&mut self, value: &str) -> Result<(),IoError> {
        try!(self.writer.write_le_uint(value.len()));
        self.writer.write_str(value)
    }
    #[inline]
    fn emit_enum(&mut self, _: &str, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                 -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_enum_variant(&mut self,
                         _: &str,
                         variant_id: uint,
                         _: uint,
                         f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                         -> Result<(),IoError> {
        try!(self.writer.write_le_u16(variant_id as u16));
        f(self)
    }
    #[inline]
    fn emit_enum_variant_arg(&mut self, _: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                             -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_enum_struct_variant(&mut self,
                                _: &str,
                                variant_id: uint,
                                _: uint,
                                f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                                -> Result<(),IoError> {
        try!(self.writer.write_le_u16(variant_id as u16));
        f(self)
    }
    #[inline]
    fn emit_enum_struct_variant_field(&mut self,
                                      _: &str,
                                      _: uint,
                                      f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                                      -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_struct(&mut self, _: &str, _: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                   -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_struct_field(&mut self,
                         _: &str,
                         _: uint,
                         f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                         -> Result<(),IoError>  {
        f(self)
    }
    #[inline]
    fn emit_tuple(&mut self, _: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                  -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_tuple_arg(&mut self, _: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                      -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_tuple_struct(&mut self,
                         _: &str,
                         _: uint,
                         f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                         -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_tuple_struct_arg(&mut self, _: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                             -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_option(&mut self, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                   -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_option_none(&mut self) -> Result<(),IoError> {
        self.writer.write_u8(0)
    }
    #[inline]
    fn emit_option_some(&mut self, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                        -> Result<(),IoError> {
        try!(self.writer.write_u8(1));
        f(self)
    }
    #[inline]
    fn emit_seq(&mut self, len: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                -> Result<(),IoError> {
        try!(self.writer.write_le_uint(len));
        f(self)
    }
    #[inline]
    fn emit_seq_elt(&mut self, _: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                    -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_map(&mut self, len: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                -> Result<(),IoError> {
        try!(self.writer.write_le_uint(len));
        f(self)
    }
    #[inline]
    fn emit_map_elt_key(&mut self, _: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                        -> Result<(),IoError> {
        f(self)
    }
    #[inline]
    fn emit_map_elt_val(&mut self, _: uint, f: |&mut ServoEncoder<'a>| -> Result<(),IoError>)
                        -> Result<(),IoError> {
        f(self)
    }
}

pub struct ServoDecoder<'a> {
    pub reader: &'a mut (Reader + 'a),
}

impl<'a> Decoder<IoError> for ServoDecoder<'a> {
    #[inline]
    fn read_nil(&mut self) -> Result<(),IoError> {
        Ok(())
    }
    #[inline]
    fn read_uint(&mut self) -> Result<uint,IoError> {
        self.reader.read_le_uint()
    }
    #[inline]
    fn read_u64(&mut self) -> Result<u64,IoError> {
        self.reader.read_le_u64()
    }
    #[inline]
    fn read_u32(&mut self) -> Result<u32,IoError> {
        self.reader.read_le_u32()
    }
    #[inline]
    fn read_u16(&mut self) -> Result<u16,IoError> {
        self.reader.read_le_u16()
    }
    #[inline]
    fn read_u8(&mut self) -> Result<u8,IoError> {
        self.reader.read_u8()
    }
    #[inline]
    fn read_int(&mut self) -> Result<int,IoError> {
        self.reader.read_le_int()
    }
    #[inline]
    fn read_i64(&mut self) -> Result<i64,IoError> {
        self.reader.read_le_i64()
    }
    #[inline]
    fn read_i32(&mut self) -> Result<i32,IoError> {
        self.reader.read_le_i32()
    }
    #[inline]
    fn read_i16(&mut self) -> Result<i16,IoError> {
        self.reader.read_le_i16()
    }
    #[inline]
    fn read_i8(&mut self) -> Result<i8,IoError> {
        self.reader.read_i8()
    }
    #[inline]
    fn read_bool(&mut self) -> Result<bool,IoError> {
        Ok(try!(self.reader.read_u8()) != 0)
    }
    #[inline]
    fn read_f64(&mut self) -> Result<f64,IoError> {
        self.reader.read_le_f64()
    }
    #[inline]
    fn read_f32(&mut self) -> Result<f32,IoError> {
        self.reader.read_le_f32()
    }
    #[inline]
    fn read_char(&mut self) -> Result<char,IoError> {
        Ok(char::from_u32(try!(self.reader.read_le_u32())).unwrap())
    }
    #[inline]
    fn read_str(&mut self) -> Result<String,IoError> {
        let len = try!(self.reader.read_le_uint());
        let bytes = try!(self.reader.read_exact(len));
        Ok(String::from_utf8(bytes).unwrap())
    }
    #[inline]
    fn read_enum<T>(&mut self, _: &str, f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                    -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_enum_variant<T>(&mut self,
                            _: &[&str],
                            f: |&mut ServoDecoder<'a>, uint| -> Result<T,IoError>)
                            -> Result<T,IoError> {
        let index = try!(self.reader.read_le_u16());
        f(self, index as uint)
    }
    #[inline]
    fn read_enum_variant_arg<T>(&mut self,
                                _: uint,
                                f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                                -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_enum_struct_variant<T>(&mut self,
                                   _: &[&str],
                                   f: |&mut ServoDecoder<'a>, uint| -> Result<T,IoError>)
                                   -> Result<T,IoError> {
        let index = try!(self.reader.read_le_u16());
        f(self, index as uint)
    }
    #[inline]
    fn read_enum_struct_variant_field<T>(&mut self,
                                         _: &str,
                                         _: uint,
                                         f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                                         -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_struct<T>(&mut self,
                      _: &str,
                      _: uint,
                      f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                      -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_struct_field<T>(&mut self,
                            _: &str,
                            _: uint,
                            f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                            -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_tuple<T>(&mut self, _: uint, f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                     -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_tuple_arg<T>(&mut self, _: uint, f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                         -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_tuple_struct<T>(&mut self,
                            _: &str,
                            _: uint,
                            f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                            -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_tuple_struct_arg<T>(&mut self,
                                _: uint,
                                f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                                -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_option<T>(&mut self, f: |&mut ServoDecoder<'a>, bool| -> Result<T,IoError>)
                      -> Result<T,IoError> {
        let is_some = try!(self.reader.read_u8()) != 0;
        f(self, is_some)
    }
    #[inline]
    fn read_seq<T>(&mut self, f: |&mut ServoDecoder<'a>, uint| -> Result<T,IoError>)
                   -> Result<T,IoError> {
        let len = try!(self.reader.read_le_uint());
        f(self, len)
    }
    #[inline]
    fn read_seq_elt<T>(&mut self, _: uint, f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                       -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_map<T>(&mut self, f: |&mut ServoDecoder<'a>, uint| -> Result<T,IoError>)
                   -> Result<T,IoError> {
        let len = try!(self.reader.read_le_uint());
        f(self, len)
    }
    #[inline]
    fn read_map_elt_key<T>(&mut self, _: uint, f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                           -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn read_map_elt_val<T>(&mut self, _: uint, f: |&mut ServoDecoder<'a>| -> Result<T,IoError>)
                           -> Result<T,IoError> {
        f(self)
    }
    #[inline]
    fn error(&mut self, _: &str) -> IoError {
        IoError::from_errno(0, false)
    }
}


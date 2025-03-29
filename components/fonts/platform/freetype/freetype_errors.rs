/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use freetype_sys::{
    FT_Err_Array_Too_Large, FT_Err_Bad_Argument, FT_Err_Bbx_Too_Big, FT_Err_CMap_Table_Missing,
    FT_Err_Cannot_Open_Resource, FT_Err_Cannot_Open_Stream, FT_Err_Cannot_Render_Glyph,
    FT_Err_Code_Overflow, FT_Err_Corrupted_Font_Glyphs, FT_Err_Corrupted_Font_Header,
    FT_Err_Could_Not_Find_Context, FT_Err_Debug_OpCode, FT_Err_Divide_By_Zero,
    FT_Err_ENDF_In_Exec_Stream, FT_Err_Execution_Too_Long, FT_Err_Hmtx_Table_Missing,
    FT_Err_Horiz_Header_Missing, FT_Err_Ignore, FT_Err_Invalid_Argument,
    FT_Err_Invalid_Cache_Handle, FT_Err_Invalid_CharMap_Format, FT_Err_Invalid_CharMap_Handle,
    FT_Err_Invalid_Character_Code, FT_Err_Invalid_CodeRange, FT_Err_Invalid_Composite,
    FT_Err_Invalid_Driver_Handle, FT_Err_Invalid_Face_Handle, FT_Err_Invalid_File_Format,
    FT_Err_Invalid_Frame_Operation, FT_Err_Invalid_Frame_Read, FT_Err_Invalid_Glyph_Format,
    FT_Err_Invalid_Glyph_Index, FT_Err_Invalid_Handle, FT_Err_Invalid_Horiz_Metrics,
    FT_Err_Invalid_Library_Handle, FT_Err_Invalid_Offset, FT_Err_Invalid_Opcode,
    FT_Err_Invalid_Outline, FT_Err_Invalid_PPem, FT_Err_Invalid_Pixel_Size,
    FT_Err_Invalid_Post_Table, FT_Err_Invalid_Post_Table_Format, FT_Err_Invalid_Reference,
    FT_Err_Invalid_Size_Handle, FT_Err_Invalid_Slot_Handle, FT_Err_Invalid_Stream_Handle,
    FT_Err_Invalid_Stream_Operation, FT_Err_Invalid_Stream_Read, FT_Err_Invalid_Stream_Seek,
    FT_Err_Invalid_Stream_Skip, FT_Err_Invalid_Table, FT_Err_Invalid_Version,
    FT_Err_Invalid_Vert_Metrics, FT_Err_Locations_Missing, FT_Err_Lower_Module_Version,
    FT_Err_Missing_Bbx_Field, FT_Err_Missing_Chars_Field, FT_Err_Missing_Encoding_Field,
    FT_Err_Missing_Font_Field, FT_Err_Missing_Fontboundingbox_Field, FT_Err_Missing_Module,
    FT_Err_Missing_Property, FT_Err_Missing_Size_Field, FT_Err_Missing_Startchar_Field,
    FT_Err_Missing_Startfont_Field, FT_Err_Name_Table_Missing, FT_Err_Nested_DEFS,
    FT_Err_Nested_Frame_Access, FT_Err_No_Unicode_Glyph_Name, FT_Err_Ok, FT_Err_Out_Of_Memory,
    FT_Err_Post_Table_Missing, FT_Err_Raster_Corrupted, FT_Err_Raster_Negative_Height,
    FT_Err_Raster_Overflow, FT_Err_Raster_Uninitialized, FT_Err_Stack_Overflow,
    FT_Err_Stack_Underflow, FT_Err_Syntax_Error, FT_Err_Table_Missing, FT_Err_Too_Few_Arguments,
    FT_Err_Too_Many_Caches, FT_Err_Too_Many_Drivers, FT_Err_Too_Many_Extensions,
    FT_Err_Too_Many_Function_Defs, FT_Err_Too_Many_Hints, FT_Err_Too_Many_Instruction_Defs,
    FT_Err_Unimplemented_Feature, FT_Err_Unknown_File_Format, FT_Err_Unlisted_Object, FT_Error,
};

// Trait bellow generated from:
// https://freetype.org/freetype2/docs/reference/ft2-error_code_values.html
pub trait CustomFtErrorMethods {
    fn ft_get_error_message(&self) -> &'static str;
}

impl CustomFtErrorMethods for FT_Error {
    #[allow(non_upper_case_globals)]
    fn ft_get_error_message(&self) -> &'static str {
        match *self {
            /* generic errors */
            FT_Err_Ok => "no error",
            FT_Err_Cannot_Open_Resource => "cannot open resource",
            FT_Err_Unknown_File_Format => "unknown file format",
            FT_Err_Invalid_File_Format => "broken file",
            FT_Err_Invalid_Version => "invalid FreeType version",
            FT_Err_Lower_Module_Version => "module version is too low",
            FT_Err_Invalid_Argument => "invalid argument",
            FT_Err_Unimplemented_Feature => "unimplemented feature",
            FT_Err_Invalid_Table => "broken table",
            FT_Err_Invalid_Offset => "broken offset within table",
            FT_Err_Array_Too_Large => "array allocation size too large",
            FT_Err_Missing_Module => "missing module",
            FT_Err_Missing_Property => "missing property",

            /* glyph/character errors */
            FT_Err_Invalid_Glyph_Index => "invalid glyph index",
            FT_Err_Invalid_Character_Code => "invalid character code",
            FT_Err_Invalid_Glyph_Format => "unsupported glyph image format",
            FT_Err_Cannot_Render_Glyph => "cannot render this glyph format",
            FT_Err_Invalid_Outline => "invalid outline",
            FT_Err_Invalid_Composite => "invalid composite glyph",
            FT_Err_Too_Many_Hints => "too many hints",
            FT_Err_Invalid_Pixel_Size => "invalid pixel size",
            // FT_Err_Invalid_SVG_Document => "invalid SVG document",

            /* handle errors */
            FT_Err_Invalid_Handle => "invalid object handle",
            FT_Err_Invalid_Library_Handle => "invalid library handle",
            FT_Err_Invalid_Driver_Handle => "invalid module handle",
            FT_Err_Invalid_Face_Handle => "invalid face handle",
            FT_Err_Invalid_Size_Handle => "invalid size handle",
            FT_Err_Invalid_Slot_Handle => "invalid glyph slot handle",
            FT_Err_Invalid_CharMap_Handle => "invalid charmap handle",
            FT_Err_Invalid_Cache_Handle => "invalid cache manager handle",
            FT_Err_Invalid_Stream_Handle => "invalid stream handle",

            /* driver errors */
            FT_Err_Too_Many_Drivers => "too many modules",
            FT_Err_Too_Many_Extensions => "too many extensions",

            /* memory errors */
            FT_Err_Out_Of_Memory => "out of memory",
            FT_Err_Unlisted_Object => "unlisted object",

            /* stream errors */
            FT_Err_Cannot_Open_Stream => "cannot open stream",
            FT_Err_Invalid_Stream_Seek => "invalid stream seek",
            FT_Err_Invalid_Stream_Skip => "invalid stream skip",
            FT_Err_Invalid_Stream_Read => "invalid stream read",
            FT_Err_Invalid_Stream_Operation => "invalid stream operation",
            FT_Err_Invalid_Frame_Operation => "invalid frame operation",
            FT_Err_Nested_Frame_Access => "nested frame access",
            FT_Err_Invalid_Frame_Read => "invalid frame read",

            /* raster errors */
            FT_Err_Raster_Uninitialized => "raster uninitialized",
            FT_Err_Raster_Corrupted => "raster corrupted",
            FT_Err_Raster_Overflow => "raster overflow",
            FT_Err_Raster_Negative_Height => "negative height while rastering",

            /* cache errors */
            FT_Err_Too_Many_Caches => "too many registered caches",

            /* TrueType and SFNT errors */
            FT_Err_Invalid_Opcode => "invalid opcode",
            FT_Err_Too_Few_Arguments => "too few arguments",
            FT_Err_Stack_Overflow => "stack overflow",
            FT_Err_Code_Overflow => "code overflow",
            FT_Err_Bad_Argument => "bad argument",
            FT_Err_Divide_By_Zero => "division by zero",
            FT_Err_Invalid_Reference => "invalid reference",
            FT_Err_Debug_OpCode => "found debug opcode",
            FT_Err_ENDF_In_Exec_Stream => "found ENDF opcode in execution stream",
            FT_Err_Nested_DEFS => "nested DEFS",
            FT_Err_Invalid_CodeRange => "invalid code range",
            FT_Err_Execution_Too_Long => "execution context too long",
            FT_Err_Too_Many_Function_Defs => "too many function definitions",
            FT_Err_Too_Many_Instruction_Defs => "too many instruction definitions",
            FT_Err_Table_Missing => "SFNT font table missing",
            FT_Err_Horiz_Header_Missing => "horizontal header (hhea) table missing",
            FT_Err_Locations_Missing => "locations (loca) table missing",
            FT_Err_Name_Table_Missing => "name table missing",
            FT_Err_CMap_Table_Missing => "character map (cmap) table missing",
            FT_Err_Hmtx_Table_Missing => "horizontal metrics (hmtx) table missing",
            FT_Err_Post_Table_Missing => "PostScript (post) table missing",
            FT_Err_Invalid_Horiz_Metrics => "invalid horizontal metrics",
            FT_Err_Invalid_CharMap_Format => "invalid character map (cmap) format",
            FT_Err_Invalid_PPem => "invalid ppem value",
            FT_Err_Invalid_Vert_Metrics => "invalid vertical metrics",
            FT_Err_Could_Not_Find_Context => "could not find context",
            FT_Err_Invalid_Post_Table_Format => "invalid PostScript (post) table format",
            FT_Err_Invalid_Post_Table => "invalid PostScript (post) table",
            // FT_Err_DEF_In_Glyf_Bytecode => "found FDEF or IDEF opcode in glyf bytecode",
            // FT_Err_Missing_Bitmap => "missing bitmap in strike",
            // FT_Err_Missing_SVG_Hooks => "SVG hooks have not been set",

            /* CFF, CID, and Type 1 errors */
            FT_Err_Syntax_Error => "opcode syntax error",
            FT_Err_Stack_Underflow => "argument stack underflow",
            FT_Err_Ignore => "ignore",
            FT_Err_No_Unicode_Glyph_Name => "no Unicode glyph name found",
            // FT_Err_Glyph_Too_Big => "glyph too big for hinting",

            /* BDF errors */
            FT_Err_Missing_Startfont_Field => "`STARTFONT' field missing",
            FT_Err_Missing_Font_Field => "`FONT` field missing",
            FT_Err_Missing_Size_Field => "`SIZE` field missing",
            FT_Err_Missing_Fontboundingbox_Field => "`FONTBOUNDINGBOX' field missing",
            FT_Err_Missing_Chars_Field => "`CHARS` field missing",
            FT_Err_Missing_Startchar_Field => "`STARTCHAR` field missing",
            FT_Err_Missing_Encoding_Field => "`ENCODING` field missing",
            FT_Err_Missing_Bbx_Field => "`BBX` field missing",
            FT_Err_Bbx_Too_Big => "`BBX` too big",
            FT_Err_Corrupted_Font_Header => "Font header corrupted or missing fields",
            FT_Err_Corrupted_Font_Glyphs => "Font glyphs corrupted or missing fields",

            _ => "unknown error",
        }
    }
}

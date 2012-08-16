import gfx::surface;
import io::WriterUtil;

fn encode(writer: io::Writer, surface: surface::image_surface) {
    assert surface.format == gfx::surface::fo_rgba_8888;

    writer.write_u8(0u8);                               // identsize
    writer.write_u8(0u8);                               // colourmaptype
    writer.write_u8(2u8);                               // imagetype

    writer.write_le_u16(0u16);                          // colourmapstart
    writer.write_le_u16(0u16);                          // colourmaplength
    writer.write_u8(16u8);                              // colourmapbits

    writer.write_le_u16(0u16);                          // xstart
    writer.write_le_u16(0u16);                          // ystart
    writer.write_le_u16(surface.size.width as u16);     // width
    writer.write_le_u16(surface.size.height as u16);    // height
    writer.write_u8(32u8);                              // bits
    writer.write_u8(0x30u8);                            // descriptor

    writer.write(surface.buffer);
}


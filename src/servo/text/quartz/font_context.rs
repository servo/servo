pub struct QuartzFontContext {
    ctx: u8,

    drop { }
}

pub impl QuartzFontContext {
    // this is a placeholder until NSFontManager or whatever is bound in here.
    static pub fn new() -> QuartzFontContext {
        QuartzFontContext { ctx: 42 }
    }
}
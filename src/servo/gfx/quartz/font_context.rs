pub struct QuartzFontContextHandle {
    ctx: (),

    drop { }
}

pub impl QuartzFontContextHandle {
    // this is a placeholder until NSFontManager or whatever is bound in here.
    static pub fn new() -> QuartzFontContextHandle {
        QuartzFontContextHandle { ctx: () }
    }
}
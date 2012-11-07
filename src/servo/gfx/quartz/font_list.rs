pub struct QuartzFontListHandle {
    ctx: (),

    drop { }
}

pub impl QuartzFontListHandle {
    // this is a placeholder until CTFontCollection is bound here.
    static pub fn new() -> QuartzFontListHandle {
        QuartzFontListHandle { ctx: () }
    }
}
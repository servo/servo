
pub struct QuartzNativeFontMatcher {
    dummy: int,

    drop { }
}

pub impl QuartzNativeFontMatcher {
    // this is a placeholder until NSFontManager or whatever is bound in here.
    static pub fn new() -> QuartzNativeFontMatcher {
        QuartzNativeFontMatcher { dummy: 42 }
    }
}
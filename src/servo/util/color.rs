#[doc = "A library for handling colors and parsing css color declarations."]

// TODO: handle #rrggbb color declarations, handle rgb(r%,g%,b%),
// sanitize input / crop it to correct ranges, predefine other 130
// css-defined colors

import float::round;
import libc::types::os::arch::c95::c_double;
import css_colors::*;

enum Color = {red : u8, green : u8, blue : u8, alpha : float};

fn rgba(r : u8, g : u8, b : u8, a : float) -> Color {
    Color({red : r, green : g, blue : b, alpha : a})
}

fn rgb(r : u8, g : u8, b : u8) -> Color {
    ret rgba(r, g, b, 1.0);
}

fn hsla(h : float, s : float, l : float, a : float) -> Color {
    // Algorithm for converting hsl to rbg taken from
    // http://www.w3.org/TR/2003/CR-css3-color-20030514/#hsl-color
    let m2 = if l <= 0.5 { l*(s + 1.0) } else { l + s - l*s };
    let m1 = l*2.0 - m2;
    let h = h / 360.0; 
    
    fn hue_to_rgb(m1 : float, m2 : float, h : float) -> float {
        let h = if h < 0.0 { h + 1.0 } else if h > 1.0 { h - 1.0 } else { h };

        alt h {
          0.0 to 1.0/6.0     { ret m1 + (m2 - m1)*h*6.0; }
          1.0/6.0 to 1.0/2.0 { ret m2; }
          1.0/2.0 to 2.0/3.0 { ret m1 + (m2 - m1)*(4.0 - 6.0*h); }
          2.0/3.0 to 1.0     { ret m1; }
          _                  { fail "unexpected hue value"; }
        }
    }

    let r = round(255.0*hue_to_rgb(m1, m2, h + 1.0/3.0) as c_double);;
    let g = round(255.0*hue_to_rgb(m1, m2, h) as c_double);
    let b = round(255.0*hue_to_rgb(m1, m2, h - 1.0/3.0) as c_double);

    ret rgba(r as u8, g as u8, b as u8, a);
}

fn hsl(h : float, s : float, l : float) -> Color {
    ret hsla(h, s, l, 1.0);
}

impl methods for Color {
    fn print() -> str {
        #fmt["rgba(%u,%u,%u,%f)", self.red as uint, self.green as uint,
             self.blue as uint, self.alpha]
    }
}

mod parsing {
    export parse_color;

    // TODO, fail by ignoring the rule instead of setting the
    // color to black
    fn fail_unrecognized(col : str) -> Color {
        #warn["Unrecognized color %s", col];
        ret  black();
    }

    #[doc="Match an exact color keyword."]
    fn parse_by_name(color : str) -> Color {
        alt color.to_lower() {
          "black"   { black()   }
          "silver"  { silver()  }
          "gray"    { gray()    }
          "grey"    { gray()    }
          "white"   { white()   }
          "maroon"  { maroon()  }
          "red"     { red()     }
          "purple"  { purple()  }
          "fuschia" { fuschia() }
          "green"   { green()   }
          "lime"    { lime()    }
          "olive"   { olive()   }
          "yellow"  { yellow()  }
          "navy"    { navy()    }
          "blue"    { blue()    }
          "teal"    { teal()    }
          "aqua"    { aqua()    }
          _         { fail_unrecognized(color) }
        }
    }
    
    #[doc="Parses a color specification in the form rgb(foo,bar,baz)"]
    fn parse_rgb(color : str) -> Color {
        // Shave off the rgb( and the )
        let only_colors = color.substr(4u, color.len() - 5u);

        // split up r, g, and b
        let cols = only_colors.split_char(',');
        if cols.len() != 3u { ret fail_unrecognized(color); } 

        alt (u8::from_str(cols[0]), u8::from_str(cols[1]), 
             u8::from_str(cols[2])) {
          (some(r), some(g), some(b)) { rgb(r, g, b) }
          _               { fail_unrecognized(color) }
        }
    }

    #[doc="Parses a color specification in the form rgba(foo,bar,baz,qux)"]
    fn parse_rgba(color : str) -> Color {
        // Shave off the rgba( and the )
        let only_vals = color.substr(5u, color.len() - 6u);

        // split up r, g, and b
        let cols = only_vals.split_char(',');
        if cols.len() != 4u { ret fail_unrecognized(color); } 

        alt (u8::from_str(cols[0]), u8::from_str(cols[1]), 
             u8::from_str(cols[2]), float::from_str(cols[3])) {
          (some(r), some(g), some(b), some(a)) { rgba(r, g, b, a) }
          _ { fail_unrecognized(color) }
        }
    }

    #[doc="Parses a color specification in the form hsl(foo,bar,baz)"]
    fn parse_hsl(color : str) -> Color {
        // Shave off the hsl( and the )
        let only_vals = color.substr(4u, color.len() - 5u);

        // split up h, s, and l
        let vals = only_vals.split_char(',');
        if vals.len() != 3u { ret fail_unrecognized(color); } 

        alt (float::from_str(vals[0]), float::from_str(vals[1]), 
             float::from_str(vals[2])) {
          (some(h), some(s), some(l)) { hsl(h, s, l) }
          _               { fail_unrecognized(color) }
        }
    }

    #[doc="Parses a color specification in the form hsla(foo,bar,baz,qux)"]
    fn parse_hsla(color : str) -> Color {
        // Shave off the hsla( and the )
        let only_vals = color.substr(5u, color.len() - 6u);

        let vals = only_vals.split_char(',');
        if vals.len() != 4u { ret fail_unrecognized(color); } 

        alt (float::from_str(vals[0]), float::from_str(vals[1]), 
             float::from_str(vals[2]), float::from_str(vals[3])) {
          (some(h), some(s), some(l), some(a)) { hsla(h, s, l, a) }
          _ { fail_unrecognized(color) }
        }
    }

    // Currently colors are supported in rgb(a,b,c) form and also by
    // keywords for several common colors.
    // TODO: extend this
    fn parse_color(color : str) -> Color {
        alt color {
          c if c.starts_with("rgb(")  { parse_rgb(c) }
          c if c.starts_with("rgba(") { parse_rgba(c) }
          c if c.starts_with("hsl(")  { parse_hsl(c) }
          c if c.starts_with("hsla(") { parse_hsla(c) }
          c                           { parse_by_name(c) }
        }
    }
}

mod test {
    import css_colors::*;
    import parsing::parse_color;

    #[test]
    fn test_parse_by_name() {
        assert red() == parse_color("red");
        assert lime() == parse_color("Lime");
        assert blue() == parse_color("BLUE");
        assert green() == parse_color("GreEN");
        assert white() == parse_color("white");
        assert black() == parse_color("Black");
        assert gray() == parse_color("Gray");
        assert silver() == parse_color("SiLvEr");
        assert maroon() == parse_color("maroon");
        assert purple() == parse_color("PURPLE");
        assert fuschia() == parse_color("FUSCHIA");
        assert olive() == parse_color("oLiVe");
        assert yellow() == parse_color("yellow");
        assert navy() == parse_color("NAVY");
        assert teal() == parse_color("Teal");
        assert aqua() == parse_color("Aqua");
    }

    #[test]
    fn test_parsing_rgb() {
        assert red() == parse_color("rgb(255,0,0)");
        assert red() == parse_color("rgba(255,0,0,1.0)");
        assert red() == parse_color("rgba(255,0,0,1)");
        assert lime() == parse_color("rgba(0,255,0,1.00)");
        assert rgb(1u8,2u8,3u8) == parse_color("rgb(1,2,03)");
        assert rgba(15u8,250u8,3u8,0.5) == parse_color("rgba(15,250,3,.5)");
        assert rgba(15u8,250u8,3u8,0.5) == parse_color("rgba(15,250,3,0.5)");
    }

    #[test]
    fn test_parsing_hsl() {
        assert red() == parse_color("hsl(0,1,.5)");
        assert lime() == parse_color("hsl(120.0,1.0,.5)");
        assert blue() == parse_color("hsl(240.0,1.0,.5)");
        assert green() == parse_color("hsl(120.0,1.0,.25)");
        assert white() == parse_color("hsl(1.0,1.,1.0)");
        assert white() == parse_color("hsl(129.0,0.3,1.0)");
        assert black() == parse_color("hsl(231.2,0.75,0.0)");
        assert black() == parse_color("hsl(11.2,0.0,0.0)");
        assert gray() == parse_color("hsl(0.0,0.0,0.5)");
        assert maroon() == parse_color("hsl(0.0,1.0,0.25)");
        assert purple() == parse_color("hsl(300.0,1.0,0.25)");
        assert fuschia() == parse_color("hsl(300,1.0,0.5)");
        assert olive() == parse_color("hsl(60.,1.0,0.25)");
        assert yellow() == parse_color("hsl(60.,1.0,0.5)");
        assert navy() == parse_color("hsl(240.0,1.0,.25)");
        assert teal() == parse_color("hsl(180.0,1.0,.25)");
        assert aqua() == parse_color("hsl(180.0,1.0,.5)");
    }        
}


#[doc="Define the colors specified by css"]
mod css_colors {
    // The 16 basic css colors
    fn black() -> Color {
        Color({red : 0u8, green : 0u8, blue : 0u8, alpha : 1.0})
    }
    fn silver() -> Color {
        Color({red : 192u8, green : 192u8, blue : 192u8, alpha : 1.0})
    }
    fn gray() -> Color {
        Color({red : 128u8, green : 128u8, blue : 128u8, alpha : 1.0})
    }
    fn white() -> Color {
        Color({red : 255u8, green : 255u8, blue : 255u8, alpha : 1.0})
    }
    fn maroon() -> Color {
        Color({red : 128u8, green : 0u8, blue : 0u8, alpha : 1.0})
    }
    fn red() -> Color { 
        Color({red : 255u8, green : 0u8, blue : 0u8, alpha : 1.0})
    }
    fn purple() -> Color {
        Color({red : 128u8, green : 0u8, blue : 128u8, alpha : 1.0})
    }
    fn fuschia() -> Color {
        Color({red : 255u8, green : 0u8, blue : 255u8, alpha : 1.0})
    }
    fn green() -> Color { 
        Color({red : 0u8, green : 128u8, blue : 0u8, alpha : 1.0})
    }
    fn lime() -> Color {
        Color({red : 0u8, green : 255u8, blue : 0u8, alpha : 1.0})
    }
    fn olive() -> Color {
        Color({red : 128u8, green : 128u8, blue : 0u8, alpha : 1.0})
    }
    fn yellow() -> Color {
        Color({red : 255u8, green : 255u8, blue : 0u8, alpha : 1.0})
    }
    fn navy() -> Color {
        Color({red : 0u8, green : 0u8, blue : 128u8, alpha : 1.0})
    }
    fn blue() -> Color {
        Color({red : 0u8, green : 0u8, blue : 255u8, alpha : 1.0})
    }
    fn teal() -> Color {
        Color({red : 0u8, green : 128u8, blue : 128u8, alpha : 1.0})
    }
    fn aqua() -> Color {
        Color({red : 0u8, green : 255u8, blue : 255u8, alpha : 1.0})
    }


    // The other 130 css colors
    // TODO
}

#[doc = "A library for handling colors and parsing css color declarations."]

// TODO: handle #rrggbb color declarations, handle rgb(r%,g%,b%),
// sanitize input / crop it to correct ranges, predefine other 130
// css-defined colors

import float::round;
import libc::types::os::arch::c95::c_double;
import css_colors::*;
import cmp::eq;

enum Color = {red : u8, green : u8, blue : u8, alpha : float};

impl Color of eq for Color {
    pure fn eq(&&other: Color) -> bool {
        return self.red == other.red && self.green == other.green && self.blue == other.blue &&
               self.alpha == other.alpha;
    }
}

fn rgba(r : u8, g : u8, b : u8, a : float) -> Color {
    Color({red : r, green : g, blue : b, alpha : a})
}

fn rgb(r : u8, g : u8, b : u8) -> Color {
    return rgba(r, g, b, 1.0);
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
          0.0 to 1.0/6.0 => m1 + (m2 - m1)*h*6.0,
          1.0/6.0 to 1.0/2.0 => m2,
          1.0/2.0 to 2.0/3.0 => m1 + (m2 - m1)*(4.0 - 6.0*h),
          2.0/3.0 to 1.0 => return m1,
          _ => fail ~"unexpected hue value"
        }
    }

    let r = round(255.0*hue_to_rgb(m1, m2, h + 1.0/3.0) as c_double);;
    let g = round(255.0*hue_to_rgb(m1, m2, h) as c_double);
    let b = round(255.0*hue_to_rgb(m1, m2, h - 1.0/3.0) as c_double);

    return rgba(r as u8, g as u8, b as u8, a);
}

fn hsl(h : float, s : float, l : float) -> Color {
    return hsla(h, s, l, 1.0);
}

impl methods for Color {
    fn print() -> ~str {
        #fmt["rgba(%u,%u,%u,%f)", self.red as uint, self.green as uint,
             self.blue as uint, self.alpha]
    }
}

mod parsing {
    export parse_color;

    fn fail_unrecognized(col : ~str) -> option<Color> {
        #warn["Unrecognized color %s", col];
        return none;
    }

    #[doc="Match an exact color keyword."]
    fn parse_by_name(color : ~str) -> option<Color> {
        let col = alt color.to_lower() {
          ~"black" => black(),
          ~"silver" => silver(),
          ~"gray" => gray(),
          ~"grey" => gray(),
          ~"white" => white(),
          ~"maroon" => maroon(),
          ~"red" => red(),
          ~"purple" => purple(),
          ~"fuchsia" => fuchsia(),
          ~"green" => green(),
          ~"lime" => lime(),
          ~"olive" => olive(),
          ~"yellow" => yellow(),
          ~"navy" => navy(),
          ~"blue" => blue(),
          ~"teal" => teal(),
          ~"aqua" => aqua(),
          _ => return fail_unrecognized(color)
        };

        return some(col);
    }
    
    #[doc="Parses a color specification in the form rgb(foo,bar,baz)"]
    fn parse_rgb(color : ~str) -> option<Color> {
        // Shave off the rgb( and the )
        let only_colors = color.substr(4u, color.len() - 5u);

        // split up r, g, and b
        let cols = only_colors.split_char(',');
        if cols.len() != 3u { return fail_unrecognized(color); }

        alt (u8::from_str(cols[0]), u8::from_str(cols[1]), 
             u8::from_str(cols[2])) {
          (some(r), some(g), some(b)) => { some(rgb(r, g, b)) }
          _ => { fail_unrecognized(color) }
        }
    }

    #[doc="Parses a color specification in the form rgba(foo,bar,baz,qux)"]
    fn parse_rgba(color : ~str) -> option<Color> {
        // Shave off the rgba( and the )
        let only_vals = color.substr(5u, color.len() - 6u);

        // split up r, g, and b
        let cols = only_vals.split_char(',');
        if cols.len() != 4u { return fail_unrecognized(color); }

        alt (u8::from_str(cols[0]), u8::from_str(cols[1]), 
             u8::from_str(cols[2]), float::from_str(cols[3])) {
          (some(r), some(g), some(b), some(a)) => { some(rgba(r, g, b, a)) }
          _ => { fail_unrecognized(color) }
        }
    }

    #[doc="Parses a color specification in the form hsl(foo,bar,baz)"]
    fn parse_hsl(color : ~str) -> option<Color> {
        // Shave off the hsl( and the )
        let only_vals = color.substr(4u, color.len() - 5u);

        // split up h, s, and l
        let vals = only_vals.split_char(',');
        if vals.len() != 3u { return fail_unrecognized(color); }

        alt (float::from_str(vals[0]), float::from_str(vals[1]), 
             float::from_str(vals[2])) {
          (some(h), some(s), some(l)) => { some(hsl(h, s, l)) }
          _ => { fail_unrecognized(color) }
        }
    }

    #[doc="Parses a color specification in the form hsla(foo,bar,baz,qux)"]
    fn parse_hsla(color : ~str) -> option<Color> {
        // Shave off the hsla( and the )
        let only_vals = color.substr(5u, color.len() - 6u);

        let vals = only_vals.split_char(',');
        if vals.len() != 4u { return fail_unrecognized(color); }

        alt (float::from_str(vals[0]), float::from_str(vals[1]), 
             float::from_str(vals[2]), float::from_str(vals[3])) {
          (some(h), some(s), some(l), some(a)) => { some(hsla(h, s, l, a)) }
          _ => { fail_unrecognized(color) }
        }
    }

    // Currently colors are supported in rgb(a,b,c) form and also by
    // keywords for several common colors.
    // TODO: extend this
    fn parse_color(color : ~str) -> option<Color> {
        alt color {
          c if c.starts_with(~"rgb(") => parse_rgb(c),
          c if c.starts_with(~"rgba(") => parse_rgba(c),
          c if c.starts_with(~"hsl(") => parse_hsl(c),
          c if c.starts_with(~"hsla(") => parse_hsla(c),
          c => parse_by_name(c)
        }
    }
}

#[cfg(test)]
mod test {
    import css_colors::*;
    import option::unwrap;
    import parsing::parse_color;

    #[test]
    fn test_parse_by_name() {
        assert red().eq(unwrap(parse_color(~"red")));
        assert lime().eq(unwrap(parse_color(~"Lime")));
        assert blue().eq(unwrap(parse_color(~"BLUE")));
        assert green().eq(unwrap(parse_color(~"GreEN")));
        assert white().eq(unwrap(parse_color(~"white")));
        assert black().eq(unwrap(parse_color(~"Black")));
        assert gray().eq(unwrap(parse_color(~"Gray")));
        assert silver().eq(unwrap(parse_color(~"SiLvEr")));
        assert maroon().eq(unwrap(parse_color(~"maroon")));
        assert purple().eq(unwrap(parse_color(~"PURPLE")));
        assert fuchsia().eq(unwrap(parse_color(~"FUCHSIA")));
        assert olive().eq(unwrap(parse_color(~"oLiVe")));
        assert yellow().eq(unwrap(parse_color(~"yellow")));
        assert navy().eq(unwrap(parse_color(~"NAVY")));
        assert teal().eq(unwrap(parse_color(~"Teal")));
        assert aqua().eq(unwrap(parse_color(~"Aqua")));
        assert none == parse_color(~"foobarbaz");
    }

    #[test]
    fn test_parsing_rgb() {
        assert red().eq(unwrap(parse_color(~"rgb(255,0,0)")));
        assert red().eq(unwrap(parse_color(~"rgba(255,0,0,1.0)")));
        assert red().eq(unwrap(parse_color(~"rgba(255,0,0,1)")));
        assert lime().eq(unwrap(parse_color(~"rgba(0,255,0,1.00)")));
        assert rgb(1u8,2u8,3u8).eq(unwrap(parse_color(~"rgb(1,2,03)")));
        assert rgba(15u8,250u8,3u8,0.5).eq(unwrap(parse_color(~"rgba(15,250,3,.5)")));
        assert rgba(15u8,250u8,3u8,0.5).eq(unwrap(parse_color(~"rgba(15,250,3,0.5)")));
        assert none == parse_color(~"rbga(1,2,3)");
    }

    #[test]
    fn test_parsing_hsl() {
        assert red().eq(unwrap(parse_color(~"hsl(0,1,.5)")));
        assert lime().eq(unwrap(parse_color(~"hsl(120.0,1.0,.5)")));
        assert blue().eq(unwrap(parse_color(~"hsl(240.0,1.0,.5)")));
        assert green().eq(unwrap(parse_color(~"hsl(120.0,1.0,.25)")));
        assert white().eq(unwrap(parse_color(~"hsl(1.0,1.,1.0)")));
        assert white().eq(unwrap(parse_color(~"hsl(129.0,0.3,1.0)")));
        assert black().eq(unwrap(parse_color(~"hsl(231.2,0.75,0.0)")));
        assert black().eq(unwrap(parse_color(~"hsl(11.2,0.0,0.0)")));
        assert gray().eq(unwrap(parse_color(~"hsl(0.0,0.0,0.5)")));
        assert maroon().eq(unwrap(parse_color(~"hsl(0.0,1.0,0.25)")));
        assert purple().eq(unwrap(parse_color(~"hsl(300.0,1.0,0.25)")));
        assert fuchsia().eq(unwrap(parse_color(~"hsl(300,1.0,0.5)")));
        assert olive().eq(unwrap(parse_color(~"hsl(60.,1.0,0.25)")));
        assert yellow().eq(unwrap(parse_color(~"hsl(60.,1.0,0.5)")));
        assert navy().eq(unwrap(parse_color(~"hsl(240.0,1.0,.25)")));
        assert teal().eq(unwrap(parse_color(~"hsl(180.0,1.0,.25)")));
        assert aqua().eq(unwrap(parse_color(~"hsl(180.0,1.0,.5)")));
        assert none == parse_color(~"hsl(1,2,3,.4)");
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
    fn fuchsia() -> Color {
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

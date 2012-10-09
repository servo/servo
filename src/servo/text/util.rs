pub fn float_to_fixed(before: int, f: float) -> i32 {
    (1i32 << before) * (f as i32)
}

pub fn fixed_to_float(before: int, f: i32) -> float {
    f as float * 1.0f / ((1i32 << before) as float)
}

pub fn fixed_to_rounded_int(before: int, f: i32) -> int {
    let half = 1i32 << (before-1);
    if f > 0i32 {
        ((half + f) >> before) as int
    } else {
       -((half - f) >> before) as int
    }
}

/* Generate a 32-bit TrueType tag from its 4 charecters */
pub fn true_type_tag(a: char, b: char, c: char, d: char) -> u32 {
    (a << 24 | b << 16 | c << 8 | d) as u32
}

#[test]
fn test_true_type_tag() {
    assert true_type_tag('c', 'm', 'a', 'p') == 0x_63_6D_61_70_u32;
}

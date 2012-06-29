export true_type_tag;

#[doc = "Generate a 32-bit TrueType tag from its 4 charecters"]
fn true_type_tag(a: char, b: char, c: char, d: char) -> u32 {
    (a << 24 | b << 16 | c << 8 | d) as u32
}

#[test]
fn test_true_type_tag() {
    assert true_type_tag('c', 'm', 'a', 'p') == 0x_63_6D_61_70_u32;
}

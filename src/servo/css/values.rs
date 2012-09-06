import util::color::Color;

#[doc = "
  Defines how css rules, both selectors and style specifications, are
  stored.  CSS selector-matching rules, as presented by 
  http://www.w3.org/TR/CSS2/selector.html are represented by nested types.
"]

enum DisplayType {
    DisBlock,
    DisInline,
    DisNone
}

enum Unit {
    Auto,
    Percent(float),
    Mm(float),
    Pt(float),
    Px(float)
}

enum StyleDeclaration {
    BackgroundColor(Color),
    Display(DisplayType),
    FontSize(Unit),
    Height(Unit),
    TextColor(Color),
    Width(Unit)
}

enum Attr{
    Exists(~str),
    Exact(~str, ~str),
    Includes(~str, ~str),
    StartsWith(~str, ~str)
}
    
enum Selector{
    Element(~str, ~[Attr]),
    Child(~Selector, ~Selector),
    Descendant(~Selector, ~Selector),
    Sibling(~Selector, ~Selector)
}

type Rule = (~[~Selector], ~[StyleDeclaration]);

type Stylesheet = ~[~Rule];

#[doc="Convert between units measured in millimeteres and pixels"]
pure fn MmToPx(u : Unit) -> Unit {
    match u {
        Mm(m) => Px(m * 3.7795),
        _ => fail ~"Calling MmToPx on a unit that is not a Mm"
    }
}

#[doc="Convert between units measured in points and pixels"]
pure fn PtToPx(u : Unit) -> Unit {
    match u {
        Pt(m) => Px(m * 1.3333),
        _ => fail ~"Calling PtToPx on a unit that is not a Pt"
    }
}

impl DisplayType: cmp::Eq {
    pure fn eq(&&other: DisplayType) -> bool {
        self as uint == other as uint
    }
}

impl Unit: cmp::Eq {
    pure fn eq(&&other: Unit) -> bool {
        match (self, other) {
          (Auto, Auto) => true,
          (Auto, _) => false,
          (Percent(a), Percent(b)) => a == b,
          (Percent(*), _) => false,
          (Mm(a), Mm(b)) => a == b,
          (Mm(*), _) => false,
          (Pt(a), Pt(b)) => a == b,
          (Pt(*), _) => false,
          (Px(a), Px(b)) => a == b,
          (Px(*), _) => false
        }
    }
}

impl StyleDeclaration: cmp::Eq {
    pure fn eq(&&other: StyleDeclaration) -> bool {
        match (self, other) {
          (BackgroundColor(a), BackgroundColor(b)) => a == b,
          (Display(a), Display(b)) => a == b,
          (FontSize(a), FontSize(b)) => a == b,
          (Height(a), Height(b)) => a == b,
          (TextColor(a), TextColor(b)) => a == b,
          (Width(a), Width(b)) => a == b,

          (BackgroundColor(*), _)
          | (Display(*), _)
          | (FontSize(*), _)
          | (Height(*), _)
          | (TextColor(*), _)
          | (Width(*), _) => false
        }
    }
}

impl Attr: cmp::Eq {
    pure fn eq(&&other: Attr) -> bool {
        match (copy self, copy other) {
          (Exists(a), Exists(b)) => a == b,

          (Exact(a, aa), Exact(b, bb))
          | (Includes(a, aa), Includes(b, bb))
          | (StartsWith(a, aa), StartsWith(b, bb)) => a == b && aa == bb,

          (Exists(*), _)
          | (Exact(*), _)
          | (Includes(*), _)
          | (StartsWith(*), _) => false
        }
    }
}

impl Selector: cmp::Eq {
    pure fn eq(&&other: Selector) -> bool {
        // FIXME: Lots of copying here
        match (copy self, copy other) {
          (Element(s_a, attrs_a), Element(s_b, attrs_b)) => s_a == s_b && attrs_a == attrs_b,

          (Child(s1a, s2a), Child(s1b, s2b))
          | (Descendant(s1a, s2a), Descendant(s1b, s2b))
          | (Sibling(s1a, s2a), Sibling(s1b, s2b)) => {
            s1a == s1b && s2a == s2b
          }

          (Element(*), _) => false,
          (Child(*), _) => false,
          (Descendant(*), _) => false,
          (Sibling(*), _) => false
        }
    }
}
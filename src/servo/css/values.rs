use SharedColor = util::color::Color;
use cmp::Eq;

#[doc = "
  Defines how css rules, both selectors and style specifications, are
  stored.  CSS selector-matching rules, as presented by 
  http://www.w3.org/TR/CSS2/selector.html are represented by nested types.
"]

// CSS Units

enum ParseResult<T> {
    Value(T),
    CSSInitial,
    CSSInherit,
    Fail
}

enum CSSValue<T : Copy> {
    Specified(T),
    Initial,
    Inherit
}

impl<T : Copy> ParseResult<T> {
    pure fn extract<U>(f: fn(CSSValue<T>) -> U) -> Option<U> { extract(self, f) }
}

pure fn extract<T : Copy, U>(res: ParseResult<T>, f: fn(CSSValue<T>) -> U) -> Option<U> {
    match res {
        Fail => None,
        CSSInitial => Some(f(Initial)),
        CSSInherit => Some(f(Inherit)),
        Value(x) => Some(f(Specified(x)))
    }
}

impl<T: Eq Copy> CSSValue<T> : Eq {
    pure fn eq(&&other: CSSValue<T>) -> bool {
        match (self, other) {
            (Initial, Initial) => true,
            (Inherit, Inherit) => true,
            (Specified(a), Specified(b)) => a == b,
            _ => false
        }
    }
    pure fn ne(&&other: CSSValue<T>) -> bool {
        return !self.eq(other);
    }
}

enum Auto = ();

pub enum Length {
    Em(float), // normalized to 'em'
    Px(float) // normalized to 'px'
}

impl Length {
    pure fn rel() -> float {
        match self {
            Em(x) => x,
            _ => fail ~"attempted to access relative unit of an absolute length"
        }
    }
    pure fn abs() -> float {
        match self {
            Em(x) => x,
            _ => fail ~"attempted to access relative unit of an absolute length"
        }
    }
}

pub enum BoxSizing { // used by width, height, top, left, etc
    BoxLength(Length),
    BoxPercent(float),
    BoxAuto
}

enum AbsoluteSize {
    XXSmall,
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
    XXLarge
}

enum RelativeSize {
    Larger,
    Smaller
}

// CSS property values

enum CSSBackgroundAttachment {
    BgAttachScroll,
    BgAttachFixed
}

enum CSSBackgroundColor {
    BgColor(SharedColor),
    BgTransparent
}

enum CSSBackgroundRepeat {
    BgRepeat,
    BgRepeatX,
    BgRepeatY,
    BgNoRepeat
}

enum CSSColor {
    TextColor(SharedColor)
}

enum CSSDirection {
    DirectionLtr,
    DirectionRtl
}

enum CSSDisplay {
    DisplayInline,
    DisplayBlock,
    DisplayListItem,
    DisplayInlineBlock,
    DisplayTable,
    DisplayInlineTable,
    DisplayTableRowGroup,
    DisplayTableHeaderGroup,
    DisplayTableFooterGroup,
    DisplayTableRow,
    DisplayTableColumnGroup,
    DisplayTableColumn,
    DisplayTableCell,
    DisplayTableCaption,
    DisplayNone
}

enum CSSFloat {
    FloatLeft,
    FloatRight,
    FloatNone
}

enum CSSFontSize {
    AbsoluteSize(AbsoluteSize),
    RelativeSize(RelativeSize),
    LengthSize(Length),
    PercentSize(float)
}

// Stylesheet parts

enum StyleDeclaration {
    BackgroundColor(CSSValue<CSSBackgroundColor>),
    Display(CSSValue<CSSDisplay>),
    FontSize(CSSValue<CSSFontSize>),
    Height(CSSValue<BoxSizing>),
    Color(CSSValue<CSSColor>),
    Width(CSSValue<BoxSizing>)
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


impl Length: cmp::Eq {
    pure fn eq(&&other: Length) -> bool {
        match (self, other) {
          (Em(a), Em(b)) => a == b,
          (Px(a), Px(b)) => a == b,
          (_, _) => false
        }
    }
    pure fn ne(&&other: Length) -> bool {
        return !self.eq(other);
    }
}

impl BoxSizing: cmp::Eq {
    pure fn eq(&&other: BoxSizing) -> bool {
        match (self, other) {
          (BoxLength(a), BoxLength(b)) => a == b,
          (BoxPercent(a), BoxPercent(b)) => a == b,
          (BoxAuto, BoxAuto) => true,
          (_, _) => false
        }
    }
    pure fn ne(&&other: BoxSizing) -> bool {
        return !self.eq(other);
    }
}

impl AbsoluteSize: cmp::Eq {
    pure fn eq(&&other: AbsoluteSize) -> bool {
        self as uint == other as uint
    }
    pure fn ne(&&other: AbsoluteSize) -> bool {
        return !self.eq(other);
    }
}

impl RelativeSize: cmp::Eq {
    pure fn eq(&&other: RelativeSize) -> bool {
        self as uint == other as uint
    }
    pure fn ne(&&other: RelativeSize) -> bool {
        return !self.eq(other);
    }
}



impl CSSBackgroundColor: cmp::Eq {
    pure fn eq(&&other: CSSBackgroundColor) -> bool {
        match (self, other) {
            (BgColor(a), BgColor(b)) => a == b,
            (BgTransparent, BgTransparent) => true,
            (_, _) => false
        }
    }
    pure fn ne(&&other: CSSBackgroundColor) -> bool {
        return !self.eq(other);
    }
}


impl CSSColor: cmp::Eq {
    pure fn eq(&&other: CSSColor) -> bool {
        match (self, other) {
            (TextColor(a), TextColor(b)) => a == b
        }
    }
    pure fn ne(&&other: CSSColor) -> bool {
        return !self.eq(other);
    }
}

impl CSSDisplay: cmp::Eq {
    pure fn eq(&&other: CSSDisplay) -> bool {
        self as uint == other as uint
    }
    pure fn ne(&&other: CSSDisplay) -> bool {
        return !self.eq(other);
    }
}


impl CSSFontSize: cmp::Eq {
    pure fn eq(&&other: CSSFontSize) -> bool {
        match (self, other) {
            (AbsoluteSize(a), AbsoluteSize(b)) => a == b,
            (RelativeSize(a), RelativeSize(b)) => a == b,
            (LengthSize(a),   LengthSize(b))   => a == b,
            (PercentSize(a),  PercentSize(b))  => a == b,
            (_, _) => false
        }
    }
    pure fn ne(&&other: CSSFontSize) -> bool {
        return !self.eq(other);
    }
}
/*
impl StyleDeclaration: cmp::Eq {
    pure fn eq(&&other: StyleDeclaration) -> bool {
        match (self, other) {
          (BackgroundColor(a), BackgroundColor(b)) => a == b,
          (Display(a), Display(b)) => a == b,
          (FontSize(a), FontSize(b)) => a == b,
          (Height(a), Height(b)) => a == b,
          (Color(a), Color(b)) => a == b,
          (Width(a), Width(b)) => a == b,

          (BackgroundColor(*), _)
          | (Display(*), _)
          | (FontSize(*), _)
          | (Height(*), _)
          | (Color(*), _)
          | (Width(*), _) => false
        }
    }
}*/

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
    pure fn ne(&&other: Attr) -> bool {
        return !self.eq(other);
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
    pure fn ne(&&other: Selector) -> bool {
        return !self.eq(other);
    }
}
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

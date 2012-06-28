import util::color::Color;

#[doc = "
  Defines how css rules, both selectors and style specifications, are
  stored.  CSS selector-matching rules, as presented by 
  http://www.w3.org/TR/CSS2/selector.html are represented by nested, structural types,
"]

enum DisplayType {
    DisBlock,
    DisInline,
    DisNone
}

enum Unit {
    Auto,
    Percent(float),
    In(float),
    Mm(float),
    Cm(float),
    Em(float),
    Ex(float),
    Pt(float),
    Pc(float),
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

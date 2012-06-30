import util::color::Color;

enum DisplayType{
    DisBlock,
    DisInline,
    DisNone
}

enum StyleDeclaration{
    FontSize(uint), // Currently assumes format '# pt'
    Display(DisplayType),
    TextColor(Color),
    BackgroundColor(Color)
}

enum Attr{
    Exists(str),
    Exact(str, str),
    Includes(str, str),
    StartsWith(str, str)
}
    
enum Selector{
    Element(str, [Attr]),
    Child(~Selector, ~Selector),
    Descendant(~Selector, ~Selector),
    Sibling(~Selector, ~Selector)
}

type Rule = ([~Selector], [StyleDeclaration]);

type Stylesheet = [~Rule];

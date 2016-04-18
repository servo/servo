<%page args="helpers" />

${helpers.new_style_struct("Margin", is_inherited=False, gecko_name="nsStyleMargin")}

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("margin-" + side, "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Length(Au(0))")}
% endfor

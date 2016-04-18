<%page args="helpers" />

${helpers.new_style_struct("Padding", is_inherited=False, gecko_name="nsStylePadding")}

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("padding-" + side, "LengthOrPercentage",
                              "computed::LengthOrPercentage::Length(Au(0))",
                              "parse_non_negative")}
% endfor

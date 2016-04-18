<%page args="helpers" />

// CSS Fragmentation Module Level 3
// https://www.w3.org/TR/css-break-3/
${helpers.switch_to_style_struct("Border")}

${helpers.single_keyword("box-decoration-break", "slice clone", products="gecko")}

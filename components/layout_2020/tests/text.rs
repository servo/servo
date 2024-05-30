/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod text {
    use layout_2020::flow::inline::construct::WhitespaceCollapse;
    use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;

    #[test]
    fn test_collapse_whitespace() {
        let collapse = |input: &str, white_space_collapse, trim_beginning_white_space| {
            WhitespaceCollapse::new(
                input.chars(),
                white_space_collapse,
                trim_beginning_white_space,
            )
            .collect::<String>()
        };

        let output = collapse("H ", WhiteSpaceCollapse::Collapse, false);
        assert_eq!(output, "H ");

        let output = collapse(" W", WhiteSpaceCollapse::Collapse, true);
        assert_eq!(output, "W");

        let output = collapse(" W", WhiteSpaceCollapse::Collapse, false);
        assert_eq!(output, " W");

        let output = collapse(" H  W", WhiteSpaceCollapse::Collapse, false);
        assert_eq!(output, " H W");

        let output = collapse("\n   H  \n \t  W", WhiteSpaceCollapse::Collapse, false);
        assert_eq!(output, " H W");

        let output = collapse("\n   H  \n \t  W   \n", WhiteSpaceCollapse::Preserve, false);
        assert_eq!(output, "\n   H  \n \t  W   \n");

        let output = collapse(
            "\n   H  \n \t  W   \n ",
            WhiteSpaceCollapse::PreserveBreaks,
            false,
        );
        assert_eq!(output, "\nH\nW\n");

        let output = collapse("Hello \n World", WhiteSpaceCollapse::PreserveBreaks, true);
        assert_eq!(output, "Hello\nWorld");

        let output = collapse(" \n World", WhiteSpaceCollapse::PreserveBreaks, true);
        assert_eq!(output, "\nWorld");

        let output = collapse(" ", WhiteSpaceCollapse::Collapse, true);
        assert_eq!(output, "");

        let output = collapse(" ", WhiteSpaceCollapse::Collapse, false);
        assert_eq!(output, " ");

        let output = collapse("\n        ", WhiteSpaceCollapse::Collapse, true);
        assert_eq!(output, "");

        let output = collapse("\n        ", WhiteSpaceCollapse::Collapse, false);
        assert_eq!(output, " ");
    }
}

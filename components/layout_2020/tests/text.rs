/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod text {
    use layout_2020::flow::text_run::WhitespaceCollapse;
    use style::computed_values::white_space::T as WhiteSpace;

    #[test]
    fn test_collapse_whitespace() {
        let collapse = |input: &str, white_space, trim_beginning_white_space| {
            WhitespaceCollapse::new(input.chars(), white_space, trim_beginning_white_space)
                .collect::<String>()
        };

        let output = collapse("H ", WhiteSpace::Normal, false);
        assert_eq!(output, "H ");

        let output = collapse(" W", WhiteSpace::Normal, true);
        assert_eq!(output, "W");

        let output = collapse(" W", WhiteSpace::Normal, false);
        assert_eq!(output, " W");

        let output = collapse(" H  W", WhiteSpace::Normal, false);
        assert_eq!(output, " H W");

        let output = collapse("\n   H  \n \t  W", WhiteSpace::Normal, false);
        assert_eq!(output, " H W");

        let output = collapse("\n   H  \n \t  W   \n", WhiteSpace::Pre, false);
        assert_eq!(output, "\n   H  \n \t  W   \n");

        let output = collapse("\n   H  \n \t  W   \n ", WhiteSpace::PreLine, false);
        assert_eq!(output, "\nH\nW\n");

        let output = collapse("Hello \n World", WhiteSpace::PreLine, true);
        assert_eq!(output, "Hello\nWorld");

        let output = collapse(" \n World", WhiteSpace::PreLine, true);
        assert_eq!(output, "\nWorld");

        let output = collapse(" ", WhiteSpace::Normal, true);
        assert_eq!(output, "");

        let output = collapse(" ", WhiteSpace::Normal, false);
        assert_eq!(output, " ");

        let output = collapse("\n        ", WhiteSpace::Normal, true);
        assert_eq!(output, "");

        let output = collapse("\n        ", WhiteSpace::Normal, false);
        assert_eq!(output, " ");
    }
}

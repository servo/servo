/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::{assert_computed_serialization, parse, parse_entirely};
use style::parser::Parse;
use style::values::specified::position::*;
use style_traits::ToCss;

#[test]
fn test_position() {
    // Serialization is not actually specced
    // though these are the values expected by basic-shape
    // https://github.com/w3c/csswg-drafts/issues/368
    assert_roundtrip_with_context!(Position::parse, "center", "center center");
    assert_roundtrip_with_context!(Position::parse, "top left", "left top");
    assert_roundtrip_with_context!(Position::parse, "left top", "left top");
    assert_roundtrip_with_context!(Position::parse, "top right", "right top");
    assert_roundtrip_with_context!(Position::parse, "right top", "right top");
    assert_roundtrip_with_context!(Position::parse, "bottom left", "left bottom");
    assert_roundtrip_with_context!(Position::parse, "left bottom", "left bottom");
    assert_roundtrip_with_context!(Position::parse, "left center", "left center");
    assert_roundtrip_with_context!(Position::parse, "right center", "right center");
    assert_roundtrip_with_context!(Position::parse, "center top", "center top");
    assert_roundtrip_with_context!(Position::parse, "center bottom", "center bottom");
    assert_roundtrip_with_context!(Position::parse, "center 10px", "center 10px");
    assert_roundtrip_with_context!(Position::parse, "center 10%", "center 10%");
    assert_roundtrip_with_context!(Position::parse, "right 10%", "right 10%");

    // Only keywords can be reordered
    assert!(parse_entirely(Position::parse, "top 40%").is_err());
    assert!(parse_entirely(Position::parse, "40% left").is_err());

    // 3 and 4 value serialization
    assert_roundtrip_with_context!(Position::parse, "left 10px top 15px", "left 10px top 15px");
    assert_roundtrip_with_context!(Position::parse, "top 15px left 10px", "left 10px top 15px");
    assert_roundtrip_with_context!(Position::parse, "left 10% top 15px", "left 10% top 15px");
    assert_roundtrip_with_context!(Position::parse, "top 15px left 10%", "left 10% top 15px");
    assert_roundtrip_with_context!(Position::parse, "left top 15px", "left top 15px");
    assert_roundtrip_with_context!(Position::parse, "top 15px left", "left top 15px");
    assert_roundtrip_with_context!(Position::parse, "left 10px top", "left 10px top");
    assert_roundtrip_with_context!(Position::parse, "top left 10px", "left 10px top");
    assert_roundtrip_with_context!(Position::parse, "right 10px bottom", "right 10px bottom");
    assert_roundtrip_with_context!(Position::parse, "bottom right 10px", "right 10px bottom");
    assert_roundtrip_with_context!(Position::parse, "center right 10px", "right 10px center");
    assert_roundtrip_with_context!(Position::parse, "center bottom 10px", "center bottom 10px");

    // Invalid 3 value positions
    assert!(parse_entirely(Position::parse, "20px 30px 20px").is_err());
    assert!(parse_entirely(Position::parse, "top 30px 20px").is_err());
    assert!(parse_entirely(Position::parse, "50% bottom 20%").is_err());

    // Only horizontal and vertical keywords can have positions
    assert!(parse_entirely(Position::parse, "center 10px left 15px").is_err());
    assert!(parse_entirely(Position::parse, "center 10px 15px").is_err());
    assert!(parse_entirely(Position::parse, "center 10px bottom").is_err());

    // "Horizontal Horizontal" or "Vertical Vertical" positions cause error
    assert!(parse_entirely(Position::parse, "left right").is_err());
    assert!(parse_entirely(Position::parse, "left 10px right").is_err());
    assert!(parse_entirely(Position::parse, "left 10px right 15%").is_err());
    assert!(parse_entirely(Position::parse, "top bottom").is_err());
    assert!(parse_entirely(Position::parse, "top 10px bottom").is_err());
    assert!(parse_entirely(Position::parse, "top 10px bottom 15%").is_err());

    // Logical keywords are not supported in Position yet.
    assert!(parse(Position::parse, "x-start").is_err());
    assert!(parse(Position::parse, "y-end").is_err());
    assert!(parse(Position::parse, "x-start y-end").is_err());
    assert!(parse(Position::parse, "x-end 10px").is_err());
    assert!(parse(Position::parse, "y-start 20px").is_err());
    assert!(parse(Position::parse, "x-start bottom 10%").is_err());
    assert!(parse_entirely(Position::parse, "left y-start 10%").is_err());
    assert!(parse(Position::parse, "x-start 20px y-end 10%").is_err());
}

#[test]
fn test_horizontal_position() {
    // One value serializations.
    assert_roundtrip_with_context!(HorizontalPosition::parse, "20px", "20px");
    assert_roundtrip_with_context!(HorizontalPosition::parse, "25%", "25%");
    assert_roundtrip_with_context!(HorizontalPosition::parse, "center", "center");
    assert_roundtrip_with_context!(HorizontalPosition::parse, "left", "left");
    assert_roundtrip_with_context!(HorizontalPosition::parse, "right", "right");

    // Two value serializations.
    assert_roundtrip_with_context!(HorizontalPosition::parse, "right 10px", "right 10px");

    // Invalid horizontal positions.
    assert!(parse(HorizontalPosition::parse, "top").is_err());
    assert!(parse(HorizontalPosition::parse, "bottom").is_err());
    assert!(parse(HorizontalPosition::parse, "y-start").is_err());
    assert!(parse(HorizontalPosition::parse, "y-end").is_err());
    assert!(parse(HorizontalPosition::parse, "y-end 20px ").is_err());
    assert!(parse(HorizontalPosition::parse, "bottom 20px").is_err());
    assert!(parse(HorizontalPosition::parse, "bottom top").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "20px y-end").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "20px top").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "left center").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "left top").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "left right").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "20px 30px").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "10px left").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "x-end 20%").is_err());
    assert!(parse_entirely(HorizontalPosition::parse, "20px x-start").is_err());

    // Logical keywords are not supported in Position yet.
    assert!(parse(HorizontalPosition::parse, "x-start").is_err());
    assert!(parse(HorizontalPosition::parse, "x-end").is_err());
}

#[test]
fn test_vertical_position() {
    // One value serializations.
    assert_roundtrip_with_context!(VerticalPosition::parse, "20px", "20px");
    assert_roundtrip_with_context!(VerticalPosition::parse, "25%", "25%");
    assert_roundtrip_with_context!(VerticalPosition::parse, "center", "center");
    assert_roundtrip_with_context!(VerticalPosition::parse, "top", "top");
    assert_roundtrip_with_context!(VerticalPosition::parse, "bottom", "bottom");

    // Two value serializations.
    assert_roundtrip_with_context!(VerticalPosition::parse, "bottom 10px", "bottom 10px");

    // Invalid vertical positions.
    assert!(parse(VerticalPosition::parse, "left").is_err());
    assert!(parse(VerticalPosition::parse, "right").is_err());
    assert!(parse(VerticalPosition::parse, "x-start").is_err());
    assert!(parse(VerticalPosition::parse, "x-end").is_err());
    assert!(parse(VerticalPosition::parse, "x-end 20px").is_err());
    assert!(parse(VerticalPosition::parse, "left 20px").is_err());
    assert!(parse(VerticalPosition::parse, "left center").is_err());
    assert!(parse(VerticalPosition::parse, "left top").is_err());
    assert!(parse(VerticalPosition::parse, "left right").is_err());
    assert!(parse_entirely(VerticalPosition::parse, "20px x-end").is_err());
    assert!(parse_entirely(VerticalPosition::parse, "20px right").is_err());
    assert!(parse_entirely(VerticalPosition::parse, "bottom top").is_err());
    assert!(parse_entirely(VerticalPosition::parse, "20px 30px").is_err());
    assert!(parse_entirely(VerticalPosition::parse, "10px top").is_err());
    assert!(parse_entirely(VerticalPosition::parse, "y-end 20%").is_err());
    assert!(parse_entirely(VerticalPosition::parse, "20px y-start").is_err());

    // Logical keywords are not supported in Position yet.
    assert!(parse(VerticalPosition::parse, "y-start").is_err());
    assert!(parse(VerticalPosition::parse, "y-end").is_err());
}

#[test]
fn test_grid_auto_flow() {
    use style::properties::longhands::grid_auto_flow;

    assert_roundtrip_with_context!(grid_auto_flow::parse, "row dense", "row dense");
    assert_roundtrip_with_context!(grid_auto_flow::parse, "dense row", "row dense");
    assert_roundtrip_with_context!(grid_auto_flow::parse, "column dense", "column dense");
    assert_roundtrip_with_context!(grid_auto_flow::parse, "dense column", "column dense");
    assert_roundtrip_with_context!(grid_auto_flow::parse, "dense", "row dense");
    assert_roundtrip_with_context!(grid_auto_flow::parse, "row", "row");
    assert_roundtrip_with_context!(grid_auto_flow::parse, "column", "column");

    // Neither row, column or dense can be repeated
    assert!(parse(grid_auto_flow::parse, "dense dense").is_err());
    assert!(parse(grid_auto_flow::parse, "row row").is_err());
    assert!(parse(grid_auto_flow::parse, "column column").is_err());
    assert!(parse(grid_auto_flow::parse, "row dense dense").is_err());
    assert!(parse(grid_auto_flow::parse, "column dense dense").is_err());

    // Only row, column, dense idents are allowed
    assert!(parse(grid_auto_flow::parse, "dense 1").is_err());
    assert!(parse(grid_auto_flow::parse, "column 'dense'").is_err());
    assert!(parse(grid_auto_flow::parse, "column 2px dense").is_err());
}

#[test]
fn test_grid_auto_rows_columns() {
    use style::properties::longhands::grid_auto_rows;

    // the grammar is <track-size>+ but gecko supports only a single value, so we've clamped ourselves
    assert_roundtrip_with_context!(grid_auto_rows::parse, "55%");
    assert_roundtrip_with_context!(grid_auto_rows::parse, "0.5fr");
    assert_roundtrip_with_context!(grid_auto_rows::parse, "fit-content(11%)");
    // only <inflexible-breadth> is allowed in first arg of minmax
    assert!(parse(grid_auto_rows::parse, "minmax(1fr, max-content)").is_err());
}

#[test]
fn test_grid_template_rows_columns() {
    use style::properties::longhands::grid_template_rows;

    assert_roundtrip_with_context!(grid_template_rows::parse, "none");      // none keyword
    // <track-size>{2} with `<track-breadth> minmax(<inflexible-breadth>, <track-breadth>)`
    assert_roundtrip_with_context!(grid_template_rows::parse, "1fr minmax(min-content, 1fr)");
    // <track-size> with <track-breadth> as <length-percentage>
    assert_roundtrip_with_context!(grid_template_rows::parse, "calc(4em + 5px)");
    // <track-size> with <length> followed by <track-repeat> with `<track-size>{3}` (<flex>, auto, minmax)
    assert_roundtrip_with_context!(grid_template_rows::parse,
                                   "10px repeat(2, 1fr auto minmax(200px, 1fr))",
                                   "10px 1fr auto minmax(200px, 1fr) 1fr auto minmax(200px, 1fr)");
    // <track-repeat> with `<track-size> <line-names>` followed by <track-size>
    assert_roundtrip_with_context!(grid_template_rows::parse,
                                   "repeat(2, 10px [col-start] 250px [col-end]) 10px",
                                   "10px [col-start] 250px [col-end] 10px [col-start] 250px [col-end] 10px");
    // mixture of <track-size>, <track-repeat> and <line-names>
    assert_roundtrip_with_context!(grid_template_rows::parse,
                                   "[a] auto [b] minmax(min-content, 1fr) [b c d] repeat(2, 40px [e] 30px) [i]",
                                   "[a] auto [b] minmax(min-content, 1fr) [b c d] 40px [e] 30px 40px [e] 30px [i]");
    assert!(parse(grid_template_rows::parse, "subgrid").is_ok());

    // no span allowed in <line-names>
    assert!(parse(grid_template_rows::parse, "[a span] 10px").is_err());
    // <track-list> needs at least one <track-size> | <track-repeat>
    assert!(parse(grid_template_rows::parse, "[a b c]").is_err());
    // at least one argument of <fixed-size> should be a <fixed-breadth> (i.e., <length-percentage>)
    assert!(parse(grid_template_rows::parse, "[a b] repeat(auto-fill, 50px) minmax(auto, 1fr)").is_err());
    // fit-content is not a <fixed-size>
    assert!(parse(grid_template_rows::parse, "[a b] repeat(auto-fill, fit-content(20%))").is_err());
    // <auto-track-list> only allows <fixed-size> | <fixed-repeat>
    assert!(parse(grid_template_rows::parse, "[a] repeat(2, auto) repeat(auto-fill, 10px)").is_err());
    // only <inflexible-breadth> allowed in <auto-track-repeat>
    assert!(parse(grid_template_rows::parse, "[a] repeat(auto-fill, 1fr)").is_err());
    // <auto-track-repeat> is allowed only once
    assert!(parse(grid_template_rows::parse, "[a] repeat(auto-fit, [b] 8px) [c] repeat(auto-fill, [c] 8px)").is_err());
}

#[test]
fn test_computed_grid_template_rows_colums() {
    use style::properties::longhands::grid_template_rows;

    assert_computed_serialization(grid_template_rows::parse,
        "[a] repeat(calc(1 + 1), [b] auto)", "[a b] auto [b] auto");

    assert_computed_serialization(grid_template_rows::parse,
        "[a] repeat(2, [b c] auto [e] auto [d])",
        "[a b c] auto [e] auto [d b c] auto [e] auto [d]");

    assert_computed_serialization(grid_template_rows::parse,
        "[a] 50px [b] 10% [b c d] repeat(2, [e] 40px [f]) [g] repeat(auto-fill, [h i] 20px [j]) [k] 10px [l]",
        "[a] 50px [b] 10% [b c d e] 40px [f e] 40px [f g] repeat(auto-fill, [h i] 20px [j]) [k] 10px [l]");

    assert_computed_serialization(grid_template_rows::parse,
        "10px repeat(2, 1fr auto minmax(200px, 1fr))",
        "10px minmax(auto, 1fr) auto minmax(200px, 1fr) minmax(auto, 1fr) auto minmax(200px, 1fr)");

    assert_computed_serialization(grid_template_rows::parse,
        "subgrid [a] [] repeat(auto-fill, [])", "subgrid [a] [] repeat(auto-fill, [])");

    assert_computed_serialization(grid_template_rows::parse,
        "subgrid [a] [b] repeat(2, [c d] [] [e]) [] repeat(auto-fill, [])",
        "subgrid [a] [b] [c d] [] [e] [c d] [] [e] [] repeat(auto-fill, [])");
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="flex-flow" sub_properties="flex-direction flex-wrap" extra_prefixes="webkit"
                    spec="https://drafts.csswg.org/css-flexbox/#flex-flow-property">
    use properties::longhands::{flex_direction, flex_wrap};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut direction = None;
        let mut wrap = None;
        loop {
            if direction.is_none() {
                if let Ok(value) = input.try(|input| flex_direction::parse(context, input)) {
                    direction = Some(value);
                    continue
                }
            }
            if wrap.is_none() {
                if let Ok(value) = input.try(|input| flex_wrap::parse(context, input)) {
                    wrap = Some(value);
                    continue
                }
            }
            break
        }

        if direction.is_none() && wrap.is_none() {
            return Err(())
        }
        Ok(expanded! {
            flex_direction: unwrap_or_initial!(flex_direction, direction),
            flex_wrap: unwrap_or_initial!(flex_wrap, wrap),
        })
    }


    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.flex_direction.to_css(dest)?;
            dest.write_str(" ")?;
            self.flex_wrap.to_css(dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="flex" sub_properties="flex-grow flex-shrink flex-basis" extra_prefixes="webkit"
                    spec="https://drafts.csswg.org/css-flexbox/#flex-property">
    use values::specified::Number;

    fn parse_flexibility(context: &ParserContext, input: &mut Parser)
                         -> Result<(Number, Option<Number>),()> {
        let grow = try!(Number::parse_non_negative(context, input));
        let shrink = input.try(|i| Number::parse_non_negative(context, i)).ok();
        Ok((grow, shrink))
    }

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut grow = None;
        let mut shrink = None;
        let mut basis = None;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(expanded! {
                flex_grow: Number::new(0.0),
                flex_shrink: Number::new(0.0),
                flex_basis: longhands::flex_basis::SpecifiedValue::auto(),
            })
        }
        loop {
            if grow.is_none() {
                if let Ok((flex_grow, flex_shrink)) = input.try(|i| parse_flexibility(context, i)) {
                    grow = Some(flex_grow);
                    shrink = flex_shrink;
                    continue
                }
            }
            if basis.is_none() {
                if let Ok(value) = input.try(|input| longhands::flex_basis::parse_specified(context, input)) {
                    basis = Some(value);
                    continue
                }
            }
            break
        }

        if grow.is_none() && basis.is_none() {
            return Err(())
        }
        Ok(expanded! {
            flex_grow: grow.unwrap_or(Number::new(1.0)),
            flex_shrink: shrink.unwrap_or(Number::new(1.0)),
            flex_basis: basis.unwrap_or(longhands::flex_basis::SpecifiedValue::zero()),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.flex_grow.to_css(dest));
            try!(dest.write_str(" "));

            try!(self.flex_shrink.to_css(dest));
            try!(dest.write_str(" "));

            self.flex_basis.to_css(dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="grid-gap" sub_properties="grid-row-gap grid-column-gap"
                    spec="https://drafts.csswg.org/css-grid/#propdef-grid-gap"
                    products="gecko">
  use properties::longhands::{grid_row_gap, grid_column_gap};

  pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
      let row_gap = grid_row_gap::parse(context, input)?;
      let column_gap = input.try(|input| grid_column_gap::parse(context, input)).unwrap_or(row_gap.clone());

      Ok(expanded! {
        grid_row_gap: row_gap,
        grid_column_gap: column_gap,
      })
  }

  impl<'a> ToCss for LonghandsToSerialize<'a>  {
      fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
          if self.grid_row_gap == self.grid_column_gap {
            self.grid_row_gap.to_css(dest)
          } else {
            self.grid_row_gap.to_css(dest)?;
            dest.write_str(" ")?;
            self.grid_column_gap.to_css(dest)
          }
      }
  }

</%helpers:shorthand>

% for kind in ["row", "column"]:
<%helpers:shorthand name="grid-${kind}" sub_properties="grid-${kind}-start grid-${kind}-end"
                    spec="https://drafts.csswg.org/css-grid/#propdef-grid-${kind}"
                    products="gecko">
    use values::specified::GridLine;
    use parser::Parse;

    // NOTE: Since both the shorthands have the same code, we should (re-)use code from one to implement
    // the other. This might not be a big deal for now, but we should consider looking into this in the future
    // to limit the amount of code generated.
    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let start = input.try(|i| GridLine::parse(context, i))?;
        let end = if input.try(|i| i.expect_delim('/')).is_ok() {
            GridLine::parse(context, input)?
        } else {
            let mut line = GridLine::default();
            if start.line_num.is_none() && !start.is_span {
                line.ident = start.ident.clone();       // ident from start value should be taken
            }

            line
        };

        Ok(expanded! {
            grid_${kind}_start: start,
            grid_${kind}_end: end,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.grid_${kind}_start.to_css(dest)?;
            dest.write_str(" / ")?;
            self.grid_${kind}_end.to_css(dest)
        }
    }
</%helpers:shorthand>
% endfor

<%helpers:shorthand name="grid-area"
                    sub_properties="grid-row-start grid-row-end grid-column-start grid-column-end"
                    spec="https://drafts.csswg.org/css-grid/#propdef-grid-area"
                    products="gecko">
    use values::specified::GridLine;
    use parser::Parse;

    // The code is the same as `grid-{row,column}` except that this can have four values at most.
    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        fn line_with_ident_from(other: &GridLine) -> GridLine {
            let mut this = GridLine::default();
            if other.line_num.is_none() && !other.is_span {
                this.ident = other.ident.clone();
            }

            this
        }

        let row_start = input.try(|i| GridLine::parse(context, i))?;
        let (column_start, row_end, column_end) = if input.try(|i| i.expect_delim('/')).is_ok() {
            let column_start = GridLine::parse(context, input)?;
            let (row_end, column_end) = if input.try(|i| i.expect_delim('/')).is_ok() {
                let row_end = GridLine::parse(context, input)?;
                let column_end = if input.try(|i| i.expect_delim('/')).is_ok() {
                    GridLine::parse(context, input)?
                } else {        // grid-column-end has not been given
                    line_with_ident_from(&column_start)
                };

                (row_end, column_end)
            } else {        // grid-row-start and grid-column-start has been given
                let row_end = line_with_ident_from(&row_start);
                let column_end = line_with_ident_from(&column_start);
                (row_end, column_end)
            };

            (column_start, row_end, column_end)
        } else {        // only grid-row-start is given
            let line = line_with_ident_from(&row_start);
            (line.clone(), line.clone(), line)
        };

        Ok(expanded! {
            grid_row_start: row_start,
            grid_row_end: row_end,
            grid_column_start: column_start,
            grid_column_end: column_end,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.grid_row_start.to_css(dest)?;
            let values = [&self.grid_column_start, &self.grid_row_end, &self.grid_column_end];
            for value in &values {
                dest.write_str(" / ")?;
                value.to_css(dest)?;
            }

            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="grid-template"
                    sub_properties="grid-template-rows grid-template-columns grid-template-areas"
                    spec="https://drafts.csswg.org/css-grid/#propdef-grid-template"
                    disable_when_testing="True"
                    products="gecko">
    use cssparser::serialize_string;
    use parser::Parse;
    use properties::longhands::grid_template_rows;
    use properties::longhands::grid_template_areas::TemplateAreas;
    use values::{Either, None_};
    use values::generics::grid::{TrackSize, TrackList, TrackListType, concat_serialize_idents};
    use values::specified::TrackListOrNone;
    use values::specified::grid::parse_line_names;

    /// Parsing for `<grid-template>` shorthand (also used by `grid` shorthand).
    pub fn parse_grid_template(context: &ParserContext, input: &mut Parser)
                               -> Result<(TrackListOrNone, TrackListOrNone, Either<TemplateAreas, None_>), ()> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok((Either::Second(None_), Either::Second(None_), Either::Second(None_)))
        }

        let first_line_names = input.try(parse_line_names).unwrap_or(vec![]);
        if let Ok(s) = input.try(Parser::expect_string) {
            let mut strings = vec![];
            let mut values = vec![];
            let mut line_names = vec![];
            let mut names = first_line_names;
            let mut string = s.into_owned().into_boxed_str();
            loop {
                line_names.push(names);
                strings.push(string);
                let size = input.try(|i| TrackSize::parse(context, i)).unwrap_or_default();
                values.push(Either::First(size));
                names = input.try(parse_line_names).unwrap_or(vec![]);
                if let Ok(v) = input.try(parse_line_names) {
                    names.extend(v);
                }

                string = match input.try(Parser::expect_string) {
                    Ok(s) => s.into_owned().into_boxed_str(),
                    _ => {      // only the named area determines whether we should bail out
                        line_names.push(names);
                        break
                    },
                };
            }

            if line_names.len() == values.len() {
                line_names.push(vec![]);        // should be one longer than track sizes
            }

            let template_areas = TemplateAreas::from_vec(strings)?;
            let template_rows = TrackList {
                list_type: TrackListType::Normal,
                values: values,
                line_names: line_names,
                auto_repeat: None,
            };

            let template_cols = if input.try(|i| i.expect_delim('/')).is_ok() {
                let track_list = TrackList::parse(context, input)?;
                if track_list.list_type != TrackListType::Explicit {
                    return Err(())
                }

                Either::First(track_list)
            } else {
                Either::Second(None_)
            };

            Ok((Either::First(template_rows), template_cols, Either::First(template_areas)))
        } else {
            let mut template_rows = grid_template_rows::parse(context, input)?;
            if let Either::First(ref mut list) = template_rows {
                list.line_names[0] = first_line_names;      // won't panic
            }

            Ok((template_rows, grid_template_rows::parse(context, input)?, Either::Second(None_)))
        }
    }

    #[inline]
    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let (rows, columns, areas) = parse_grid_template(context, input)?;
        Ok(expanded! {
            grid_template_rows: rows,
            grid_template_columns: columns,
            grid_template_areas: areas,
        })
    }

    /// Serialization for `<grid-template>` shorthand (also used by `grid` shorthand).
    pub fn serialize_grid_template<W>(template_rows: &TrackListOrNone,
                                      template_columns: &TrackListOrNone,
                                      template_areas: &Either<TemplateAreas, None_>,
                                      dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *template_areas {
            Either::Second(_none) => {
                if template_rows == &Either::Second(None_) &&
                   template_columns == &Either::Second(None_) {
                    dest.write_str("none")
                } else {
                    template_rows.to_css(dest)?;
                    dest.write_str(" / ")?;
                    template_columns.to_css(dest)
                }
            },
            Either::First(ref areas) => {
                let track_list = match *template_rows {
                    Either::First(ref list) => list,
                    Either::Second(_none) => unreachable!(),    // should exist!
                };

                let mut names_iter = track_list.line_names.iter();
                for (((i, string), names), size) in areas.strings.iter().enumerate()
                                                                 .zip(&mut names_iter)
                                                                 .zip(track_list.values.iter()) {
                    if i > 0 {
                        dest.write_str(" ")?;
                    }

                    if !names.is_empty() {
                        concat_serialize_idents("[", "] ", names, " ", dest)?;
                    }

                    serialize_string(string, dest)?;
                    dest.write_str(" ")?;
                    size.to_css(dest)?;
                }

                if let Some(names) = names_iter.next() {
                    concat_serialize_idents(" [", "]", names, " ", dest)?;
                }

                if let Either::First(ref list) = *template_columns {
                    dest.write_str(" / ")?;
                    list.to_css(dest)?;
                }

                Ok(())
            },
        }
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        #[inline]
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            serialize_grid_template(self.grid_template_rows, self.grid_template_columns,
                                    self.grid_template_areas, dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="grid"
                    sub_properties="grid-template-rows grid-template-columns grid-template-areas
                                    grid-auto-rows grid-auto-columns grid-row-gap grid-column-gap
                                    grid-auto-flow"
                    spec="https://drafts.csswg.org/css-grid/#propdef-grid"
                    disable_when_testing="True"
                    products="gecko">
    use properties::longhands::{grid_auto_columns, grid_auto_rows, grid_auto_flow};
    use properties::longhands::{grid_template_columns, grid_template_rows};
    use properties::longhands::grid_auto_flow::computed_value::{AutoFlow, T as SpecifiedAutoFlow};
    use values::{Either, None_};
    use values::specified::{LengthOrPercentage, TrackSize};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let mut temp_rows = Either::Second(None_);
        let mut temp_cols = Either::Second(None_);
        let mut temp_areas = Either::Second(None_);
        let mut auto_rows = TrackSize::default();
        let mut auto_cols = TrackSize::default();
        let mut flow = grid_auto_flow::get_initial_value();

        fn parse_auto_flow(input: &mut Parser, is_row: bool) -> Result<SpecifiedAutoFlow, ()> {
            let mut auto_flow = None;
            let mut dense = false;
            for _ in 0..2 {
                if input.try(|i| i.expect_ident_matching("auto-flow")).is_ok() {
                    auto_flow = if is_row {
                        Some(AutoFlow::Row)
                    } else {
                        Some(AutoFlow::Column)
                    };
                } else if input.try(|i| i.expect_ident_matching("dense")).is_ok() {
                    dense = true;
                } else {
                    break
                }
            }

            auto_flow.map(|flow| {
                SpecifiedAutoFlow {
                    autoflow: flow,
                    dense: dense,
                }
            }).ok_or(())
        }

        if let Ok((rows, cols, areas)) = input.try(|i| super::grid_template::parse_grid_template(context, i)) {
            temp_rows = rows;
            temp_cols = cols;
            temp_areas = areas;
        } else if let Ok(rows) = input.try(|i| grid_template_rows::parse(context, i)) {
            temp_rows = rows;
            input.expect_delim('/')?;
            flow = parse_auto_flow(input, false)?;
            auto_cols = grid_auto_columns::parse(context, input).unwrap_or_default();
        } else {
            flow = parse_auto_flow(input, true)?;
            auto_rows = input.try(|i| grid_auto_rows::parse(context, i)).unwrap_or_default();
            input.expect_delim('/')?;
            temp_cols = grid_template_columns::parse(context, input)?;
        }

        Ok(expanded! {
            grid_template_rows: temp_rows,
            grid_template_columns: temp_cols,
            grid_template_areas: temp_areas,
            grid_auto_rows: auto_rows,
            grid_auto_columns: auto_cols,
            grid_auto_flow: flow,
            // This shorthand also resets grid gap
            grid_row_gap: LengthOrPercentage::zero(),
            grid_column_gap: LengthOrPercentage::zero(),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let Either::First(_) = *self.grid_template_areas {
                super::grid_template::serialize_grid_template(self.grid_template_rows,
                                                              self.grid_template_columns,
                                                              self.grid_template_areas, dest)
            } else if self.grid_auto_flow.autoflow == AutoFlow::Row {
                self.grid_template_rows.to_css(dest)?;
                dest.write_str(" / auto-flow")?;
                if self.grid_auto_flow.dense {
                    dest.write_str(" dense")?;
                }

                self.grid_auto_columns.to_css(dest)
            } else {
                dest.write_str("auto-flow ")?;
                if self.grid_auto_flow.dense {
                    dest.write_str("dense ")?;
                }

                self.grid_auto_rows.to_css(dest)?;
                dest.write_str(" / ")?;
                self.grid_template_columns.to_css(dest)
            }
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="place-content" sub_properties="align-content justify-content"
                    spec="https://drafts.csswg.org/css-align/#propdef-place-content"
                    products="gecko" disable_when_testing="True">
    use properties::longhands::align_content;
    use properties::longhands::justify_content;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let align = align_content::parse(context, input)?;
        if align.has_extra_flags() {
            return Err(());
        }
        let justify = input.try(|input| justify_content::parse(context, input))
                           .unwrap_or(justify_content::SpecifiedValue::from(align));
        if justify.has_extra_flags() {
            return Err(());
        }

        Ok(expanded! {
            align_content: align,
            justify_content: justify,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.align_content.to_css(dest)?;
            if self.align_content != self.justify_content {
                dest.write_str(" ")?;
                self.justify_content.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="place-self" sub_properties="align-self justify-self"
                    spec="https://drafts.csswg.org/css-align/#place-self-property"
                    products="gecko" disable_when_testing="True">
    use values::specified::align::AlignJustifySelf;
    use parser::Parse;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let align = AlignJustifySelf::parse(context, input)?;
        if align.has_extra_flags() {
            return Err(());
        }
        let justify = input.try(|input| AlignJustifySelf::parse(context, input)).unwrap_or(align.clone());
        if justify.has_extra_flags() {
            return Err(());
        }

        Ok(expanded! {
            align_self: align,
            justify_self: justify,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.align_self == self.justify_self {
                self.align_self.to_css(dest)
            } else {
                self.align_self.to_css(dest)?;
                dest.write_str(" ")?;
                self.justify_self.to_css(dest)
            }
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="place-items" sub_properties="align-items justify-items"
                    spec="https://drafts.csswg.org/css-align/#place-items-property"
                    products="gecko" disable_when_testing="True">
    use values::specified::align::{AlignItems, JustifyItems};
    use parser::Parse;

    impl From<AlignItems> for JustifyItems {
        fn from(align: AlignItems) -> JustifyItems {
            JustifyItems(align.0)
        }
    }

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let align = AlignItems::parse(context, input)?;
        if align.has_extra_flags() {
            return Err(());
        }
        let justify = input.try(|input| JustifyItems::parse(context, input))
                           .unwrap_or(JustifyItems::from(align));
        if justify.has_extra_flags() {
            return Err(());
        }

        Ok(expanded! {
            align_items: align,
            justify_items: justify,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.align_items.0 == self.justify_items.0 {
                self.align_items.to_css(dest)
            } else {
                self.align_items.to_css(dest)?;
                dest.write_str(" ")?;
                self.justify_items.to_css(dest)
            }
        }
    }
</%helpers:shorthand>

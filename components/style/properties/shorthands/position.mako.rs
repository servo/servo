/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="flex-flow"
                    engines="gecko servo",
                    servo_pref="layout.flexbox.enabled",
                    sub_properties="flex-direction flex-wrap"
                    extra_prefixes="webkit"
                    spec="https://drafts.csswg.org/css-flexbox/#flex-flow-property">
    use crate::properties::longhands::{flex_direction, flex_wrap};

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let mut direction = None;
        let mut wrap = None;
        loop {
            if direction.is_none() {
                if let Ok(value) = input.try_parse(|input| flex_direction::parse(context, input)) {
                    direction = Some(value);
                    continue
                }
            }
            if wrap.is_none() {
                if let Ok(value) = input.try_parse(|input| flex_wrap::parse(context, input)) {
                    wrap = Some(value);
                    continue
                }
            }
            break
        }

        if direction.is_none() && wrap.is_none() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
        Ok(expanded! {
            flex_direction: unwrap_or_initial!(flex_direction, direction),
            flex_wrap: unwrap_or_initial!(flex_wrap, wrap),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            if *self.flex_direction == flex_direction::get_initial_specified_value() &&
               *self.flex_wrap != flex_wrap::get_initial_specified_value() {
                return self.flex_wrap.to_css(dest)
            }
            self.flex_direction.to_css(dest)?;
            if *self.flex_wrap != flex_wrap::get_initial_specified_value() {
                dest.write_char(' ')?;
                self.flex_wrap.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="flex"
                    engines="gecko servo",
                    servo_pref="layout.flexbox.enabled",
                    sub_properties="flex-grow flex-shrink flex-basis"
                    extra_prefixes="webkit"
                    derive_serialize="True"
                    spec="https://drafts.csswg.org/css-flexbox/#flex-property">
    use crate::parser::Parse;
    use crate::values::specified::NonNegativeNumber;
    use crate::properties::longhands::flex_basis::SpecifiedValue as FlexBasis;

    fn parse_flexibility<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<(NonNegativeNumber, Option<NonNegativeNumber>),ParseError<'i>> {
        let grow = NonNegativeNumber::parse(context, input)?;
        let shrink = input.try_parse(|i| NonNegativeNumber::parse(context, i)).ok();
        Ok((grow, shrink))
    }

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let mut grow = None;
        let mut shrink = None;
        let mut basis = None;

        if input.try_parse(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(expanded! {
                flex_grow: NonNegativeNumber::new(0.0),
                flex_shrink: NonNegativeNumber::new(0.0),
                flex_basis: FlexBasis::auto(),
            })
        }
        loop {
            if grow.is_none() {
                if let Ok((flex_grow, flex_shrink)) = input.try_parse(|i| parse_flexibility(context, i)) {
                    grow = Some(flex_grow);
                    shrink = flex_shrink;
                    continue
                }
            }
            if basis.is_none() {
                if let Ok(value) = input.try_parse(|input| FlexBasis::parse(context, input)) {
                    basis = Some(value);
                    continue
                }
            }
            break
        }

        if grow.is_none() && basis.is_none() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
        Ok(expanded! {
            flex_grow: grow.unwrap_or(NonNegativeNumber::new(1.0)),
            flex_shrink: shrink.unwrap_or(NonNegativeNumber::new(1.0)),
            // Per spec, this should be SpecifiedValue::zero(), but all
            // browsers currently agree on using `0%`. This is a spec
            // change which hasn't been adopted by browsers:
            // https://github.com/w3c/csswg-drafts/commit/2c446befdf0f686217905bdd7c92409f6bd3921b
            flex_basis: basis.unwrap_or(FlexBasis::zero_percent()),
        })
    }
</%helpers:shorthand>

<%helpers:shorthand
    name="gap"
    engines="gecko"
    aliases="grid-gap"
    sub_properties="row-gap column-gap"
    spec="https://drafts.csswg.org/css-align-3/#gap-shorthand"
>
  use crate::properties::longhands::{row_gap, column_gap};

  pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<Longhands, ParseError<'i>> {
      let r_gap = row_gap::parse(context, input)?;
      let c_gap = input.try_parse(|input| column_gap::parse(context, input)).unwrap_or(r_gap.clone());

      Ok(expanded! {
        row_gap: r_gap,
        column_gap: c_gap,
      })
  }

  impl<'a> ToCss for LonghandsToSerialize<'a>  {
      fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
          if self.row_gap == self.column_gap {
            self.row_gap.to_css(dest)
          } else {
            self.row_gap.to_css(dest)?;
            dest.write_char(' ')?;
            self.column_gap.to_css(dest)
          }
      }
  }

</%helpers:shorthand>

% for kind in ["row", "column"]:
<%helpers:shorthand
    name="grid-${kind}"
    sub_properties="grid-${kind}-start grid-${kind}-end"
    engines="gecko",
    spec="https://drafts.csswg.org/css-grid/#propdef-grid-${kind}"
>
    use crate::values::specified::GridLine;
    use crate::parser::Parse;
    use crate::Zero;

    // NOTE: Since both the shorthands have the same code, we should (re-)use code from one to implement
    // the other. This might not be a big deal for now, but we should consider looking into this in the future
    // to limit the amount of code generated.
    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let start = input.try_parse(|i| GridLine::parse(context, i))?;
        let end = if input.try_parse(|i| i.expect_delim('/')).is_ok() {
            GridLine::parse(context, input)?
        } else {
            let mut line = GridLine::auto();
            if start.line_num.is_zero() && !start.is_span {
                line.ident = start.ident.clone(); // ident from start value should be taken
            }

            line
        };

        Ok(expanded! {
            grid_${kind}_start: start,
            grid_${kind}_end: end,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        // Return the shortest possible serialization of the `grid-${kind}-[start/end]` values.
        // This function exploits the opportunities to omit the end value per this spec text:
        //
        // https://drafts.csswg.org/css-grid/#propdef-grid-column
        // "When the second value is omitted, if the first value is a <custom-ident>,
        // the grid-row-end/grid-column-end longhand is also set to that <custom-ident>;
        // otherwise, it is set to auto."
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.grid_${kind}_start.to_css(dest)?;
            if self.grid_${kind}_start.can_omit(self.grid_${kind}_end) {
                return Ok(());  // the end value is redundant
            }
            dest.write_str(" / ")?;
            self.grid_${kind}_end.to_css(dest)
        }
    }
</%helpers:shorthand>
% endfor

<%helpers:shorthand
    name="grid-area"
    engines="gecko"
    sub_properties="grid-row-start grid-row-end grid-column-start grid-column-end"
    spec="https://drafts.csswg.org/css-grid/#propdef-grid-area"
>
    use crate::values::specified::GridLine;
    use crate::parser::Parse;
    use crate::Zero;

    // The code is the same as `grid-{row,column}` except that this can have four values at most.
    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        fn line_with_ident_from(other: &GridLine) -> GridLine {
            let mut this = GridLine::auto();
            if other.line_num.is_zero() && !other.is_span {
                this.ident = other.ident.clone();
            }

            this
        }

        let row_start = input.try_parse(|i| GridLine::parse(context, i))?;
        let (column_start, row_end, column_end) = if input.try_parse(|i| i.expect_delim('/')).is_ok() {
            let column_start = GridLine::parse(context, input)?;
            let (row_end, column_end) = if input.try_parse(|i| i.expect_delim('/')).is_ok() {
                let row_end = GridLine::parse(context, input)?;
                let column_end = if input.try_parse(|i| i.expect_delim('/')).is_ok() {
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
        // Return the shortest possible serialization of the `grid-[column/row]-[start/end]` values.
        // This function exploits the opportunities to omit trailing values per this spec text:
        //
        // https://drafts.csswg.org/css-grid/#propdef-grid-area
        // "If four <grid-line> values are specified, grid-row-start is set to the first value,
        // grid-column-start is set to the second value, grid-row-end is set to the third value,
        // and grid-column-end is set to the fourth value.
        //
        // When grid-column-end is omitted, if grid-column-start is a <custom-ident>,
        // grid-column-end is set to that <custom-ident>; otherwise, it is set to auto.
        //
        // When grid-row-end is omitted, if grid-row-start is a <custom-ident>, grid-row-end is
        // set to that <custom-ident>; otherwise, it is set to auto.
        //
        // When grid-column-start is omitted, if grid-row-start is a <custom-ident>, all four
        // longhands are set to that value. Otherwise, it is set to auto."
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.grid_row_start.to_css(dest)?;
            let mut trailing_values = 3;
            if self.grid_column_start.can_omit(self.grid_column_end) {
                trailing_values -= 1;
                if self.grid_row_start.can_omit(self.grid_row_end) {
                    trailing_values -= 1;
                    if self.grid_row_start.can_omit(self.grid_column_start) {
                        trailing_values -= 1;
                    }
                }
            }
            let values = [&self.grid_column_start, &self.grid_row_end, &self.grid_column_end];
            for value in values.iter().take(trailing_values) {
                dest.write_str(" / ")?;
                value.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    name="grid-template"
    engines="gecko"
    sub_properties="grid-template-rows grid-template-columns grid-template-areas"
    spec="https://drafts.csswg.org/css-grid/#propdef-grid-template"
>
    use crate::parser::Parse;
    use servo_arc::Arc;
    use crate::values::generics::grid::{TrackSize, TrackList};
    use crate::values::generics::grid::{TrackListValue, concat_serialize_idents};
    use crate::values::specified::{GridTemplateComponent, GenericGridTemplateComponent};
    use crate::values::specified::grid::parse_line_names;
    use crate::values::specified::position::{GridTemplateAreas, TemplateAreas, TemplateAreasArc};

    /// Parsing for `<grid-template>` shorthand (also used by `grid` shorthand).
    pub fn parse_grid_template<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<(GridTemplateComponent, GridTemplateComponent, GridTemplateAreas), ParseError<'i>> {
        // Other shorthand sub properties also parse the `none` keyword and this shorthand
        // should know after this keyword there is nothing to parse. Otherwise it gets
        // confused and rejects the sub properties that contains `none`.
        <% keywords = {
            "none": "GenericGridTemplateComponent::None",
        }
        %>
        % for keyword, rust_type in keywords.items():
            if let Ok(x) = input.try_parse(|i| {
                if i.try_parse(|i| i.expect_ident_matching("${keyword}")).is_ok() {
                    if !i.is_exhausted() {
                        return Err(());
                    }
                    return Ok((${rust_type}, ${rust_type}, GridTemplateAreas::None));
                }
                Err(())
            }) {
                return Ok(x);
            }
        % endfor

        let first_line_names = input.try_parse(parse_line_names).unwrap_or_default();
        if let Ok(string) = input.try_parse(|i| i.expect_string().map(|s| s.as_ref().to_owned().into())) {
            let mut strings = vec![];
            let mut values = vec![];
            let mut line_names = vec![];
            line_names.push(first_line_names);
            strings.push(string);
            loop {
                let size = input.try_parse(|i| TrackSize::parse(context, i)).unwrap_or_default();
                values.push(TrackListValue::TrackSize(size));
                let mut names = input.try_parse(parse_line_names).unwrap_or_default();
                let more_names = input.try_parse(parse_line_names);

                match input.try_parse(|i| i.expect_string().map(|s| s.as_ref().to_owned().into())) {
                    Ok(string) => {
                        strings.push(string);
                        if let Ok(v) = more_names {
                            // We got `[names] [more_names] "string"` - merge the two name lists.
                            let mut names_vec = names.into_vec();
                            names_vec.extend(v.into_iter());
                            names = names_vec.into();
                        }
                        line_names.push(names);
                    },
                    Err(e) => {
                        if more_names.is_ok() {
                            // We've parsed `"string" [names] [more_names]` but then failed to parse another `"string"`.
                            // The grammar doesn't allow two trailing `<line-names>` so this is an invalid value.
                            return Err(e.into());
                        }
                        // only the named area determines whether we should bail out
                        line_names.push(names);
                        break
                    },
                };
            }

            if line_names.len() == values.len() {
                // should be one longer than track sizes
                line_names.push(Default::default());
            }

            let template_areas = TemplateAreas::from_vec(strings)
                .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))?;
            let template_rows = TrackList {
                values: values.into(),
                line_names: line_names.into(),
                auto_repeat_index: std::usize::MAX,
            };

            let template_cols = if input.try_parse(|i| i.expect_delim('/')).is_ok() {
                let value = GridTemplateComponent::parse_without_none(context, input)?;
                if let GenericGridTemplateComponent::TrackList(ref list) = value {
                    if !list.is_explicit() {
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    }
                }

                value
            } else {
                GridTemplateComponent::default()
            };

            Ok((
                GenericGridTemplateComponent::TrackList(Box::new(template_rows)),
                template_cols,
                GridTemplateAreas::Areas(TemplateAreasArc(Arc::new(template_areas)))
            ))
        } else {
            let mut template_rows = GridTemplateComponent::parse(context, input)?;
            if let GenericGridTemplateComponent::TrackList(ref mut list) = template_rows {
                // Fist line names are parsed already and it shouldn't be parsed again.
                // If line names are not empty, that means given property value is not acceptable
                if list.line_names[0].is_empty() {
                    list.line_names[0] = first_line_names;      // won't panic
                } else {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
            }

            input.expect_delim('/')?;
            Ok((template_rows, GridTemplateComponent::parse(context, input)?, GridTemplateAreas::None))
        }
    }

    #[inline]
    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let (rows, columns, areas) = parse_grid_template(context, input)?;
        Ok(expanded! {
            grid_template_rows: rows,
            grid_template_columns: columns,
            grid_template_areas: areas,
        })
    }

    /// Serialization for `<grid-template>` shorthand (also used by `grid` shorthand).
    pub fn serialize_grid_template<W>(
        template_rows: &GridTemplateComponent,
        template_columns: &GridTemplateComponent,
        template_areas: &GridTemplateAreas,
        dest: &mut CssWriter<W>,
    ) -> fmt::Result
    where
        W: Write {
        match *template_areas {
            GridTemplateAreas::None => {
                if template_rows.is_initial() && template_columns.is_initial() {
                    return GridTemplateComponent::default().to_css(dest);
                }
                template_rows.to_css(dest)?;
                dest.write_str(" / ")?;
                template_columns.to_css(dest)
            },
            GridTemplateAreas::Areas(ref areas) => {
                // The length of template-area and template-rows values should be equal.
                if areas.0.strings.len() != template_rows.track_list_len() {
                    return Ok(());
                }

                let track_list = match *template_rows {
                    GenericGridTemplateComponent::TrackList(ref list) => {
                        // We should fail if there is a `repeat` function.
                        // `grid` and `grid-template` shorthands doesn't accept
                        // that. Only longhand accepts.
                        if !list.is_explicit() {
                            return Ok(());
                        }
                        list
                    },
                    // Others template components shouldn't exist with normal shorthand values.
                    // But if we need to serialize a group of longhand sub-properties for
                    // the shorthand, we should be able to return empty string instead of crashing.
                    _ => return Ok(()),
                };

                // We need to check some values that longhand accepts but shorthands don't.
                match *template_columns {
                    // We should fail if there is a `repeat` function. `grid` and
                    // `grid-template` shorthands doesn't accept that. Only longhand accepts that.
                    GenericGridTemplateComponent::TrackList(ref list) => {
                        if !list.is_explicit() {
                            return Ok(());
                        }
                    },
                    // Also the shorthands don't accept subgrids unlike longhand.
                    // We should fail without an error here.
                    GenericGridTemplateComponent::Subgrid(_) => {
                        return Ok(());
                    },
                    _ => {},
                }

                let mut names_iter = track_list.line_names.iter();
                for (((i, string), names), value) in areas.0.strings.iter().enumerate()
                                                                  .zip(&mut names_iter)
                                                                  .zip(track_list.values.iter()) {
                    if i > 0 {
                        dest.write_char(' ')?;
                    }

                    if !names.is_empty() {
                        concat_serialize_idents("[", "] ", names, " ", dest)?;
                    }

                    string.to_css(dest)?;

                    // If the track size is the initial value then it's redundant here.
                    if !value.is_initial() {
                        dest.write_char(' ')?;
                        value.to_css(dest)?;
                    }
                }

                if let Some(names) = names_iter.next() {
                    concat_serialize_idents(" [", "]", names, " ", dest)?;
                }

                if let GenericGridTemplateComponent::TrackList(ref list) = *template_columns {
                    dest.write_str(" / ")?;
                    list.to_css(dest)?;
                }

                Ok(())
            },
        }
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        #[inline]
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            serialize_grid_template(
                self.grid_template_rows,
                self.grid_template_columns,
                self.grid_template_areas,
                dest
            )
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    name="grid"
    engines="gecko"
    sub_properties="grid-template-rows grid-template-columns grid-template-areas
                    grid-auto-rows grid-auto-columns grid-auto-flow"
    spec="https://drafts.csswg.org/css-grid/#propdef-grid"
>
    use crate::parser::Parse;
    use crate::properties::longhands::{grid_auto_columns, grid_auto_rows, grid_auto_flow};
    use crate::values::generics::grid::GridTemplateComponent;
    use crate::values::specified::{GenericGridTemplateComponent, ImplicitGridTracks};
    use crate::values::specified::position::{GridAutoFlow, GridTemplateAreas};

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let mut temp_rows = GridTemplateComponent::default();
        let mut temp_cols = GridTemplateComponent::default();
        let mut temp_areas = GridTemplateAreas::None;
        let mut auto_rows = ImplicitGridTracks::default();
        let mut auto_cols = ImplicitGridTracks::default();
        let mut flow = grid_auto_flow::get_initial_value();

        fn parse_auto_flow<'i, 't>(
            input: &mut Parser<'i, 't>,
            is_row: bool,
        ) -> Result<GridAutoFlow, ParseError<'i>> {
            let mut track = None;
            let mut dense = GridAutoFlow::empty();

            for _ in 0..2 {
                if input.try_parse(|i| i.expect_ident_matching("auto-flow")).is_ok() {
                    track = if is_row {
                        Some(GridAutoFlow::ROW)
                    } else {
                        Some(GridAutoFlow::COLUMN)
                    };
                } else if input.try_parse(|i| i.expect_ident_matching("dense")).is_ok() {
                    dense = GridAutoFlow::DENSE
                } else {
                    break
                }
            }

            if track.is_some() {
                Ok(track.unwrap() | dense)
            } else {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            }
        }

        if let Ok((rows, cols, areas)) = input.try_parse(|i| super::grid_template::parse_grid_template(context, i)) {
            temp_rows = rows;
            temp_cols = cols;
            temp_areas = areas;
        } else if let Ok(rows) = input.try_parse(|i| GridTemplateComponent::parse(context, i)) {
            temp_rows = rows;
            input.expect_delim('/')?;
            flow = parse_auto_flow(input, false)?;
            auto_cols = input.try_parse(|i| grid_auto_columns::parse(context, i)).unwrap_or_default();
        } else {
            flow = parse_auto_flow(input, true)?;
            auto_rows = input.try_parse(|i| grid_auto_rows::parse(context, i)).unwrap_or_default();
            input.expect_delim('/')?;
            temp_cols = GridTemplateComponent::parse(context, input)?;
        }

        Ok(expanded! {
            grid_template_rows: temp_rows,
            grid_template_columns: temp_cols,
            grid_template_areas: temp_areas,
            grid_auto_rows: auto_rows,
            grid_auto_columns: auto_cols,
            grid_auto_flow: flow,
        })
    }

    impl<'a> LonghandsToSerialize<'a> {
        /// Returns true if other sub properties except template-{rows,columns} are initial.
        fn is_grid_template(&self) -> bool {
            self.grid_auto_rows.is_initial() &&
            self.grid_auto_columns.is_initial() &&
            *self.grid_auto_flow == grid_auto_flow::get_initial_value()
        }
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            if self.is_grid_template() {
                return super::grid_template::serialize_grid_template(
                    self.grid_template_rows,
                    self.grid_template_columns,
                    self.grid_template_areas,
                    dest
                );
            }

            if *self.grid_template_areas != GridTemplateAreas::None {
                // No other syntax can set the template areas, so fail to
                // serialize.
                return Ok(());
            }

            if self.grid_auto_flow.contains(GridAutoFlow::COLUMN) {
                // It should fail to serialize if other branch of the if condition's values are set.
                if !self.grid_auto_rows.is_initial() ||
                    !self.grid_template_columns.is_initial() {
                    return Ok(());
                }

                // It should fail to serialize if template-rows value is not Explicit.
                if let GenericGridTemplateComponent::TrackList(ref list) = *self.grid_template_rows {
                    if !list.is_explicit() {
                        return Ok(());
                    }
                }

                self.grid_template_rows.to_css(dest)?;
                dest.write_str(" / auto-flow")?;
                if self.grid_auto_flow.contains(GridAutoFlow::DENSE) {
                    dest.write_str(" dense")?;
                }

                if !self.grid_auto_columns.is_initial() {
                    dest.write_char(' ')?;
                    self.grid_auto_columns.to_css(dest)?;
                }

                return Ok(());
            }

            // It should fail to serialize if other branch of the if condition's values are set.
            if !self.grid_auto_columns.is_initial() ||
                !self.grid_template_rows.is_initial() {
                return Ok(());
            }

            // It should fail to serialize if template-column value is not Explicit.
            if let GenericGridTemplateComponent::TrackList(ref list) = *self.grid_template_columns {
                if !list.is_explicit() {
                    return Ok(());
                }
            }

            dest.write_str("auto-flow")?;
            if self.grid_auto_flow.contains(GridAutoFlow::DENSE) {
                dest.write_str(" dense")?;
            }

            if !self.grid_auto_rows.is_initial() {
                dest.write_char(' ')?;
                self.grid_auto_rows.to_css(dest)?;
            }

            dest.write_str(" / ")?;
            self.grid_template_columns.to_css(dest)?;
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    name="place-content"
    engines="gecko"
    sub_properties="align-content justify-content"
    spec="https://drafts.csswg.org/css-align/#propdef-place-content"
>
    use crate::values::specified::align::{AlignContent, JustifyContent, ContentDistribution, AxisDirection};

    pub fn parse_value<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let align_content =
            ContentDistribution::parse(input, AxisDirection::Block)?;

        let justify_content = input.try_parse(|input| {
            ContentDistribution::parse(input, AxisDirection::Inline)
        });

        let justify_content = match justify_content {
            Ok(v) => v,
            Err(..) => {
                // https://drafts.csswg.org/css-align-3/#place-content:
                //
                //   The second value is assigned to justify-content; if
                //   omitted, it is copied from the first value, unless that
                //   value is a <baseline-position> in which case it is
                //   defaulted to start.
                //
                if !align_content.is_baseline_position() {
                    align_content
                } else {
                    ContentDistribution::start()
                }
            }
        };

        Ok(expanded! {
            align_content: AlignContent(align_content),
            justify_content: JustifyContent(justify_content),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.align_content.to_css(dest)?;
            if self.align_content.0 != self.justify_content.0 {
                dest.write_char(' ')?;
                self.justify_content.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    name="place-self"
    engines="gecko"
    sub_properties="align-self justify-self"
    spec="https://drafts.csswg.org/css-align/#place-self-property"
>
    use crate::values::specified::align::{AlignSelf, JustifySelf, SelfAlignment, AxisDirection};

    pub fn parse_value<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let align = SelfAlignment::parse(input, AxisDirection::Block)?;
        let justify = input.try_parse(|input| SelfAlignment::parse(input, AxisDirection::Inline));

        let justify = match justify {
            Ok(v) => v,
            Err(..) => {
                debug_assert!(align.is_valid_on_both_axes());
                align
            }
        };

        Ok(expanded! {
            align_self: AlignSelf(align),
            justify_self: JustifySelf(justify),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.align_self.to_css(dest)?;
            if self.align_self.0 != self.justify_self.0 {
                dest.write_char(' ')?;
                self.justify_self.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand
    name="place-items"
    engines="gecko"
    sub_properties="align-items justify-items"
    spec="https://drafts.csswg.org/css-align/#place-items-property"
>
    use crate::values::specified::align::{AlignItems, JustifyItems};
    use crate::parser::Parse;

    impl From<AlignItems> for JustifyItems {
        fn from(align: AlignItems) -> JustifyItems {
            JustifyItems(align.0)
        }
    }

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let align = AlignItems::parse(context, input)?;
        let justify =
            input.try_parse(|input| JustifyItems::parse(context, input))
                 .unwrap_or_else(|_| JustifyItems::from(align));

        Ok(expanded! {
            align_items: align,
            justify_items: justify,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.align_items.to_css(dest)?;
            if self.align_items.0 != self.justify_items.0 {
                dest.write_char(' ')?;
                self.justify_items.to_css(dest)?;
            }

            Ok(())
        }
    }
</%helpers:shorthand>

// See https://github.com/w3c/csswg-drafts/issues/3525 for the quirks stuff.
${helpers.four_sides_shorthand(
    "inset",
    "%s",
    "specified::LengthPercentageOrAuto::parse",
    engines="gecko servo",
    spec="https://drafts.csswg.org/css-logical/#propdef-inset",
    allow_quirks="No",
)}

${helpers.two_properties_shorthand(
    "inset-block",
    "inset-block-start",
    "inset-block-end",
    "specified::LengthPercentageOrAuto::parse",
    engines="gecko servo",
    spec="https://drafts.csswg.org/css-logical/#propdef-inset-block"
)}

${helpers.two_properties_shorthand(
    "inset-inline",
    "inset-inline-start",
    "inset-inline-end",
    "specified::LengthPercentageOrAuto::parse",
    engines="gecko servo",
    spec="https://drafts.csswg.org/css-logical/#propdef-inset-inline"
)}

${helpers.two_properties_shorthand(
    "contain-intrinsic-size",
    "contain-intrinsic-width",
    "contain-intrinsic-height",
    engines="gecko",
    gecko_pref="layout.css.contain-intrinsic-size.enabled",
    spec="https://drafts.csswg.org/css-sizing-4/#intrinsic-size-override",
)}

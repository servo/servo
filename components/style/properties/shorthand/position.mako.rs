/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="flex-flow"
                    sub_properties="flex-direction flex-wrap"
                    extra_prefixes="webkit"
                    derive_serialize="True"
                    spec="https://drafts.csswg.org/css-flexbox/#flex-flow-property">
    use properties::longhands::{flex_direction, flex_wrap};

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
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
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
        Ok(expanded! {
            flex_direction: unwrap_or_initial!(flex_direction, direction),
            flex_wrap: unwrap_or_initial!(flex_wrap, wrap),
        })
    }
</%helpers:shorthand>

<%helpers:shorthand name="flex"
                    sub_properties="flex-grow flex-shrink flex-basis"
                    extra_prefixes="webkit"
                    derive_serialize="True"
                    spec="https://drafts.csswg.org/css-flexbox/#flex-property">
    use parser::Parse;
    use values::specified::NonNegativeNumber;
    use properties::longhands::flex_basis::SpecifiedValue as FlexBasis;

    fn parse_flexibility<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                 -> Result<(NonNegativeNumber, Option<NonNegativeNumber>),ParseError<'i>> {
        let grow = NonNegativeNumber::parse(context, input)?;
        let shrink = input.try(|i| NonNegativeNumber::parse(context, i)).ok();
        Ok((grow, shrink))
    }

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        let mut grow = None;
        let mut shrink = None;
        let mut basis = None;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(expanded! {
                flex_grow: NonNegativeNumber::new(0.0),
                flex_shrink: NonNegativeNumber::new(0.0),
                flex_basis: FlexBasis::auto(),
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
                if let Ok(value) = input.try(|input| FlexBasis::parse(context, input)) {
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

<%helpers:shorthand name="grid-gap" sub_properties="grid-row-gap grid-column-gap"
                    spec="https://drafts.csswg.org/css-grid/#propdef-grid-gap"
                    products="gecko">
  use properties::longhands::{grid_row_gap, grid_column_gap};

  pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<Longhands, ParseError<'i>> {
      let row_gap = grid_row_gap::parse(context, input)?;
      let column_gap = input.try(|input| grid_column_gap::parse(context, input)).unwrap_or(row_gap.clone());

      Ok(expanded! {
        grid_row_gap: row_gap,
        grid_column_gap: column_gap,
      })
  }

  impl<'a> ToCss for LonghandsToSerialize<'a>  {
      fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
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
    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        let start = input.try(|i| GridLine::parse(context, i))?;
        let end = if input.try(|i| i.expect_delim('/')).is_ok() {
            GridLine::parse(context, input)?
        } else {
            let mut line = GridLine::auto();
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
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
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
    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        fn line_with_ident_from(other: &GridLine) -> GridLine {
            let mut this = GridLine::auto();
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
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
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
                    products="gecko">
    use parser::Parse;
    use values::{Either, None_};
    use values::generics::grid::{LineNameList, TrackSize, TrackList, TrackListType};
    use values::generics::grid::{TrackListValue, concat_serialize_idents};
    use values::specified::{GridTemplateComponent, GenericGridTemplateComponent};
    use values::specified::grid::parse_line_names;
    use values::specified::position::TemplateAreas;

    /// Parsing for `<grid-template>` shorthand (also used by `grid` shorthand).
    pub fn parse_grid_template<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                       -> Result<(GridTemplateComponent,
                                                  GridTemplateComponent,
                                                  Either<TemplateAreas, None_>), ParseError<'i>> {
        // Other shorthand sub properties also parse `none` and `subgrid` keywords and this
        // shorthand should know after these keywords there is nothing to parse. Otherwise it
        // gets confused and rejects the sub properties that contains `none` or `subgrid`.
        <% keywords = {
            "none": "GenericGridTemplateComponent::None",
            "subgrid": "GenericGridTemplateComponent::Subgrid(LineNameList::default())"
        }
        %>
        % for keyword, rust_type in keywords.items():
            if let Ok(x) = input.try(|i| {
                if i.try(|i| i.expect_ident_matching("${keyword}")).is_ok() {
                    if i.is_exhausted() {
                        return Ok((${rust_type},
                                   ${rust_type},
                                   Either::Second(None_)))
                    } else {
                        return Err(());
                    }
                }
                Err(())
            }) {
                return Ok(x);
            }
        % endfor

        let first_line_names = input.try(parse_line_names).unwrap_or(vec![].into_boxed_slice());
        if let Ok(mut string) = input.try(|i| i.expect_string().map(|s| s.as_ref().into())) {
            let mut strings = vec![];
            let mut values = vec![];
            let mut line_names = vec![];
            let mut names = first_line_names.into_vec();
            loop {
                line_names.push(names.into_boxed_slice());
                strings.push(string);
                let size = input.try(|i| TrackSize::parse(context, i)).unwrap_or_default();
                values.push(TrackListValue::TrackSize(size));
                names = input.try(parse_line_names).unwrap_or(vec![].into_boxed_slice()).into_vec();
                if let Ok(v) = input.try(parse_line_names) {
                    names.extend(v.into_vec());
                }

                string = match input.try(|i| i.expect_string().map(|s| s.as_ref().into())) {
                    Ok(s) => s,
                    _ => {      // only the named area determines whether we should bail out
                        line_names.push(names.into_boxed_slice());
                        break
                    },
                };
            }

            if line_names.len() == values.len() {
                // should be one longer than track sizes
                line_names.push(vec![].into_boxed_slice());
            }

            let template_areas = TemplateAreas::from_vec(strings)
                .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))?;
            let template_rows = TrackList {
                list_type: TrackListType::Normal,
                values: values,
                line_names: line_names.into_boxed_slice(),
                auto_repeat: None,
            };

            let template_cols = if input.try(|i| i.expect_delim('/')).is_ok() {
                let value = GridTemplateComponent::parse_without_none(context, input)?;
                if let GenericGridTemplateComponent::TrackList(ref list) = value {
                    if list.list_type != TrackListType::Explicit {
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    }
                }

                value
            } else {
                GenericGridTemplateComponent::None
            };

            Ok((GenericGridTemplateComponent::TrackList(template_rows),
                template_cols, Either::First(template_areas)))
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
            Ok((template_rows, GridTemplateComponent::parse(context, input)?, Either::Second(None_)))
        }
    }

    #[inline]
    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
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
        template_areas: &Either<TemplateAreas, None_>,
        dest: &mut CssWriter<W>,
    ) -> fmt::Result
    where
        W: Write {
        match *template_areas {
            Either::Second(_none) => {
                template_rows.to_css(dest)?;
                dest.write_str(" / ")?;
                template_columns.to_css(dest)
            },
            Either::First(ref areas) => {
                // The length of template-area and template-rows values should be equal.
                if areas.strings.len() != template_rows.track_list_len() {
                    return Ok(());
                }

                let track_list = match *template_rows {
                    GenericGridTemplateComponent::TrackList(ref list) => {
                        // We should fail if there is a `repeat` function. `grid` and
                        // `grid-template` shorthands doesn't accept that. Only longhand accepts.
                        if list.auto_repeat.is_some() ||
                           list.values.iter().any(|v| match *v {
                               TrackListValue::TrackRepeat(_) => true,
                               _ => false,
                           }) {
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
                        if list.auto_repeat.is_some() ||
                           list.values.iter().any(|v| match *v {
                               TrackListValue::TrackRepeat(_) => true,
                               _ => false,
                           }) {
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
                for (((i, string), names), value) in areas.strings.iter().enumerate()
                                                                  .zip(&mut names_iter)
                                                                  .zip(track_list.values.iter()) {
                    if i > 0 {
                        dest.write_str(" ")?;
                    }

                    if !names.is_empty() {
                        concat_serialize_idents("[", "] ", names, " ", dest)?;
                    }

                    string.to_css(dest)?;
                    dest.write_str(" ")?;
                    value.to_css(dest)?;
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
            serialize_grid_template(self.grid_template_rows, self.grid_template_columns,
                                    self.grid_template_areas, dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="grid"
                    sub_properties="grid-template-rows grid-template-columns grid-template-areas
                                    grid-auto-rows grid-auto-columns grid-auto-flow"
                    spec="https://drafts.csswg.org/css-grid/#propdef-grid"
                    products="gecko">
    use parser::Parse;
    use properties::longhands::{grid_auto_columns, grid_auto_rows, grid_auto_flow};
    use values::{Either, None_};
    use values::generics::grid::{GridTemplateComponent, TrackListType};
    use values::specified::{GenericGridTemplateComponent, TrackSize};
    use values::specified::position::{AutoFlow, GridAutoFlow};

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        let mut temp_rows = GridTemplateComponent::None;
        let mut temp_cols = GridTemplateComponent::None;
        let mut temp_areas = Either::Second(None_);
        let mut auto_rows = TrackSize::default();
        let mut auto_cols = TrackSize::default();
        let mut flow = grid_auto_flow::get_initial_value();

        fn parse_auto_flow<'i, 't>(input: &mut Parser<'i, 't>, is_row: bool)
                                   -> Result<GridAutoFlow, ParseError<'i>> {
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
                GridAutoFlow {
                    autoflow: flow,
                    dense: dense,
                }
            }).ok_or(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }

        if let Ok((rows, cols, areas)) = input.try(|i| super::grid_template::parse_grid_template(context, i)) {
            temp_rows = rows;
            temp_cols = cols;
            temp_areas = areas;
        } else if let Ok(rows) = input.try(|i| GridTemplateComponent::parse(context, i)) {
            temp_rows = rows;
            input.expect_delim('/')?;
            flow = parse_auto_flow(input, false)?;
            auto_cols = grid_auto_columns::parse(context, input).unwrap_or_default();
        } else {
            flow = parse_auto_flow(input, true)?;
            auto_rows = input.try(|i| grid_auto_rows::parse(context, i)).unwrap_or_default();
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
            *self.grid_template_areas == Either::Second(None_) &&
            *self.grid_auto_rows == TrackSize::default() &&
            *self.grid_auto_columns == TrackSize::default() &&
            *self.grid_auto_flow == grid_auto_flow::get_initial_value()
        }
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            if *self.grid_template_areas != Either::Second(None_) ||
               (*self.grid_template_rows != GridTemplateComponent::None &&
                   *self.grid_template_columns != GridTemplateComponent::None) ||
               self.is_grid_template() {
                return super::grid_template::serialize_grid_template(self.grid_template_rows,
                                                                     self.grid_template_columns,
                                                                     self.grid_template_areas, dest);
            }

            if self.grid_auto_flow.autoflow == AutoFlow::Column {
                // It should fail to serialize if other branch of the if condition's values are set.
                if *self.grid_auto_rows != TrackSize::default() ||
                   *self.grid_template_columns != GridTemplateComponent::None {
                    return Ok(());
                }

                // It should fail to serialize if template-rows value is not Explicit.
                if let GenericGridTemplateComponent::TrackList(ref list) = *self.grid_template_rows {
                    if list.list_type != TrackListType::Explicit {
                        return Ok(());
                    }
                }

                self.grid_template_rows.to_css(dest)?;
                dest.write_str(" / auto-flow")?;
                if self.grid_auto_flow.dense {
                    dest.write_str(" dense")?;
                }

                if !self.grid_auto_columns.is_default() {
                    dest.write_str(" ")?;
                    self.grid_auto_columns.to_css(dest)?;
                }
            } else {
                // It should fail to serialize if other branch of the if condition's values are set.
                if *self.grid_auto_columns != TrackSize::default() ||
                   *self.grid_template_rows != GridTemplateComponent::None {
                    return Ok(());
                }

                // It should fail to serialize if template-column value is not Explicit.
                if let GenericGridTemplateComponent::TrackList(ref list) = *self.grid_template_columns {
                    if list.list_type != TrackListType::Explicit {
                        return Ok(());
                    }
                }

                dest.write_str("auto-flow")?;
                if self.grid_auto_flow.dense {
                    dest.write_str(" dense")?;
                }

                if !self.grid_auto_rows.is_default() {
                    dest.write_str(" ")?;
                    self.grid_auto_rows.to_css(dest)?;
                }

                dest.write_str(" / ")?;
                self.grid_template_columns.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="place-content" sub_properties="align-content justify-content"
                    spec="https://drafts.csswg.org/css-align/#propdef-place-content"
                    products="gecko">
    use values::specified::align::{AlignJustifyContent, FallbackAllowed};

    pub fn parse_value<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let align = AlignJustifyContent::parse_with_fallback(input, FallbackAllowed::No)?;
        if align.has_extra_flags() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        let justify =
            input.try(|input| AlignJustifyContent::parse_with_fallback(input, FallbackAllowed::No))
                .unwrap_or(align);
        if justify.has_extra_flags() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(expanded! {
            align_content: align,
            justify_content: justify,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
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
                    products="gecko">
    use values::specified::align::AlignJustifySelf;
    use parser::Parse;

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        let align = AlignJustifySelf::parse(context, input)?;
        if align.has_extra_flags() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        let justify = input.try(|input| AlignJustifySelf::parse(context, input)).unwrap_or(align.clone());
        if justify.has_extra_flags() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(expanded! {
            align_self: align,
            justify_self: justify,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
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
                    products="gecko">
    use values::specified::align::{AlignItems, JustifyItems};
    use parser::Parse;

    impl From<AlignItems> for JustifyItems {
        fn from(align: AlignItems) -> JustifyItems {
            JustifyItems(align.0)
        }
    }

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        let align = AlignItems::parse(context, input)?;
        if align.has_extra_flags() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        let justify = input.try(|input| JustifyItems::parse(context, input))
                           .unwrap_or(JustifyItems::from(align));
        if justify.has_extra_flags() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(expanded! {
            align_items: align,
            justify_items: justify,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a> {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
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

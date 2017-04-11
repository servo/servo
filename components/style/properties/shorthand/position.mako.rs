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
        Ok(Longhands {
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
            return Ok(Longhands {
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
                if let Ok(value) = input.try(|input| longhands::flex_basis::parse(context, input)) {
                    basis = Some(value);
                    continue
                }
            }
            break
        }

        if grow.is_none() && basis.is_none() {
            return Err(())
        }
        Ok(Longhands {
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

      Ok(Longhands {
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

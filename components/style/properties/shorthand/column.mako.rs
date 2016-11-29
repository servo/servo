/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="columns" sub_properties="column-count column-width" experimental="True">
    use properties::longhands::{column_count, column_width};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {

        let mut column_count = None;
        let mut column_width = None;
        let mut autos = 0;

        loop {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                // Leave the options to None, 'auto' is the initial value.
                autos += 1;
                continue
            }

            if column_count.is_none() {
                if let Ok(value) = input.try(|input| column_count::parse(context, input)) {
                    column_count = Some(value);
                    continue
                }
            }

            if column_width.is_none() {
                if let Ok(value) = input.try(|input| column_width::parse(context, input)) {
                    column_width = Some(value);
                    continue
                }
            }

            break
        }

        let values = autos + column_count.iter().len() + column_width.iter().len();
        if values == 0 || values > 2 {
            Err(())
        } else {
            Ok(Longhands {
                column_count: column_count,
                column_width: column_width,
            })
        }
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.column_width.to_css(dest));
            try!(write!(dest, " "));

            self.column_count.to_css(dest)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="-moz-column-rule" products="gecko"
    sub_properties="-moz-column-rule-width -moz-column-rule-style -moz-column-rule-color">
    use properties::longhands::{_moz_column_rule_width, _moz_column_rule_style};
    use properties::longhands::_moz_column_rule_color;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        % for name in "width style color".split():
        let mut column_rule_${name} = None;
        % endfor
        let mut any = false;

        loop {
            % for name in "width style color".split():
            if column_rule_${name}.is_none() {
                if let Ok(value) = input.try(|input| _moz_column_rule_${name}::parse(context, input)) {
                    column_rule_${name} = Some(value);
                    any = true;
                    continue
                }
            }
            % endfor

            break
        }
        if any {
            Ok(Longhands {
                % for name in "width style".split():
                    _moz_column_rule_${name}: column_rule_${name}
                        .or(Some(_moz_column_rule_${name}::get_initial_specified_value())),
                % endfor
                _moz_column_rule_color: column_rule_color,
            })
        } else {
            Err(())
        }
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut need_space = false;
            try!(self._moz_column_rule_width.to_css(dest));

            if let DeclaredValue::Value(ref width) = *self._moz_column_rule_width {
                try!(width.to_css(dest));
                need_space = true;
            }

            if let DeclaredValue::Value(ref style) = *self._moz_column_rule_style {
                if need_space {
                    try!(write!(dest, " "));
                }
                try!(style.to_css(dest));
                need_space = true;
            }

            if let DeclaredValue::Value(ref color) = *self._moz_column_rule_color {
                if need_space {
                    try!(write!(dest, " "));
                }
                try!(color.to_css(dest));
            }
            Ok(())
        }
    }
</%helpers:shorthand>

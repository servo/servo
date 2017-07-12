/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Gecko's pseudo-element definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PseudoElement {
    % for pseudo in PSEUDOS:
        /// ${pseudo.value}
        % if pseudo.is_tree_pseudo_element():
        ${pseudo.capitalized()}(Box<[String]>),
        % else:
        ${pseudo.capitalized()},
        % endif
    % endfor
}

<% EAGER_PSEUDOS = ["Before", "After", "FirstLine", "FirstLetter"] %>

/// The number of eager pseudo-elements.
pub const EAGER_PSEUDO_COUNT: usize = ${len(EAGER_PSEUDOS)};

/// The list of eager pseudos.
pub const EAGER_PSEUDOS: [PseudoElement; EAGER_PSEUDO_COUNT] = [
    % for eager_pseudo_name in EAGER_PSEUDOS:
    PseudoElement::${eager_pseudo_name},
    % endfor
];

<% TREE_PSEUDOS = [pseudo for pseudo in PSEUDOS if pseudo.is_tree_pseudo_element()] %>
<% SIMPLE_PSEUDOS = [pseudo for pseudo in PSEUDOS if not pseudo.is_tree_pseudo_element()] %>

<%def name="pseudo_element_variant(pseudo, tree_arg='..')">\
PseudoElement::${pseudo.capitalized()}${"({})".format(tree_arg) if pseudo.is_tree_pseudo_element() else ""}\
</%def>

impl PseudoElement {
    /// Executes a closure with each simple (not functional)
    /// pseudo-element as an argument.
    pub fn each_simple<F>(mut fun: F)
        where F: FnMut(Self),
    {
        % for pseudo in SIMPLE_PSEUDOS:
            fun(${pseudo_element_variant(pseudo)});
        % endfor
    }

    /// Get the pseudo-element as an atom.
    #[inline]
    pub fn atom(&self) -> Atom {
        match *self {
            % for pseudo in PSEUDOS:
                ${pseudo_element_variant(pseudo)} => atom!("${pseudo.value}"),
            % endfor
        }
    }

    /// Whether this pseudo-element is an anonymous box.
    #[inline]
    fn is_anon_box(&self) -> bool {
        match *self {
            % for pseudo in PSEUDOS:
                % if pseudo.is_anon_box():
                    ${pseudo_element_variant(pseudo)} => true,
                % endif
            % endfor
            _ => false,
        }
    }

    /// Whether this pseudo-element is eagerly-cascaded.
    #[inline]
    pub fn is_eager(&self) -> bool {
        matches!(*self,
                 ${" | ".join(map(lambda name: "PseudoElement::{}".format(name), EAGER_PSEUDOS))})
    }

    /// Gets the flags associated to this pseudo-element, or 0 if it's an
    /// anonymous box.
    pub fn flags(&self) -> u32 {
        match *self {
            % for pseudo in PSEUDOS:
                ${pseudo_element_variant(pseudo)} =>
                % if pseudo.is_tree_pseudo_element():
                    0,
                % elif pseudo.is_anon_box():
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                % else:
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_${pseudo.original_ident},
                % endif
            % endfor
        }
    }

    /// Construct a pseudo-element from a `CSSPseudoElementType`.
    #[inline]
    pub fn from_pseudo_type(type_: CSSPseudoElementType) -> Option<Self> {
        match type_ {
            % for pseudo in PSEUDOS:
                % if not pseudo.is_anon_box():
                    CSSPseudoElementType::${pseudo.original_ident} => {
                        Some(${pseudo_element_variant(pseudo)})
                    },
                % endif
            % endfor
            _ => None,
        }
    }

    /// Construct a pseudo-element from an anonymous box `Atom`.
    #[inline]
    pub fn from_anon_box_atom(atom: &Atom) -> Option<Self> {
        % for pseudo in PSEUDOS:
            % if pseudo.is_tree_pseudo_element():
                // We cannot generate ${pseudo_element_variant(pseudo)} from just an atom.
            % elif pseudo.is_anon_box():
                if atom == &atom!("${pseudo.value}") {
                    return Some(${pseudo_element_variant(pseudo)});
                }
            % endif
        % endfor
        None
    }

    /// Constructs an atom from a string of text, and whether we're in a
    /// user-agent stylesheet.
    ///
    /// If we're not in a user-agent stylesheet, we will never parse anonymous
    /// box pseudo-elements.
    ///
    /// Returns `None` if the pseudo-element is not recognised.
    #[inline]
    pub fn from_slice(s: &str, in_ua_stylesheet: bool) -> Option<Self> {
        use std::ascii::AsciiExt;

        // We don't need to support tree pseudos because functional
        // pseudo-elements needs arguments, and thus should be created
        // via other methods.
        % for pseudo in SIMPLE_PSEUDOS:
            if in_ua_stylesheet || ${pseudo_element_variant(pseudo)}.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("${pseudo.value[1:]}") {
                    return Some(${pseudo_element_variant(pseudo)});
                }
            }
        % endfor

        None
    }

    /// Constructs a tree pseudo-element from the given name and arguments.
    /// "name" must start with "-moz-tree-".
    ///
    /// Returns `None` if the pseudo-element is not recognized.
    #[inline]
    pub fn tree_pseudo_element(name: &str, args: Box<[String]>) -> Option<Self> {
        use std::ascii::AsciiExt;
        debug_assert!(name.starts_with("-moz-tree-"));
        let tree_part = &name[10..];
        % for pseudo in TREE_PSEUDOS:
            if tree_part.eq_ignore_ascii_case("${pseudo.value[11:]}") {
                return Some(${pseudo_element_variant(pseudo, "args")});
            }
        % endfor
        None
    }
}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_char(':')?;
        match *self {
            % for pseudo in PSEUDOS:
                ${pseudo_element_variant(pseudo)} => dest.write_str("${pseudo.value}")?,
            % endfor
        }
        match *self {
            ${" |\n            ".join("PseudoElement::{}(ref args)".format(pseudo.capitalized())
                                      for pseudo in TREE_PSEUDOS)} => {
                dest.write_char('(')?;
                let mut iter = args.iter();
                if let Some(first) = iter.next() {
                    serialize_identifier(first, dest)?;
                    for item in iter {
                        dest.write_str(", ")?;
                        serialize_identifier(item, dest)?;
                    }
                }
                dest.write_char(')')
            }
            _ => Ok(()),
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Gecko's pseudo-element definition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

/// Important: If you change this, you should also update Gecko's
/// nsCSSPseudoElements::IsEagerlyCascadedInServo.
<% EAGER_PSEUDOS = ["Before", "After", "FirstLine", "FirstLetter"] %>
<% TREE_PSEUDOS = [pseudo for pseudo in PSEUDOS if pseudo.is_tree_pseudo_element()] %>
<% SIMPLE_PSEUDOS = [pseudo for pseudo in PSEUDOS if not pseudo.is_tree_pseudo_element()] %>

/// The number of eager pseudo-elements.
pub const EAGER_PSEUDO_COUNT: usize = ${len(EAGER_PSEUDOS)};

/// The number of non-functional pseudo-elements.
pub const SIMPLE_PSEUDO_COUNT: usize = ${len(SIMPLE_PSEUDOS)};

/// The list of eager pseudos.
pub const EAGER_PSEUDOS: [PseudoElement; EAGER_PSEUDO_COUNT] = [
    % for eager_pseudo_name in EAGER_PSEUDOS:
    PseudoElement::${eager_pseudo_name},
    % endfor
];

<%def name="pseudo_element_variant(pseudo, tree_arg='..')">\
PseudoElement::${pseudo.capitalized()}${"({})".format(tree_arg) if pseudo.is_tree_pseudo_element() else ""}\
</%def>

impl PseudoElement {
    /// Get the pseudo-element as an atom.
    #[inline]
    pub fn atom(&self) -> Atom {
        match *self {
            % for pseudo in PSEUDOS:
                ${pseudo_element_variant(pseudo)} => atom!("${pseudo.value}"),
            % endfor
        }
    }

    /// Returns an index if the pseudo-element is a simple (non-functional)
    /// pseudo.
    #[inline]
    pub fn simple_index(&self) -> Option<usize> {
        match *self {
            % for i, pseudo in enumerate(SIMPLE_PSEUDOS):
            ${pseudo_element_variant(pseudo)} => Some(${i}),
            % endfor
            _ => None,
        }
    }

    /// Returns an array of `None` values.
    ///
    /// FIXME(emilio): Integer generics can't come soon enough.
    pub fn simple_pseudo_none_array<T>() -> [Option<T>; SIMPLE_PSEUDO_COUNT] {
        [
            ${",\n".join(["None" for pseudo in SIMPLE_PSEUDOS])}
        ]
    }

    /// Whether this pseudo-element is an anonymous box.
    #[inline]
    pub fn is_anon_box(&self) -> bool {
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

    /// Construct a `CSSPseudoElementType` from a pseudo-element
    #[inline]
    pub fn pseudo_type(&self) -> CSSPseudoElementType {
        use gecko_bindings::structs::CSSPseudoElementType_InheritingAnonBox;

        match *self {
            % for pseudo in PSEUDOS:
                % if not pseudo.is_anon_box():
                    PseudoElement::${pseudo.capitalized()} => CSSPseudoElementType::${pseudo.original_ident},
                % elif pseudo.is_tree_pseudo_element():
                    PseudoElement::${pseudo.capitalized()}(..) => CSSPseudoElementType_InheritingAnonBox,
                % elif pseudo.is_inheriting_anon_box():
                    PseudoElement::${pseudo.capitalized()} => CSSPseudoElementType_InheritingAnonBox,
                % else:
                    PseudoElement::${pseudo.capitalized()} => CSSPseudoElementType::NonInheritingAnonBox,
                % endif
            % endfor
        }
    }

    /// Get a PseudoInfo for a pseudo
    pub fn pseudo_info(&self) -> (*mut structs::nsIAtom, CSSPseudoElementType) {
        (self.atom().as_ptr(), self.pseudo_type())
    }

    /// Construct a pseudo-element from an `Atom`.
    #[inline]
    pub fn from_atom(atom: &Atom) -> Option<Self> {
        % for pseudo in PSEUDOS:
            % if pseudo.is_tree_pseudo_element():
                // We cannot generate ${pseudo_element_variant(pseudo)} from just an atom.
            % else:
                if atom == &atom!("${pseudo.value}") {
                    return Some(${pseudo_element_variant(pseudo)});
                }
            % endif
        % endfor
        None
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

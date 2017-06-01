/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Gecko's pseudo-element definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PseudoElement {
    % for pseudo in PSEUDOS:
        /// ${pseudo.value}
        ${pseudo.capitalized()},
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

impl PseudoElement {
    /// Executes a closure with each pseudo-element as an argument.
    pub fn each<F>(mut fun: F)
        where F: FnMut(Self),
    {
        % for pseudo in PSEUDOS:
            fun(PseudoElement::${pseudo.capitalized()});
        % endfor
    }

    /// Get the pseudo-element as an atom.
    #[inline]
    pub fn atom(&self) -> Atom {
        match *self {
            % for pseudo in PSEUDOS:
                PseudoElement::${pseudo.capitalized()} => atom!("${pseudo.value}"),
            % endfor
        }
    }

    /// Whether this pseudo-element is an anonymous box.
    #[inline]
    fn is_anon_box(&self) -> bool {
        match *self {
            % for pseudo in PSEUDOS:
                PseudoElement::${pseudo.capitalized()} => ${str(pseudo.is_anon_box()).lower()},
            % endfor
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
                PseudoElement::${pseudo.capitalized()} => {
                    % if pseudo.is_anon_box():
                        0
                    % else:
                        structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_${pseudo.original_ident}
                    % endif
                }
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
                        Some(PseudoElement::${pseudo.capitalized()})
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
            % if pseudo.is_anon_box():
                if atom == &atom!("${pseudo.value}") {
                    return Some(PseudoElement::${pseudo.capitalized()});
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

        % for pseudo in PSEUDOS:
            if in_ua_stylesheet || PseudoElement::${pseudo.capitalized()}.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("${pseudo.value[1:]}") {
                    return Some(PseudoElement::${pseudo.capitalized()})
                }
            }
        % endfor

        None
    }

    /// Returns the pseudo-element's definition as a string, with only one colon
    /// before it.
    pub fn as_str(&self) -> &'static str {
        match *self {
        % for pseudo in PSEUDOS:
            PseudoElement::${pseudo.capitalized()} => "${pseudo.value}",
        % endfor
        }
    }
}

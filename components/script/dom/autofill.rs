/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use servo_atoms::Atom;
use style::str::split_html_space_chars;

use self::AutofillCategory::*;
use crate::dom::bindings::import::module::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::htmlformelement::FormControl;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AutofillCategory {
    Off,
    Automatic,
    Normal,
    Contact,
    Credential,
}

const FIELD_CATEGORY_MAPPING: &[(&str, AutofillCategory)] = &[
    ("additional-name", Normal),
    ("address-level1", Normal),
    ("address-level2", Normal),
    ("address-level3", Normal),
    ("address-level4", Normal),
    ("address-line1", Normal),
    ("address-line2", Normal),
    ("address-line3", Normal),
    ("bday", Normal),
    ("bday-day", Normal),
    ("bday-month", Normal),
    ("bday-year", Normal),
    ("cc-additional-name", Normal),
    ("cc-csc", Normal),
    ("cc-exp", Normal),
    ("cc-exp-month", Normal),
    ("cc-exp-year", Normal),
    ("cc-family-name", Normal),
    ("cc-given-name", Normal),
    ("cc-name", Normal),
    ("cc-number", Normal),
    ("cc-type", Normal),
    ("country", Normal),
    ("country-name", Normal),
    ("current-password", Normal),
    ("device-eid", Normal),
    ("device-imei", Normal),
    ("email", Contact),
    ("family-name", Normal),
    ("given-name", Normal),
    ("honorific-prefix", Normal),
    ("honorific-suffix", Normal),
    ("impp", Contact),
    ("language", Normal),
    ("name", Normal),
    ("new-password", Normal),
    ("nickname", Normal),
    ("off", Off),
    ("on", Automatic),
    ("one-time-code", Normal),
    ("organization", Normal),
    ("organization-title", Normal),
    ("photo", Normal),
    ("postal-code", Normal),
    ("sex", Normal),
    ("street-address", Normal),
    ("tel", Contact),
    ("tel-area-code", Contact),
    ("tel-country-code", Contact),
    ("tel-extension", Contact),
    ("tel-local", Contact),
    ("tel-local-prefix", Contact),
    ("tel-local-suffix", Contact),
    ("tel-national", Contact),
    ("transaction-amount", Normal),
    ("transaction-currency", Normal),
    ("url", Normal),
    ("username", Normal),
    ("webauthn", Credential),
];

#[derive(Debug, PartialEq)]
pub enum NonAutofillCredentialType {
    WebAuthn,
}

#[derive(Debug, PartialEq)]
pub enum AutofillMantle {
    Anchor,
    Expectation,
}

#[allow(unused)]
#[derive(Debug)]
pub struct AutofillData {
    field_name: Atom,
    pub idl_exposed_value: DOMString,
    credential_type: Option<NonAutofillCredentialType>,
}

impl AutofillData {
    fn new(
        field_name: Atom,
        idl_exposed_value: DOMString,
        credential_type: Option<NonAutofillCredentialType>,
    ) -> Self {
        Self {
            field_name,
            idl_exposed_value,
            credential_type,
        }
    }

    // https://html.spec.whatwg.org/multipage/#autofill-processing-model
    pub fn from_form_control<E: FormControl>(element: &E) -> Self {
        let default_label = || {
            // 32. Default: Let the element's IDL-exposed autofill value be the empty string, and its autofill hint set
            // and autofill scope be empty.

            // 33. If the element's autocomplete attribute is wearing the autofill anchor mantle, then let the element's
            // autofill field name be the empty string and return.
            if element.autofill_mantle() == AutofillMantle::Anchor {
                return AutofillData::new(Atom::from(""), DOMString::new(), None);
            }

            // 34. Let form be the element's form owner, if any, or null otherwise.
            if let Some(form) = element.form_owner() {
                // 35. If form is not null and form's autocomplete attribute is in the off state, then let the element's
                // autofill field name be "off".
                if form
                    .upcast::<Element>()
                    .get_string_attribute(&local_name!("autocomplete"))
                    .eq_ignore_ascii_case("off")
                {
                    return AutofillData::new(Atom::from("off"), DOMString::new(), None);
                }
            }

            // Otherwise, let the element's autofill field name be "on".
            AutofillData::new(Atom::from("on"), DOMString::new(), None)
        };

        // 1. If the element has no autocomplete attribute, then jump to the step labeled default.
        if !element
            .to_element()
            .has_attribute(&local_name!("autocomplete"))
        {
            return default_label();
        }

        let mut attribute_value = element
            .to_element()
            .get_string_attribute(&local_name!("autocomplete"));
        attribute_value.make_ascii_lowercase();

        // 2. Let tokens be the result of splitting the attribute's value on ASCII whitespace.
        let tokens = split_html_space_chars(&attribute_value)
            .map(Atom::from)
            .collect::<Vec<Atom>>();

        // 3. If tokens is empty, then jump to the step labeled default.
        if tokens.is_empty() {
            return default_label();
        }

        // 4. Let index be the index of the last token in tokens.
        let mut index = tokens.len() - 1;

        // 5. Let field be the indexth token in tokens.
        let field = tokens[index].clone();

        // 6. Set the category, maximum tokens pair to the result of determining a field's category given field.
        let Some(mut category) = token_category(&field) else {
            // 7. If category is null, then jump to the step labeled default.
            return default_label();
        };

        // 8. If the number of tokens in tokens is greater than maximum tokens, then jump to the step labeled default.
        if tokens.len() > category.max_tokens() {
            return default_label();
        }

        // 9. If category is Off or Automatic but the element's autocomplete attribute is wearing the autofill anchor
        // mantle, then jump to the step labeled default.
        if matches!(category, Off | Automatic) &&
            element.autofill_mantle() == AutofillMantle::Anchor
        {
            return default_label();
        }

        // 10. If category is Off, let the element's autofill field name be the string "off", let its autofill hint set
        // be empty, and let its IDL-exposed autofill value be the string "off". Then, return.
        if category == Off {
            return AutofillData::new(Atom::from("off"), DOMString::from("off"), None);
        }

        // 11. If category is Automatic, let the element's autofill field name be the string "on", let its autofill hint
        // set be empty, and let its IDL-exposed autofill value be the string "on". Then, return.
        if category == Automatic {
            return AutofillData::new(Atom::from("on"), DOMString::from("on"), None);
        }

        // TODO 12. Let scope tokens be an empty list.
        // TODO 13. Let hint tokens be an empty set.

        // 14. Let credential type be null.
        let mut credential_type = None;

        // 15. Let IDL value have the same value as field.
        let mut idl_value = field.to_string();

        // 16. If category is Credential and the indexth token in tokens is an ASCII case-insensitive match
        // for "webauthn", then run the substeps that follow:
        if category == Credential && &field == "webauthn" {
            // 16.1. Set credential type to "webauthn".
            credential_type = Some(NonAutofillCredentialType::WebAuthn);

            // 16.2. If the indexth token in tokens is the first entry, then skip to the step labeled done.
            if index == 0 {
                return AutofillData::new(field, DOMString::from(idl_value), credential_type);
            }

            // 16.3. Decrement index by one.
            index -= 1;

            // 16.4. Set the category, maximum tokens pair to the result of determining a field's category given the
            // indexth token in tokens.
            // 16.5 If category is not Normal and category is not Contact, then jump to the step labeled default.
            match token_category(&tokens[index]) {
                Some(c @ (Normal | Contact)) => category = c,
                _ => return default_label(),
            }

            // 16.6. If index is greater than maximum tokens minus one (i.e. if the number of remaining tokens is
            // greater than maximum tokens), then jump to the step labeled default.
            if index > category.max_tokens() - 1 {
                return default_label();
            }

            // 16.7. Set IDL value to the concatenation of the indexth token in tokens, a U+0020 SPACE character, and
            // the previous value of IDL value.
            idl_value = format!("{} {}", tokens[index], idl_value);
        }

        // 17. If the indexth token in tokens is the first entry, then skip to the step labeled done.
        if index == 0 {
            return AutofillData::new(field, DOMString::from(idl_value), credential_type);
        }

        // 18. Decrement index by one.
        index -= 1;

        // 19. If category is Contact and the indexth token in tokens is an ASCII case-insensitive match for one of the
        // strings in the following list, then run the substeps that follow:
        let contact_token = &tokens[index];
        if category == Contact &&
            matches!(
                contact_token.as_ref(),
                "home" | "work" | "mobile" | "fax" | "pager"
            )
        {
            // 19.1. Let contact be the matching string from the list above.
            let contact = contact_token;

            // TODO 19.2. Insert contact at the start of scope tokens.
            // TODO 19.3. Add contact to hint tokens.

            // 19.4. Let IDL value be the concatenation of contact, a U+0020 SPACE character, and the previous value of
            // IDL value.
            idl_value = format!("{} {}", contact, idl_value);

            // 19.5. If the indexth entry in tokens is the first entry, then skip to the step labeled done.
            if index == 0 {
                return AutofillData::new(field, DOMString::from(idl_value), credential_type);
            }

            // 19.6. Decrement index by one.
            index -= 1;
        }

        // 20. If the indexth token in tokens is an ASCII case-insensitive match for one of the strings in the following
        // list, then run the substeps that follow:
        let mode_token = &tokens[index];
        if matches!(mode_token.as_ref(), "shipping" | "billing") {
            // 20.1. Let mode be the matching string from the list above.
            let mode = mode_token;

            // TODO 20.2. Insert mode at the start of scope tokens.
            // TODO 20.3. Add mode to hint tokens.

            // 20.4. Let IDL value be the concatenation of mode, a U+0020 SPACE character, and the previous value of IDL
            // value.
            idl_value = format!("{} {}", mode, idl_value);

            // 20.5. If the indexth entry in tokens is the first entry, then skip to the step labeled done.
            if index == 0 {
                return AutofillData::new(field, DOMString::from(idl_value), credential_type);
            }

            // 20.6. Decrement index by one.
            index -= 1;
        }

        // 21. If the indexth entry in tokens is not the first entry, then jump to the step labeled default.
        if index != 0 {
            return default_label();
        }

        // 22. If the first eight characters of the indexth token in tokens are not an ASCII case-insensitive match for
        // the string "section-", then jump to the step labeled default.
        let section_token = &tokens[index];
        if !section_token.starts_with("section-") {
            return default_label();
        }

        // 23. Let section be the indexth token in tokens, converted to ASCII lowercase.
        let section = section_token;

        // TODO 24. Insert section at the start of scope tokens.

        // 25. Let IDL value be the concatenation of section, a U+0020 SPACE character, and the previous value of IDL
        // value.
        let idl_value = format!("{} {}", section, idl_value);

        // Done:
        // TODO 26. Let the element's autofill hint set be hint tokens.
        // 27. Let the element's non-autofill credential type be credential type.
        // TODO 28. Let the element's autofill scope be scope tokens.
        // 29. Let the element's autofill field name be field.
        // 30. Let the element's IDL-exposed autofill value be IDL value.
        // 31. Return.
        AutofillData::new(field, DOMString::from(idl_value), credential_type)
    }
}

fn token_category(field: &str) -> Option<AutofillCategory> {
    FIELD_CATEGORY_MAPPING
        .iter()
        .find(|&&(f, _)| f == field)
        .map(|&(_, c)| c)
}

impl AutofillCategory {
    fn max_tokens(&self) -> usize {
        match self {
            Off | Automatic => 1,
            Normal => 3,
            Contact => 4,
            Credential => 5,
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod document_metadata;
pub(crate) use self::document_metadata::*;
pub(crate) mod document_structure;
pub(crate) use self::document_structure::*;
pub(crate) mod embedded_content;
pub(crate) use self::embedded_content::*;
pub(crate) mod form_controls;
pub(crate) use self::form_controls::*;
pub(crate) mod grouping_content;
pub(crate) use self::grouping_content::*;
pub(crate) mod interactive;
pub(crate) use self::interactive::*;
pub(crate) mod internals;
pub(crate) use self::internals::*;
pub(crate) mod links;
pub(crate) mod scripting;
pub(crate) use self::scripting::*;
pub(crate) mod tabular_data;
pub(crate) use self::tabular_data::*;
pub(crate) mod textual;
pub(crate) use self::textual::*;
pub(crate) mod htmlcollection;
pub(crate) mod htmlelement;
pub(crate) mod htmlslotelement;
pub(crate) mod htmlunknownelement;

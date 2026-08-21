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
pub(crate) mod scripting;
pub(crate) use self::scripting::*;
pub(crate) mod tabular_data;
pub(crate) use self::tabular_data::*;
pub(crate) mod textual;
pub(crate) use self::textual::*;
pub(crate) mod htmlcollection;
pub(crate) mod htmldirectoryelement;
pub(crate) mod htmlelement;
pub(crate) mod htmlfontelement;
pub(crate) mod htmlformcontrolscollection;
pub(crate) mod htmlheadingelement;
pub(crate) mod htmlhyperlinkelementutils;
pub(crate) mod htmlmarqueeelement;
pub(crate) mod htmlmenuelement;
pub(crate) mod htmlmodelement;
pub(crate) mod htmloptionscollection;
pub(crate) mod htmlparamelement;
pub(crate) mod htmlpictureelement;
pub(crate) mod htmlquoteelement;
pub(crate) mod htmlslotelement;
pub(crate) mod htmlsourceelement;
pub(crate) mod htmlunknownelement;
pub(crate) mod interactive_element_command;

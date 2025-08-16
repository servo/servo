/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::id::{QuotaExceededErrorId, QuotaExceededErrorIndex};
use constellation_traits::SerializableQuotaExceededError;
use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::QuotaExceededErrorBinding::{
    QuotaExceededErrorMethods, QuotaExceededErrorOptions,
};
use script_bindings::num::Finite;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;

use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, reflect_dom_object_with_proto};
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::types::{DOMException, GlobalScope};

/// <https://webidl.spec.whatwg.org/#quotaexceedederror>
#[dom_struct]
pub(crate) struct QuotaExceededError {
    /// <https://webidl.spec.whatwg.org/#idl-DOMException>
    dom_exception: DOMException,
    /// <https://webidl.spec.whatwg.org/#dom-quotaexceedederror-quota>
    quota: Option<Finite<f64>>,
    /// <https://webidl.spec.whatwg.org/#dom-quotaexceedederror-requested>
    requested: Option<Finite<f64>>,
}

impl QuotaExceededError {
    fn new_inherited(
        message: DOMString,
        quota: Option<Finite<f64>>,
        requested: Option<Finite<f64>>,
    ) -> Self {
        Self {
            dom_exception: DOMException::new_inherited(
                message,
                DOMString::from_string("QuotaExceededError".to_string()),
            ),
            quota,
            requested,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        message: DOMString,
        quota: Option<Finite<f64>>,
        requested: Option<Finite<f64>>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(message, quota, requested)),
            global,
            can_gc,
        )
    }
}

impl QuotaExceededErrorMethods<crate::DomTypeHolder> for QuotaExceededError {
    /// <https://webidl.spec.whatwg.org/#dom-quotaexceedederror-quotaexceedederror>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        message: DOMString,
        options: &QuotaExceededErrorOptions,
    ) -> Result<DomRoot<Self>, Error> {
        // If options["quota"] is present:
        if let Some(quota) = options.quota {
            // If options["quota"] is less than 0, then throw a RangeError.
            if *quota < 0.0 {
                return Err(Error::Range(
                    "quota must be at least zero if present".to_string(),
                ));
            }
        }
        // If options["requested"] is present:
        if let Some(requested) = options.requested {
            // If options["requested"] is less than 0, then throw a RangeError.
            if *requested < 0.0 {
                return Err(Error::Range(
                    "requested must be at least zero if present".to_string(),
                ));
            }
        }
        // If this’s quota is not null, this’s requested is not null, and this’s requested
        // is less than this’s quota, then throw a RangeError.
        if let (Some(quota), Some(requested)) = (options.quota, options.requested) {
            if *requested < *quota {
                return Err(Error::Range("requested is less than quota".to_string()));
            }
        }
        Ok(reflect_dom_object_with_proto(
            Box::new(QuotaExceededError::new_inherited(
                message,
                options.quota,
                options.requested,
            )),
            global,
            proto,
            can_gc,
        ))
    }

    /// <https://webidl.spec.whatwg.org/#dom-quotaexceedederror-quota>
    fn GetQuota(&self) -> Option<Finite<f64>> {
        // The quota getter steps are to return this’s quota.
        self.quota
    }

    /// <https://webidl.spec.whatwg.org/#dom-quotaexceedederror-requested>
    fn GetRequested(&self) -> Option<Finite<f64>> {
        // The requested getter steps are to return this’s requested.
        self.requested
    }
}

impl Serializable for QuotaExceededError {
    type Index = QuotaExceededErrorIndex;
    type Data = SerializableQuotaExceededError;

    /// <https://webidl.spec.whatwg.org/#quotaexceedederror>
    fn serialize(&self) -> Result<(QuotaExceededErrorId, Self::Data), ()> {
        let (_, dom_exception) = self.dom_exception.serialize()?;
        let serialized = SerializableQuotaExceededError {
            dom_exception,
            quota: self.quota.as_deref().copied(),
            requested: self.requested.as_deref().copied(),
        };
        Ok((QuotaExceededErrorId::new(), serialized))
    }

    /// <https://webidl.spec.whatwg.org/#quotaexceedederror>
    fn deserialize(
        owner: &GlobalScope,
        serialized: Self::Data,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()>
    where
        Self: Sized,
    {
        Ok(Self::new(
            owner,
            DOMString::from(serialized.dom_exception.message),
            serialized
                .quota
                .map(|val| Finite::new(val).ok_or(()))
                .transpose()?,
            serialized
                .requested
                .map(|val| Finite::new(val).ok_or(()))
                .transpose()?,
            can_gc,
        ))
    }

    /// <https://webidl.spec.whatwg.org/#quotaexceedederror>
    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<QuotaExceededErrorId, Self::Data>> {
        match data {
            StructuredData::Reader(reader) => &mut reader.quota_exceeded_errors,
            StructuredData::Writer(writer) => &mut writer.quota_exceeded_errors,
        }
    }
}

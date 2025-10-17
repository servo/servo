/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[derive(jstraceable_derive::JSTraceable, malloc_size_of_derive::MallocSizeOf, PartialEq)]
pub(crate) struct DomTypeHolder;
impl crate::DomTypes for DomTypeHolder {
    type GeolocationCoordinates = crate::geolocationcoordinates::GeolocationCoordinates;
}

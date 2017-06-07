/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

impl SpecLinkMethods for SpecLink {
    amacro!("Macros inside impls should trigger spec checks.")

    // Method declarations should trigger spec checks.
    fn Test(&self) -> f32 {
        amacro!("Macros inside function declarations should not trigger spec checks.");
        if unsafe { false } {
        }
        amacro!("Even if there's weird brace counts.");
        0
    }

    // A spec link.
    // https://example.com/
    fn Foo() {}

    /// A spec link.
    /// https://example.com/
    fn Foo() {}

    /// A spec link.
    /// https://example.com/
    /// Doc comments are OK
    // Regular comments are OK
    #[allow(attributes_too)]
    fn Foo() {}
}


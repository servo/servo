/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Counters", inherited=False, gecko_name="Content") %>

${helpers.predefined_type("content",
                          "Content",
                          "computed::Content::normal()",
                          initial_specified_value="specified::Content::normal()",
                          animation_value_type="discrete",
                          spec="https://drafts.csswg.org/css-content/#propdef-content",
                          servo_restyle_damage="rebuild_and_reflow")}

${helpers.predefined_type(
    "counter-increment",
    "CounterIncrement",
    initial_value="Default::default()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-lists/#propdef-counter-increment",
    servo_restyle_damage="rebuild_and_reflow"
)}

${helpers.predefined_type(
    "counter-reset",
    "CounterReset",
    initial_value="Default::default()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-lists-3/#propdef-counter-reset",
    servo_restyle_damage="rebuild_and_reflow"
)}

<%page args="helpers"/>

<%helpers:longhand name="-servo-display-for-hypothetical-box" derived_from="display">
    pub use super::display::{SpecifiedValue, get_initial_value};
    pub use super::display::{parse};

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    #[inline]
    pub fn derive_from_display<Cx: TContext>(context: &mut Cx) {
        let d = context.style().get_box().clone_display();
        context.mutate_style().mutate_box().set__servo_display_for_hypothetical_box(d);
    }

</%helpers:longhand>

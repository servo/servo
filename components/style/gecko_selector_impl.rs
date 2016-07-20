/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use element_state::ElementState;
use selector_impl::{PseudoElementCascadeType, SelectorImplExt};
use selectors::parser::{ParserContext, SelectorImpl};
use string_cache::Atom;
use stylesheets::Stylesheet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeckoSelectorImpl;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PseudoElement {
    Before,
    After,

    Backdrop,
    FirstLetter,
    FirstLine,
    MozSelection,
    MozFocusInner,
    MozFocusOuter,
    MozListBullet,
    MozListNumber,
    MozMathAnonymous,
    MozNumberWrapper,
    MozNumberText,
    MozNumberSpinBox,
    MozNumberSpinUp,
    MozNumberSpinDown,
    MozProgressBar,
    MozRangeTrack,
    MozRangeProgress,
    MozRangeThumb,
    MozMeterBar,
    MozPlaceholder,
    MozColorSwatch,

    AnonBox(AnonBoxPseudoElement),
}

// https://mxr.mozilla.org/mozilla-central/source/layout/style/nsCSSAnonBoxList.h
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AnonBoxPseudoElement {
    MozNonElement,
    MozAnonymousBlock,
    MozAnonymousPositionedBlock,
    MozMathMLAnonymousBlock,
    MozXULAnonymousBlock,

    MozHorizontalFramesetBorder,
    MozVerticalFramesetBorder,
    MozLineFrame,
    MozButtonContent,
    MozButtonLabel,
    MozCellContent,
    MozDropdownList,
    MozFieldsetContent,
    MozFramesetBlank,
    MozDisplayComboboxControlFrame,

    MozHTMLCanvasContent,
    MozInlineTable,
    MozTable,
    MozTableCell,
    MozTableColumnGroup,
    MozTableColumn,
    MozTableOuter,
    MozTableRowGroup,
    MozTableRow,

    MozCanvas,
    MozPageBreak,
    MozPage,
    MozPageContent,
    MozPageSequence,
    MozScrolledContent,
    MozScrolledCanvas,
    MozScrolledPageSequence,
    MozColumnContent,
    MozViewport,
    MozViewportScroll,
    MozAnonymousFlexItem,
    MozAnonymousGridItem,

    MozRuby,
    MozRubyBase,
    MozRubyBaseContainer,
    MozRubyText,
    MozRubyTextContainer,

    MozTreeColumn,
    MozTreeRow,
    MozTreeSeparator,
    MozTreeCell,
    MozTreeIndentation,
    MozTreeLine,
    MozTreeTwisty,
    MozTreeImage,
    MozTreeCellText,
    MozTreeCheckbox,
    MozTreeProgressMeter,
    MozTreeDropFeedback,

    MozSVGMarkerAnonChild,
    MozSVGOuterSVGAnonChild,
    MozSVGForeignContent,
    MozSVGText,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NonTSPseudoClass {
    AnyLink,
    Link,
    Visited,
    Active,
    Focus,
    Hover,
    Enabled,
    Disabled,
    Checked,
    Indeterminate,
    ReadWrite,
    ReadOnly,
}

impl NonTSPseudoClass {
    pub fn state_flag(&self) -> ElementState {
        use element_state::*;
        use self::NonTSPseudoClass::*;
        match *self {
            Active => IN_ACTIVE_STATE,
            Focus => IN_FOCUS_STATE,
            Hover => IN_HOVER_STATE,
            Enabled => IN_ENABLED_STATE,
            Disabled => IN_DISABLED_STATE,
            Checked => IN_CHECKED_STATE,
            Indeterminate => IN_INDETERMINATE_STATE,
            ReadOnly | ReadWrite => IN_READ_WRITE_STATE,

            AnyLink |
            Link |
            Visited => ElementState::empty(),
        }
    }
}

impl SelectorImpl for GeckoSelectorImpl {
    type AttrString = Atom;
    type PseudoElement = PseudoElement;
    type NonTSPseudoClass = NonTSPseudoClass;
    fn parse_non_ts_pseudo_class(_context: &ParserContext,
                                 name: &str) -> Result<NonTSPseudoClass, ()> {
        use self::NonTSPseudoClass::*;
        let pseudo_class = match_ignore_ascii_case! { name,
            "any-link" => AnyLink,
            "link" => Link,
            "visited" => Visited,
            "active" => Active,
            "focus" => Focus,
            "hover" => Hover,
            "enabled" => Enabled,
            "disabled" => Disabled,
            "checked" => Checked,
            "indeterminate" => Indeterminate,
            "read-write" => ReadWrite,
            "read-only" => ReadOnly,
            _ => return Err(())
        };

        Ok(pseudo_class)
    }

    fn parse_pseudo_element(context: &ParserContext,
                            name: &str) -> Result<PseudoElement, ()> {
        use self::AnonBoxPseudoElement::*;
        use self::PseudoElement::*;

        // The braces here are unfortunate, but they're needed for
        // match_ignore_ascii_case! to work as expected.
        match_ignore_ascii_case! { name,
            "before" => { return Ok(Before) },
            "after" => { return Ok(After) },
            "first-line" => { return Ok(FirstLine) },
            "backdrop" => { return Ok(Backdrop) },
            "first-letter" => { return Ok(FirstLetter) },
            "first-line" => { return Ok(FirstLine) },
            "-moz-selection" => { return Ok(MozSelection) },
            "-moz-focus-inner" => { return Ok(MozFocusInner) },
            "-moz-focus-outer" => { return Ok(MozFocusOuter) },
            "-moz-list-bullet" => { return Ok(MozListBullet) },
            "-moz-list-number" => { return Ok(MozListNumber) },
            "-moz-math-anonymous" => { return Ok(MozMathAnonymous) },
            "-moz-number-wrapper" => { return Ok(MozNumberWrapper) },
            "-moz-number-text" => { return Ok(MozNumberText) },
            "-moz-number-spin-box" => { return Ok(MozNumberSpinBox) },
            "-moz-number-spin-up" => { return Ok(MozNumberSpinUp) },
            "-moz-number-spin-down" => { return Ok(MozNumberSpinDown) },
            "-moz-progress-bar" => { return Ok(MozProgressBar) },
            "-moz-range-track" => { return Ok(MozRangeTrack) },
            "-moz-range-progress" => { return Ok(MozRangeProgress) },
            "-moz-range-thumb" => { return Ok(MozRangeThumb) },
            "-moz-metter-bar" => { return Ok(MozMeterBar) },
            "-moz-placeholder" => { return Ok(MozPlaceholder) },
            "-moz-color-swatch" => { return Ok(MozColorSwatch) },

            _ => {}
        }

        if !context.in_user_agent_stylesheet {
            return Err(())
        }

        Ok(AnonBox(match_ignore_ascii_case! { name,
            "-moz-non-element" => MozNonElement,

            "-moz-anonymous-block" => MozAnonymousBlock,
            "-moz-anonymous-positioned-block" => MozAnonymousPositionedBlock,
            "-moz-mathml-anonymous-block" => MozMathMLAnonymousBlock,
            "-moz-xul-anonymous-block" => MozXULAnonymousBlock,

            "-moz-hframeset-border" => MozHorizontalFramesetBorder,
            "-moz-vframeset-border" => MozVerticalFramesetBorder,

            "-moz-line-frame" => MozLineFrame,

            "-moz-button-content" => MozButtonContent,
            "-moz-buttonlabel" => MozButtonLabel,
            "-moz-cell-content" => MozCellContent,
            "-moz-dropdown-list" => MozDropdownList,
            "-moz-fieldset-content" => MozFieldsetContent,
            "-moz-frameset-blank" => MozFramesetBlank,
            "-moz-display-comboboxcontrol-frame" => MozDisplayComboboxControlFrame,
            "-moz-html-canvas-content" => MozHTMLCanvasContent,

            "-moz-inline-table" => MozInlineTable,
            "-moz-table" => MozTable,
            "-moz-table-cell" => MozTableCell,
            "-moz-table-column-group" => MozTableColumnGroup,
            "-moz-table-column" => MozTableColumn,
            "-moz-table-outer" => MozTableOuter,
            "-moz-table-row-group" => MozTableRowGroup,
            "-moz-table-row" => MozTableRow,

            "-moz-canvas" => MozCanvas,
            "-moz-pagebreak" => MozPageBreak,
            "-moz-page" => MozPage,
            "-moz-pagecontent" => MozPageContent,
            "-moz-page-sequence" => MozPageSequence,
            "-moz-scrolled-content" => MozScrolledContent,
            "-moz-scrolled-canvas" => MozScrolledCanvas,
            "-moz-scrolled-page-sequence" => MozScrolledPageSequence,
            "-moz-column-content" => MozColumnContent,
            "-moz-viewport" => MozViewport,
            "-moz-viewport-scroll" => MozViewportScroll,
            "-moz-anonymous-flex-item" => MozAnonymousFlexItem,
            "-moz-anonymous-grid-item" => MozAnonymousGridItem,
            "-moz-ruby" => MozRuby,
            "-moz-ruby-base" => MozRubyBase,
            "-moz-ruby-base-container" => MozRubyBaseContainer,
            "-moz-ruby-text" => MozRubyText,
            "-moz-ruby-text-container" => MozRubyTextContainer,
            "-moz-tree-column" => MozTreeColumn,
            "-moz-tree-row" => MozTreeRow,
            "-moz-tree-separator" => MozTreeSeparator,
            "-moz-tree-cell" => MozTreeCell,
            "-moz-tree-indentation" => MozTreeIndentation,
            "-moz-tree-line" => MozTreeLine,
            "-moz-tree-twisty" => MozTreeTwisty,
            "-moz-tree-image" => MozTreeImage,
            "-moz-tree-cell-text" => MozTreeCellText,
            "-moz-tree-checkbox" => MozTreeCheckbox,
            "-moz-tree-progressmeter" => MozTreeProgressMeter,
            "-moz-tree-drop-feedback" => MozTreeDropFeedback,
            "-moz-svg-marker-anon-child" => MozSVGMarkerAnonChild,
            "-moz-svg-outer-svg-anon-child" => MozSVGOuterSVGAnonChild,
            "-moz-svg-foreign-content" => MozSVGForeignContent,
            "-moz-svg-text" => MozSVGText,

            _ => return Err(())
        }))
    }
}

impl SelectorImplExt for GeckoSelectorImpl {
    #[inline]
    fn pseudo_element_cascade_type(pseudo: &PseudoElement) -> PseudoElementCascadeType {
        match *pseudo {
            PseudoElement::Before |
            PseudoElement::After => PseudoElementCascadeType::Eager,
            PseudoElement::AnonBox(_) => PseudoElementCascadeType::Precomputed,
            _ => PseudoElementCascadeType::Lazy,
        }
    }

    #[inline]
    fn each_pseudo_element<F>(mut fun: F)
        where F: FnMut(PseudoElement) {
        use self::AnonBoxPseudoElement::*;
        use self::PseudoElement::*;

        fun(Before);
        fun(After);
        fun(FirstLine);

        fun(AnonBox(MozNonElement));
        fun(AnonBox(MozAnonymousBlock));
        fun(AnonBox(MozAnonymousPositionedBlock));
        fun(AnonBox(MozMathMLAnonymousBlock));
        fun(AnonBox(MozXULAnonymousBlock));

        fun(AnonBox(MozHorizontalFramesetBorder));
        fun(AnonBox(MozVerticalFramesetBorder));
        fun(AnonBox(MozLineFrame));
        fun(AnonBox(MozButtonContent));
        fun(AnonBox(MozButtonLabel));
        fun(AnonBox(MozCellContent));
        fun(AnonBox(MozDropdownList));
        fun(AnonBox(MozFieldsetContent));
        fun(AnonBox(MozFramesetBlank));
        fun(AnonBox(MozDisplayComboboxControlFrame));

        fun(AnonBox(MozHTMLCanvasContent));
        fun(AnonBox(MozInlineTable));
        fun(AnonBox(MozTable));
        fun(AnonBox(MozTableCell));
        fun(AnonBox(MozTableColumnGroup));
        fun(AnonBox(MozTableColumn));
        fun(AnonBox(MozTableOuter));
        fun(AnonBox(MozTableRowGroup));
        fun(AnonBox(MozTableRow));

        fun(AnonBox(MozCanvas));
        fun(AnonBox(MozPageBreak));
        fun(AnonBox(MozPage));
        fun(AnonBox(MozPageContent));
        fun(AnonBox(MozPageSequence));
        fun(AnonBox(MozScrolledContent));
        fun(AnonBox(MozScrolledCanvas));
        fun(AnonBox(MozScrolledPageSequence));
        fun(AnonBox(MozColumnContent));
        fun(AnonBox(MozViewport));
        fun(AnonBox(MozViewportScroll));
        fun(AnonBox(MozAnonymousFlexItem));
        fun(AnonBox(MozAnonymousGridItem));

        fun(AnonBox(MozRuby));
        fun(AnonBox(MozRubyBase));
        fun(AnonBox(MozRubyBaseContainer));
        fun(AnonBox(MozRubyText));
        fun(AnonBox(MozRubyTextContainer));

        fun(AnonBox(MozTreeColumn));
        fun(AnonBox(MozTreeRow));
        fun(AnonBox(MozTreeSeparator));
        fun(AnonBox(MozTreeCell));
        fun(AnonBox(MozTreeIndentation));
        fun(AnonBox(MozTreeLine));
        fun(AnonBox(MozTreeTwisty));
        fun(AnonBox(MozTreeImage));
        fun(AnonBox(MozTreeCellText));
        fun(AnonBox(MozTreeCheckbox));
        fun(AnonBox(MozTreeProgressMeter));
        fun(AnonBox(MozTreeDropFeedback));

        fun(AnonBox(MozSVGMarkerAnonChild));
        fun(AnonBox(MozSVGOuterSVGAnonChild));
        fun(AnonBox(MozSVGForeignContent));
        fun(AnonBox(MozSVGText));
    }

    #[inline]
    fn pseudo_is_before_or_after(pseudo: &PseudoElement) -> bool {
        match *pseudo {
            PseudoElement::Before |
            PseudoElement::After => true,
            _ => false,
        }
    }

    #[inline]
    fn pseudo_class_state_flag(pc: &NonTSPseudoClass) -> ElementState {
        pc.state_flag()
    }

    #[inline]
    fn get_user_or_user_agent_stylesheets() -> &'static [Stylesheet] {
        &[]
    }

    #[inline]
    fn get_quirks_mode_stylesheet() -> Option<&'static Stylesheet> {
        None
    }
}

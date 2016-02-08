/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use element_state::ElementState;
use selectors::parser::{ParserContext, SelectorImpl};

#[derive(Clone, Debug, PartialEq, Eq, HeapSizeOf, Hash)]
pub enum PseudoElement {
    Before,
    After,
    FirstLine,

    // https://mxr.mozilla.org/mozilla-central/source/layout/style/nsCSSAnonBoxList.h
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

#[derive(Clone, Debug, PartialEq, Eq, HeapSizeOf, Hash)]
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
    ServoNonZeroBorder,
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

            AnyLink |
            Link |
            Visited |
            ServoNonZeroBorder => ElementState::empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, HeapSizeOf)]
pub struct ServoSelectorImpl;

impl SelectorImpl for ServoSelectorImpl {
    type PseudoElement = PseudoElement;
    type NonTSPseudoClass = NonTSPseudoClass;

    fn parse_non_ts_pseudo_class(context: &ParserContext,
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
            "-servo-nonzero-border" => {
                if !context.in_user_agent_stylesheet {
                    return Err(());
                }
                ServoNonZeroBorder
            },
            _ => return Err(())
        };

        Ok(pseudo_class)
    }

    fn parse_pseudo_element(_context: &ParserContext,
                            name: &str) -> Result<PseudoElement, ()> {
        use self::PseudoElement::*;
        let pseudo_element = match_ignore_ascii_case! { name,
            "before" => Before,
            "after" => After,
            "first-line" => FirstLine,

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
        };

        Ok(pseudo_element)
    }
}

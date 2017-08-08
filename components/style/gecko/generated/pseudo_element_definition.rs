/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Gecko's pseudo-element definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PseudoElement {
        /// :after
        After,
        /// :before
        Before,
        /// :backdrop
        Backdrop,
        /// :cue
        Cue,
        /// :first-letter
        FirstLetter,
        /// :first-line
        FirstLine,
        /// :-moz-selection
        MozSelection,
        /// :-moz-focus-inner
        MozFocusInner,
        /// :-moz-focus-outer
        MozFocusOuter,
        /// :-moz-list-bullet
        MozListBullet,
        /// :-moz-list-number
        MozListNumber,
        /// :-moz-math-anonymous
        MozMathAnonymous,
        /// :-moz-number-wrapper
        MozNumberWrapper,
        /// :-moz-number-text
        MozNumberText,
        /// :-moz-number-spin-box
        MozNumberSpinBox,
        /// :-moz-number-spin-up
        MozNumberSpinUp,
        /// :-moz-number-spin-down
        MozNumberSpinDown,
        /// :-moz-progress-bar
        MozProgressBar,
        /// :-moz-range-track
        MozRangeTrack,
        /// :-moz-range-progress
        MozRangeProgress,
        /// :-moz-range-thumb
        MozRangeThumb,
        /// :-moz-meter-bar
        MozMeterBar,
        /// :-moz-placeholder
        MozPlaceholder,
        /// :placeholder
        Placeholder,
        /// :-moz-color-swatch
        MozColorSwatch,
        /// :-moz-text
        MozText,
        /// :-moz-oof-placeholder
        OofPlaceholder,
        /// :-moz-first-letter-continuation
        FirstLetterContinuation,
        /// :-moz-block-inside-inline-wrapper
        MozBlockInsideInlineWrapper,
        /// :-moz-mathml-anonymous-block
        MozMathMLAnonymousBlock,
        /// :-moz-xul-anonymous-block
        MozXULAnonymousBlock,
        /// :-moz-hframeset-border
        HorizontalFramesetBorder,
        /// :-moz-vframeset-border
        VerticalFramesetBorder,
        /// :-moz-line-frame
        MozLineFrame,
        /// :-moz-button-content
        ButtonContent,
        /// :-moz-cell-content
        CellContent,
        /// :-moz-dropdown-list
        DropDownList,
        /// :-moz-fieldset-content
        FieldsetContent,
        /// :-moz-frameset-blank
        FramesetBlank,
        /// :-moz-display-comboboxcontrol-frame
        MozDisplayComboboxControlFrame,
        /// :-moz-html-canvas-content
        HtmlCanvasContent,
        /// :-moz-inline-table
        InlineTable,
        /// :-moz-table
        Table,
        /// :-moz-table-cell
        TableCell,
        /// :-moz-table-column-group
        TableColGroup,
        /// :-moz-table-column
        TableCol,
        /// :-moz-table-wrapper
        TableWrapper,
        /// :-moz-table-row-group
        TableRowGroup,
        /// :-moz-table-row
        TableRow,
        /// :-moz-canvas
        Canvas,
        /// :-moz-pagebreak
        PageBreak,
        /// :-moz-page
        Page,
        /// :-moz-pagecontent
        PageContent,
        /// :-moz-page-sequence
        PageSequence,
        /// :-moz-scrolled-content
        ScrolledContent,
        /// :-moz-scrolled-canvas
        ScrolledCanvas,
        /// :-moz-scrolled-page-sequence
        ScrolledPageSequence,
        /// :-moz-column-content
        ColumnContent,
        /// :-moz-viewport
        Viewport,
        /// :-moz-viewport-scroll
        ViewportScroll,
        /// :-moz-anonymous-flex-item
        AnonymousFlexItem,
        /// :-moz-anonymous-grid-item
        AnonymousGridItem,
        /// :-moz-ruby
        Ruby,
        /// :-moz-ruby-base
        RubyBase,
        /// :-moz-ruby-base-container
        RubyBaseContainer,
        /// :-moz-ruby-text
        RubyText,
        /// :-moz-ruby-text-container
        RubyTextContainer,
        /// :-moz-tree-column
        MozTreeColumn(Box<[String]>),
        /// :-moz-tree-row
        MozTreeRow(Box<[String]>),
        /// :-moz-tree-separator
        MozTreeSeparator(Box<[String]>),
        /// :-moz-tree-cell
        MozTreeCell(Box<[String]>),
        /// :-moz-tree-indentation
        MozTreeIndentation(Box<[String]>),
        /// :-moz-tree-line
        MozTreeLine(Box<[String]>),
        /// :-moz-tree-twisty
        MozTreeTwisty(Box<[String]>),
        /// :-moz-tree-image
        MozTreeImage(Box<[String]>),
        /// :-moz-tree-cell-text
        MozTreeCellText(Box<[String]>),
        /// :-moz-tree-checkbox
        MozTreeCheckbox(Box<[String]>),
        /// :-moz-tree-progressmeter
        MozTreeProgressmeter(Box<[String]>),
        /// :-moz-tree-drop-feedback
        MozTreeDropFeedback(Box<[String]>),
        /// :-moz-svg-marker-anon-child
        MozSVGMarkerAnonChild,
        /// :-moz-svg-outer-svg-anon-child
        MozSVGOuterSVGAnonChild,
        /// :-moz-svg-foreign-content
        MozSVGForeignContent,
        /// :-moz-svg-text
        MozSVGText,
}



/// The number of eager pseudo-elements.
pub const EAGER_PSEUDO_COUNT: usize = 4;

/// The list of eager pseudos.
pub const EAGER_PSEUDOS: [PseudoElement; EAGER_PSEUDO_COUNT] = [
    PseudoElement::Before,
    PseudoElement::After,
    PseudoElement::FirstLine,
    PseudoElement::FirstLetter,
];






impl PseudoElement {
    /// Get the pseudo-element as an atom.
    #[inline]
    pub fn atom(&self) -> Atom {
        match *self {
                PseudoElement::After => atom!(":after"),
                PseudoElement::Before => atom!(":before"),
                PseudoElement::Backdrop => atom!(":backdrop"),
                PseudoElement::Cue => atom!(":cue"),
                PseudoElement::FirstLetter => atom!(":first-letter"),
                PseudoElement::FirstLine => atom!(":first-line"),
                PseudoElement::MozSelection => atom!(":-moz-selection"),
                PseudoElement::MozFocusInner => atom!(":-moz-focus-inner"),
                PseudoElement::MozFocusOuter => atom!(":-moz-focus-outer"),
                PseudoElement::MozListBullet => atom!(":-moz-list-bullet"),
                PseudoElement::MozListNumber => atom!(":-moz-list-number"),
                PseudoElement::MozMathAnonymous => atom!(":-moz-math-anonymous"),
                PseudoElement::MozNumberWrapper => atom!(":-moz-number-wrapper"),
                PseudoElement::MozNumberText => atom!(":-moz-number-text"),
                PseudoElement::MozNumberSpinBox => atom!(":-moz-number-spin-box"),
                PseudoElement::MozNumberSpinUp => atom!(":-moz-number-spin-up"),
                PseudoElement::MozNumberSpinDown => atom!(":-moz-number-spin-down"),
                PseudoElement::MozProgressBar => atom!(":-moz-progress-bar"),
                PseudoElement::MozRangeTrack => atom!(":-moz-range-track"),
                PseudoElement::MozRangeProgress => atom!(":-moz-range-progress"),
                PseudoElement::MozRangeThumb => atom!(":-moz-range-thumb"),
                PseudoElement::MozMeterBar => atom!(":-moz-meter-bar"),
                PseudoElement::MozPlaceholder => atom!(":-moz-placeholder"),
                PseudoElement::Placeholder => atom!(":placeholder"),
                PseudoElement::MozColorSwatch => atom!(":-moz-color-swatch"),
                PseudoElement::MozText => atom!(":-moz-text"),
                PseudoElement::OofPlaceholder => atom!(":-moz-oof-placeholder"),
                PseudoElement::FirstLetterContinuation => atom!(":-moz-first-letter-continuation"),
                PseudoElement::MozBlockInsideInlineWrapper => atom!(":-moz-block-inside-inline-wrapper"),
                PseudoElement::MozMathMLAnonymousBlock => atom!(":-moz-mathml-anonymous-block"),
                PseudoElement::MozXULAnonymousBlock => atom!(":-moz-xul-anonymous-block"),
                PseudoElement::HorizontalFramesetBorder => atom!(":-moz-hframeset-border"),
                PseudoElement::VerticalFramesetBorder => atom!(":-moz-vframeset-border"),
                PseudoElement::MozLineFrame => atom!(":-moz-line-frame"),
                PseudoElement::ButtonContent => atom!(":-moz-button-content"),
                PseudoElement::CellContent => atom!(":-moz-cell-content"),
                PseudoElement::DropDownList => atom!(":-moz-dropdown-list"),
                PseudoElement::FieldsetContent => atom!(":-moz-fieldset-content"),
                PseudoElement::FramesetBlank => atom!(":-moz-frameset-blank"),
                PseudoElement::MozDisplayComboboxControlFrame => atom!(":-moz-display-comboboxcontrol-frame"),
                PseudoElement::HtmlCanvasContent => atom!(":-moz-html-canvas-content"),
                PseudoElement::InlineTable => atom!(":-moz-inline-table"),
                PseudoElement::Table => atom!(":-moz-table"),
                PseudoElement::TableCell => atom!(":-moz-table-cell"),
                PseudoElement::TableColGroup => atom!(":-moz-table-column-group"),
                PseudoElement::TableCol => atom!(":-moz-table-column"),
                PseudoElement::TableWrapper => atom!(":-moz-table-wrapper"),
                PseudoElement::TableRowGroup => atom!(":-moz-table-row-group"),
                PseudoElement::TableRow => atom!(":-moz-table-row"),
                PseudoElement::Canvas => atom!(":-moz-canvas"),
                PseudoElement::PageBreak => atom!(":-moz-pagebreak"),
                PseudoElement::Page => atom!(":-moz-page"),
                PseudoElement::PageContent => atom!(":-moz-pagecontent"),
                PseudoElement::PageSequence => atom!(":-moz-page-sequence"),
                PseudoElement::ScrolledContent => atom!(":-moz-scrolled-content"),
                PseudoElement::ScrolledCanvas => atom!(":-moz-scrolled-canvas"),
                PseudoElement::ScrolledPageSequence => atom!(":-moz-scrolled-page-sequence"),
                PseudoElement::ColumnContent => atom!(":-moz-column-content"),
                PseudoElement::Viewport => atom!(":-moz-viewport"),
                PseudoElement::ViewportScroll => atom!(":-moz-viewport-scroll"),
                PseudoElement::AnonymousFlexItem => atom!(":-moz-anonymous-flex-item"),
                PseudoElement::AnonymousGridItem => atom!(":-moz-anonymous-grid-item"),
                PseudoElement::Ruby => atom!(":-moz-ruby"),
                PseudoElement::RubyBase => atom!(":-moz-ruby-base"),
                PseudoElement::RubyBaseContainer => atom!(":-moz-ruby-base-container"),
                PseudoElement::RubyText => atom!(":-moz-ruby-text"),
                PseudoElement::RubyTextContainer => atom!(":-moz-ruby-text-container"),
                PseudoElement::MozTreeColumn(..) => atom!(":-moz-tree-column"),
                PseudoElement::MozTreeRow(..) => atom!(":-moz-tree-row"),
                PseudoElement::MozTreeSeparator(..) => atom!(":-moz-tree-separator"),
                PseudoElement::MozTreeCell(..) => atom!(":-moz-tree-cell"),
                PseudoElement::MozTreeIndentation(..) => atom!(":-moz-tree-indentation"),
                PseudoElement::MozTreeLine(..) => atom!(":-moz-tree-line"),
                PseudoElement::MozTreeTwisty(..) => atom!(":-moz-tree-twisty"),
                PseudoElement::MozTreeImage(..) => atom!(":-moz-tree-image"),
                PseudoElement::MozTreeCellText(..) => atom!(":-moz-tree-cell-text"),
                PseudoElement::MozTreeCheckbox(..) => atom!(":-moz-tree-checkbox"),
                PseudoElement::MozTreeProgressmeter(..) => atom!(":-moz-tree-progressmeter"),
                PseudoElement::MozTreeDropFeedback(..) => atom!(":-moz-tree-drop-feedback"),
                PseudoElement::MozSVGMarkerAnonChild => atom!(":-moz-svg-marker-anon-child"),
                PseudoElement::MozSVGOuterSVGAnonChild => atom!(":-moz-svg-outer-svg-anon-child"),
                PseudoElement::MozSVGForeignContent => atom!(":-moz-svg-foreign-content"),
                PseudoElement::MozSVGText => atom!(":-moz-svg-text"),
        }
    }

    /// Whether this pseudo-element is an anonymous box.
    #[inline]
    pub fn is_anon_box(&self) -> bool {
        match *self {
                    PseudoElement::MozText => true,
                    PseudoElement::OofPlaceholder => true,
                    PseudoElement::FirstLetterContinuation => true,
                    PseudoElement::MozBlockInsideInlineWrapper => true,
                    PseudoElement::MozMathMLAnonymousBlock => true,
                    PseudoElement::MozXULAnonymousBlock => true,
                    PseudoElement::HorizontalFramesetBorder => true,
                    PseudoElement::VerticalFramesetBorder => true,
                    PseudoElement::MozLineFrame => true,
                    PseudoElement::ButtonContent => true,
                    PseudoElement::CellContent => true,
                    PseudoElement::DropDownList => true,
                    PseudoElement::FieldsetContent => true,
                    PseudoElement::FramesetBlank => true,
                    PseudoElement::MozDisplayComboboxControlFrame => true,
                    PseudoElement::HtmlCanvasContent => true,
                    PseudoElement::InlineTable => true,
                    PseudoElement::Table => true,
                    PseudoElement::TableCell => true,
                    PseudoElement::TableColGroup => true,
                    PseudoElement::TableCol => true,
                    PseudoElement::TableWrapper => true,
                    PseudoElement::TableRowGroup => true,
                    PseudoElement::TableRow => true,
                    PseudoElement::Canvas => true,
                    PseudoElement::PageBreak => true,
                    PseudoElement::Page => true,
                    PseudoElement::PageContent => true,
                    PseudoElement::PageSequence => true,
                    PseudoElement::ScrolledContent => true,
                    PseudoElement::ScrolledCanvas => true,
                    PseudoElement::ScrolledPageSequence => true,
                    PseudoElement::ColumnContent => true,
                    PseudoElement::Viewport => true,
                    PseudoElement::ViewportScroll => true,
                    PseudoElement::AnonymousFlexItem => true,
                    PseudoElement::AnonymousGridItem => true,
                    PseudoElement::Ruby => true,
                    PseudoElement::RubyBase => true,
                    PseudoElement::RubyBaseContainer => true,
                    PseudoElement::RubyText => true,
                    PseudoElement::RubyTextContainer => true,
                    PseudoElement::MozTreeColumn(..) => true,
                    PseudoElement::MozTreeRow(..) => true,
                    PseudoElement::MozTreeSeparator(..) => true,
                    PseudoElement::MozTreeCell(..) => true,
                    PseudoElement::MozTreeIndentation(..) => true,
                    PseudoElement::MozTreeLine(..) => true,
                    PseudoElement::MozTreeTwisty(..) => true,
                    PseudoElement::MozTreeImage(..) => true,
                    PseudoElement::MozTreeCellText(..) => true,
                    PseudoElement::MozTreeCheckbox(..) => true,
                    PseudoElement::MozTreeProgressmeter(..) => true,
                    PseudoElement::MozTreeDropFeedback(..) => true,
                    PseudoElement::MozSVGMarkerAnonChild => true,
                    PseudoElement::MozSVGOuterSVGAnonChild => true,
                    PseudoElement::MozSVGForeignContent => true,
                    PseudoElement::MozSVGText => true,
            _ => false,
        }
    }

    /// Whether this pseudo-element is eagerly-cascaded.
    #[inline]
    pub fn is_eager(&self) -> bool {
        matches!(*self,
                 PseudoElement::Before | PseudoElement::After | PseudoElement::FirstLine | PseudoElement::FirstLetter)
    }

    /// Gets the flags associated to this pseudo-element, or 0 if it's an
    /// anonymous box.
    pub fn flags(&self) -> u32 {
        match *self {
                PseudoElement::After =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_after,
                PseudoElement::Before =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_before,
                PseudoElement::Backdrop =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_backdrop,
                PseudoElement::Cue =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_cue,
                PseudoElement::FirstLetter =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_firstLetter,
                PseudoElement::FirstLine =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_firstLine,
                PseudoElement::MozSelection =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozSelection,
                PseudoElement::MozFocusInner =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozFocusInner,
                PseudoElement::MozFocusOuter =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozFocusOuter,
                PseudoElement::MozListBullet =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozListBullet,
                PseudoElement::MozListNumber =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozListNumber,
                PseudoElement::MozMathAnonymous =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozMathAnonymous,
                PseudoElement::MozNumberWrapper =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberWrapper,
                PseudoElement::MozNumberText =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberText,
                PseudoElement::MozNumberSpinBox =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberSpinBox,
                PseudoElement::MozNumberSpinUp =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberSpinUp,
                PseudoElement::MozNumberSpinDown =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozNumberSpinDown,
                PseudoElement::MozProgressBar =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozProgressBar,
                PseudoElement::MozRangeTrack =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozRangeTrack,
                PseudoElement::MozRangeProgress =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozRangeProgress,
                PseudoElement::MozRangeThumb =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozRangeThumb,
                PseudoElement::MozMeterBar =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozMeterBar,
                PseudoElement::MozPlaceholder =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozPlaceholder,
                PseudoElement::Placeholder =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_placeholder,
                PseudoElement::MozColorSwatch =>
                    structs::SERVO_CSS_PSEUDO_ELEMENT_FLAGS_mozColorSwatch,
                PseudoElement::MozText =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::OofPlaceholder =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::FirstLetterContinuation =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozBlockInsideInlineWrapper =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozMathMLAnonymousBlock =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozXULAnonymousBlock =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::HorizontalFramesetBorder =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::VerticalFramesetBorder =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozLineFrame =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::ButtonContent =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::CellContent =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::DropDownList =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::FieldsetContent =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::FramesetBlank =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozDisplayComboboxControlFrame =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::HtmlCanvasContent =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::InlineTable =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::Table =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::TableCell =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::TableColGroup =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::TableCol =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::TableWrapper =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::TableRowGroup =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::TableRow =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::Canvas =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::PageBreak =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::Page =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::PageContent =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::PageSequence =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::ScrolledContent =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::ScrolledCanvas =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::ScrolledPageSequence =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::ColumnContent =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::Viewport =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::ViewportScroll =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::AnonymousFlexItem =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::AnonymousGridItem =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::Ruby =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::RubyBase =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::RubyBaseContainer =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::RubyText =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::RubyTextContainer =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozTreeColumn(..) =>
                    0,
                PseudoElement::MozTreeRow(..) =>
                    0,
                PseudoElement::MozTreeSeparator(..) =>
                    0,
                PseudoElement::MozTreeCell(..) =>
                    0,
                PseudoElement::MozTreeIndentation(..) =>
                    0,
                PseudoElement::MozTreeLine(..) =>
                    0,
                PseudoElement::MozTreeTwisty(..) =>
                    0,
                PseudoElement::MozTreeImage(..) =>
                    0,
                PseudoElement::MozTreeCellText(..) =>
                    0,
                PseudoElement::MozTreeCheckbox(..) =>
                    0,
                PseudoElement::MozTreeProgressmeter(..) =>
                    0,
                PseudoElement::MozTreeDropFeedback(..) =>
                    0,
                PseudoElement::MozSVGMarkerAnonChild =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozSVGOuterSVGAnonChild =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozSVGForeignContent =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
                PseudoElement::MozSVGText =>
                    structs::CSS_PSEUDO_ELEMENT_UA_SHEET_ONLY,
        }
    }

    /// Construct a pseudo-element from a `CSSPseudoElementType`.
    #[inline]
    pub fn from_pseudo_type(type_: CSSPseudoElementType) -> Option<Self> {
        match type_ {
                    CSSPseudoElementType::after => {
                        Some(PseudoElement::After)
                    },
                    CSSPseudoElementType::before => {
                        Some(PseudoElement::Before)
                    },
                    CSSPseudoElementType::backdrop => {
                        Some(PseudoElement::Backdrop)
                    },
                    CSSPseudoElementType::cue => {
                        Some(PseudoElement::Cue)
                    },
                    CSSPseudoElementType::firstLetter => {
                        Some(PseudoElement::FirstLetter)
                    },
                    CSSPseudoElementType::firstLine => {
                        Some(PseudoElement::FirstLine)
                    },
                    CSSPseudoElementType::mozSelection => {
                        Some(PseudoElement::MozSelection)
                    },
                    CSSPseudoElementType::mozFocusInner => {
                        Some(PseudoElement::MozFocusInner)
                    },
                    CSSPseudoElementType::mozFocusOuter => {
                        Some(PseudoElement::MozFocusOuter)
                    },
                    CSSPseudoElementType::mozListBullet => {
                        Some(PseudoElement::MozListBullet)
                    },
                    CSSPseudoElementType::mozListNumber => {
                        Some(PseudoElement::MozListNumber)
                    },
                    CSSPseudoElementType::mozMathAnonymous => {
                        Some(PseudoElement::MozMathAnonymous)
                    },
                    CSSPseudoElementType::mozNumberWrapper => {
                        Some(PseudoElement::MozNumberWrapper)
                    },
                    CSSPseudoElementType::mozNumberText => {
                        Some(PseudoElement::MozNumberText)
                    },
                    CSSPseudoElementType::mozNumberSpinBox => {
                        Some(PseudoElement::MozNumberSpinBox)
                    },
                    CSSPseudoElementType::mozNumberSpinUp => {
                        Some(PseudoElement::MozNumberSpinUp)
                    },
                    CSSPseudoElementType::mozNumberSpinDown => {
                        Some(PseudoElement::MozNumberSpinDown)
                    },
                    CSSPseudoElementType::mozProgressBar => {
                        Some(PseudoElement::MozProgressBar)
                    },
                    CSSPseudoElementType::mozRangeTrack => {
                        Some(PseudoElement::MozRangeTrack)
                    },
                    CSSPseudoElementType::mozRangeProgress => {
                        Some(PseudoElement::MozRangeProgress)
                    },
                    CSSPseudoElementType::mozRangeThumb => {
                        Some(PseudoElement::MozRangeThumb)
                    },
                    CSSPseudoElementType::mozMeterBar => {
                        Some(PseudoElement::MozMeterBar)
                    },
                    CSSPseudoElementType::mozPlaceholder => {
                        Some(PseudoElement::MozPlaceholder)
                    },
                    CSSPseudoElementType::placeholder => {
                        Some(PseudoElement::Placeholder)
                    },
                    CSSPseudoElementType::mozColorSwatch => {
                        Some(PseudoElement::MozColorSwatch)
                    },
            _ => None,
        }
    }

    /// Construct a `CSSPseudoElementType` from a pseudo-element
    #[inline]
    pub fn pseudo_type(&self) -> CSSPseudoElementType {
        use gecko_bindings::structs::CSSPseudoElementType_InheritingAnonBox;

        match *self {
                    PseudoElement::After => CSSPseudoElementType::after,
                    PseudoElement::Before => CSSPseudoElementType::before,
                    PseudoElement::Backdrop => CSSPseudoElementType::backdrop,
                    PseudoElement::Cue => CSSPseudoElementType::cue,
                    PseudoElement::FirstLetter => CSSPseudoElementType::firstLetter,
                    PseudoElement::FirstLine => CSSPseudoElementType::firstLine,
                    PseudoElement::MozSelection => CSSPseudoElementType::mozSelection,
                    PseudoElement::MozFocusInner => CSSPseudoElementType::mozFocusInner,
                    PseudoElement::MozFocusOuter => CSSPseudoElementType::mozFocusOuter,
                    PseudoElement::MozListBullet => CSSPseudoElementType::mozListBullet,
                    PseudoElement::MozListNumber => CSSPseudoElementType::mozListNumber,
                    PseudoElement::MozMathAnonymous => CSSPseudoElementType::mozMathAnonymous,
                    PseudoElement::MozNumberWrapper => CSSPseudoElementType::mozNumberWrapper,
                    PseudoElement::MozNumberText => CSSPseudoElementType::mozNumberText,
                    PseudoElement::MozNumberSpinBox => CSSPseudoElementType::mozNumberSpinBox,
                    PseudoElement::MozNumberSpinUp => CSSPseudoElementType::mozNumberSpinUp,
                    PseudoElement::MozNumberSpinDown => CSSPseudoElementType::mozNumberSpinDown,
                    PseudoElement::MozProgressBar => CSSPseudoElementType::mozProgressBar,
                    PseudoElement::MozRangeTrack => CSSPseudoElementType::mozRangeTrack,
                    PseudoElement::MozRangeProgress => CSSPseudoElementType::mozRangeProgress,
                    PseudoElement::MozRangeThumb => CSSPseudoElementType::mozRangeThumb,
                    PseudoElement::MozMeterBar => CSSPseudoElementType::mozMeterBar,
                    PseudoElement::MozPlaceholder => CSSPseudoElementType::mozPlaceholder,
                    PseudoElement::Placeholder => CSSPseudoElementType::placeholder,
                    PseudoElement::MozColorSwatch => CSSPseudoElementType::mozColorSwatch,
                    PseudoElement::MozText => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::OofPlaceholder => CSSPseudoElementType::NonInheritingAnonBox,
                    PseudoElement::FirstLetterContinuation => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozBlockInsideInlineWrapper => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozMathMLAnonymousBlock => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozXULAnonymousBlock => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::HorizontalFramesetBorder => CSSPseudoElementType::NonInheritingAnonBox,
                    PseudoElement::VerticalFramesetBorder => CSSPseudoElementType::NonInheritingAnonBox,
                    PseudoElement::MozLineFrame => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::ButtonContent => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::CellContent => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::DropDownList => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::FieldsetContent => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::FramesetBlank => CSSPseudoElementType::NonInheritingAnonBox,
                    PseudoElement::MozDisplayComboboxControlFrame => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::HtmlCanvasContent => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::InlineTable => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::Table => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::TableCell => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::TableColGroup => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::TableCol => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::TableWrapper => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::TableRowGroup => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::TableRow => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::Canvas => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::PageBreak => CSSPseudoElementType::NonInheritingAnonBox,
                    PseudoElement::Page => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::PageContent => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::PageSequence => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::ScrolledContent => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::ScrolledCanvas => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::ScrolledPageSequence => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::ColumnContent => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::Viewport => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::ViewportScroll => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::AnonymousFlexItem => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::AnonymousGridItem => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::Ruby => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::RubyBase => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::RubyBaseContainer => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::RubyText => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::RubyTextContainer => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeColumn(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeRow(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeSeparator(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeCell(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeIndentation(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeLine(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeTwisty(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeImage(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeCellText(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeCheckbox(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeProgressmeter(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozTreeDropFeedback(..) => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozSVGMarkerAnonChild => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozSVGOuterSVGAnonChild => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozSVGForeignContent => CSSPseudoElementType_InheritingAnonBox,
                    PseudoElement::MozSVGText => CSSPseudoElementType_InheritingAnonBox,
        }
    }

    /// Get a PseudoInfo for a pseudo
    pub fn pseudo_info(&self) -> (*mut structs::nsIAtom, CSSPseudoElementType) {
        (self.atom().as_ptr(), self.pseudo_type())
    }

    /// Construct a pseudo-element from an `Atom`.
    #[inline]
    pub fn from_atom(atom: &Atom) -> Option<Self> {
                if atom == &atom!(":after") {
                    return Some(PseudoElement::After);
                }
                if atom == &atom!(":before") {
                    return Some(PseudoElement::Before);
                }
                if atom == &atom!(":backdrop") {
                    return Some(PseudoElement::Backdrop);
                }
                if atom == &atom!(":cue") {
                    return Some(PseudoElement::Cue);
                }
                if atom == &atom!(":first-letter") {
                    return Some(PseudoElement::FirstLetter);
                }
                if atom == &atom!(":first-line") {
                    return Some(PseudoElement::FirstLine);
                }
                if atom == &atom!(":-moz-selection") {
                    return Some(PseudoElement::MozSelection);
                }
                if atom == &atom!(":-moz-focus-inner") {
                    return Some(PseudoElement::MozFocusInner);
                }
                if atom == &atom!(":-moz-focus-outer") {
                    return Some(PseudoElement::MozFocusOuter);
                }
                if atom == &atom!(":-moz-list-bullet") {
                    return Some(PseudoElement::MozListBullet);
                }
                if atom == &atom!(":-moz-list-number") {
                    return Some(PseudoElement::MozListNumber);
                }
                if atom == &atom!(":-moz-math-anonymous") {
                    return Some(PseudoElement::MozMathAnonymous);
                }
                if atom == &atom!(":-moz-number-wrapper") {
                    return Some(PseudoElement::MozNumberWrapper);
                }
                if atom == &atom!(":-moz-number-text") {
                    return Some(PseudoElement::MozNumberText);
                }
                if atom == &atom!(":-moz-number-spin-box") {
                    return Some(PseudoElement::MozNumberSpinBox);
                }
                if atom == &atom!(":-moz-number-spin-up") {
                    return Some(PseudoElement::MozNumberSpinUp);
                }
                if atom == &atom!(":-moz-number-spin-down") {
                    return Some(PseudoElement::MozNumberSpinDown);
                }
                if atom == &atom!(":-moz-progress-bar") {
                    return Some(PseudoElement::MozProgressBar);
                }
                if atom == &atom!(":-moz-range-track") {
                    return Some(PseudoElement::MozRangeTrack);
                }
                if atom == &atom!(":-moz-range-progress") {
                    return Some(PseudoElement::MozRangeProgress);
                }
                if atom == &atom!(":-moz-range-thumb") {
                    return Some(PseudoElement::MozRangeThumb);
                }
                if atom == &atom!(":-moz-meter-bar") {
                    return Some(PseudoElement::MozMeterBar);
                }
                if atom == &atom!(":-moz-placeholder") {
                    return Some(PseudoElement::MozPlaceholder);
                }
                if atom == &atom!(":placeholder") {
                    return Some(PseudoElement::Placeholder);
                }
                if atom == &atom!(":-moz-color-swatch") {
                    return Some(PseudoElement::MozColorSwatch);
                }
                if atom == &atom!(":-moz-text") {
                    return Some(PseudoElement::MozText);
                }
                if atom == &atom!(":-moz-oof-placeholder") {
                    return Some(PseudoElement::OofPlaceholder);
                }
                if atom == &atom!(":-moz-first-letter-continuation") {
                    return Some(PseudoElement::FirstLetterContinuation);
                }
                if atom == &atom!(":-moz-block-inside-inline-wrapper") {
                    return Some(PseudoElement::MozBlockInsideInlineWrapper);
                }
                if atom == &atom!(":-moz-mathml-anonymous-block") {
                    return Some(PseudoElement::MozMathMLAnonymousBlock);
                }
                if atom == &atom!(":-moz-xul-anonymous-block") {
                    return Some(PseudoElement::MozXULAnonymousBlock);
                }
                if atom == &atom!(":-moz-hframeset-border") {
                    return Some(PseudoElement::HorizontalFramesetBorder);
                }
                if atom == &atom!(":-moz-vframeset-border") {
                    return Some(PseudoElement::VerticalFramesetBorder);
                }
                if atom == &atom!(":-moz-line-frame") {
                    return Some(PseudoElement::MozLineFrame);
                }
                if atom == &atom!(":-moz-button-content") {
                    return Some(PseudoElement::ButtonContent);
                }
                if atom == &atom!(":-moz-cell-content") {
                    return Some(PseudoElement::CellContent);
                }
                if atom == &atom!(":-moz-dropdown-list") {
                    return Some(PseudoElement::DropDownList);
                }
                if atom == &atom!(":-moz-fieldset-content") {
                    return Some(PseudoElement::FieldsetContent);
                }
                if atom == &atom!(":-moz-frameset-blank") {
                    return Some(PseudoElement::FramesetBlank);
                }
                if atom == &atom!(":-moz-display-comboboxcontrol-frame") {
                    return Some(PseudoElement::MozDisplayComboboxControlFrame);
                }
                if atom == &atom!(":-moz-html-canvas-content") {
                    return Some(PseudoElement::HtmlCanvasContent);
                }
                if atom == &atom!(":-moz-inline-table") {
                    return Some(PseudoElement::InlineTable);
                }
                if atom == &atom!(":-moz-table") {
                    return Some(PseudoElement::Table);
                }
                if atom == &atom!(":-moz-table-cell") {
                    return Some(PseudoElement::TableCell);
                }
                if atom == &atom!(":-moz-table-column-group") {
                    return Some(PseudoElement::TableColGroup);
                }
                if atom == &atom!(":-moz-table-column") {
                    return Some(PseudoElement::TableCol);
                }
                if atom == &atom!(":-moz-table-wrapper") {
                    return Some(PseudoElement::TableWrapper);
                }
                if atom == &atom!(":-moz-table-row-group") {
                    return Some(PseudoElement::TableRowGroup);
                }
                if atom == &atom!(":-moz-table-row") {
                    return Some(PseudoElement::TableRow);
                }
                if atom == &atom!(":-moz-canvas") {
                    return Some(PseudoElement::Canvas);
                }
                if atom == &atom!(":-moz-pagebreak") {
                    return Some(PseudoElement::PageBreak);
                }
                if atom == &atom!(":-moz-page") {
                    return Some(PseudoElement::Page);
                }
                if atom == &atom!(":-moz-pagecontent") {
                    return Some(PseudoElement::PageContent);
                }
                if atom == &atom!(":-moz-page-sequence") {
                    return Some(PseudoElement::PageSequence);
                }
                if atom == &atom!(":-moz-scrolled-content") {
                    return Some(PseudoElement::ScrolledContent);
                }
                if atom == &atom!(":-moz-scrolled-canvas") {
                    return Some(PseudoElement::ScrolledCanvas);
                }
                if atom == &atom!(":-moz-scrolled-page-sequence") {
                    return Some(PseudoElement::ScrolledPageSequence);
                }
                if atom == &atom!(":-moz-column-content") {
                    return Some(PseudoElement::ColumnContent);
                }
                if atom == &atom!(":-moz-viewport") {
                    return Some(PseudoElement::Viewport);
                }
                if atom == &atom!(":-moz-viewport-scroll") {
                    return Some(PseudoElement::ViewportScroll);
                }
                if atom == &atom!(":-moz-anonymous-flex-item") {
                    return Some(PseudoElement::AnonymousFlexItem);
                }
                if atom == &atom!(":-moz-anonymous-grid-item") {
                    return Some(PseudoElement::AnonymousGridItem);
                }
                if atom == &atom!(":-moz-ruby") {
                    return Some(PseudoElement::Ruby);
                }
                if atom == &atom!(":-moz-ruby-base") {
                    return Some(PseudoElement::RubyBase);
                }
                if atom == &atom!(":-moz-ruby-base-container") {
                    return Some(PseudoElement::RubyBaseContainer);
                }
                if atom == &atom!(":-moz-ruby-text") {
                    return Some(PseudoElement::RubyText);
                }
                if atom == &atom!(":-moz-ruby-text-container") {
                    return Some(PseudoElement::RubyTextContainer);
                }
                // We cannot generate PseudoElement::MozTreeColumn(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeRow(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeSeparator(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeCell(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeIndentation(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeLine(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeTwisty(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeImage(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeCellText(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeCheckbox(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeProgressmeter(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeDropFeedback(..) from just an atom.
                if atom == &atom!(":-moz-svg-marker-anon-child") {
                    return Some(PseudoElement::MozSVGMarkerAnonChild);
                }
                if atom == &atom!(":-moz-svg-outer-svg-anon-child") {
                    return Some(PseudoElement::MozSVGOuterSVGAnonChild);
                }
                if atom == &atom!(":-moz-svg-foreign-content") {
                    return Some(PseudoElement::MozSVGForeignContent);
                }
                if atom == &atom!(":-moz-svg-text") {
                    return Some(PseudoElement::MozSVGText);
                }
        None
    }

    /// Construct a pseudo-element from an anonymous box `Atom`.
    #[inline]
    pub fn from_anon_box_atom(atom: &Atom) -> Option<Self> {
                if atom == &atom!(":-moz-text") {
                    return Some(PseudoElement::MozText);
                }
                if atom == &atom!(":-moz-oof-placeholder") {
                    return Some(PseudoElement::OofPlaceholder);
                }
                if atom == &atom!(":-moz-first-letter-continuation") {
                    return Some(PseudoElement::FirstLetterContinuation);
                }
                if atom == &atom!(":-moz-block-inside-inline-wrapper") {
                    return Some(PseudoElement::MozBlockInsideInlineWrapper);
                }
                if atom == &atom!(":-moz-mathml-anonymous-block") {
                    return Some(PseudoElement::MozMathMLAnonymousBlock);
                }
                if atom == &atom!(":-moz-xul-anonymous-block") {
                    return Some(PseudoElement::MozXULAnonymousBlock);
                }
                if atom == &atom!(":-moz-hframeset-border") {
                    return Some(PseudoElement::HorizontalFramesetBorder);
                }
                if atom == &atom!(":-moz-vframeset-border") {
                    return Some(PseudoElement::VerticalFramesetBorder);
                }
                if atom == &atom!(":-moz-line-frame") {
                    return Some(PseudoElement::MozLineFrame);
                }
                if atom == &atom!(":-moz-button-content") {
                    return Some(PseudoElement::ButtonContent);
                }
                if atom == &atom!(":-moz-cell-content") {
                    return Some(PseudoElement::CellContent);
                }
                if atom == &atom!(":-moz-dropdown-list") {
                    return Some(PseudoElement::DropDownList);
                }
                if atom == &atom!(":-moz-fieldset-content") {
                    return Some(PseudoElement::FieldsetContent);
                }
                if atom == &atom!(":-moz-frameset-blank") {
                    return Some(PseudoElement::FramesetBlank);
                }
                if atom == &atom!(":-moz-display-comboboxcontrol-frame") {
                    return Some(PseudoElement::MozDisplayComboboxControlFrame);
                }
                if atom == &atom!(":-moz-html-canvas-content") {
                    return Some(PseudoElement::HtmlCanvasContent);
                }
                if atom == &atom!(":-moz-inline-table") {
                    return Some(PseudoElement::InlineTable);
                }
                if atom == &atom!(":-moz-table") {
                    return Some(PseudoElement::Table);
                }
                if atom == &atom!(":-moz-table-cell") {
                    return Some(PseudoElement::TableCell);
                }
                if atom == &atom!(":-moz-table-column-group") {
                    return Some(PseudoElement::TableColGroup);
                }
                if atom == &atom!(":-moz-table-column") {
                    return Some(PseudoElement::TableCol);
                }
                if atom == &atom!(":-moz-table-wrapper") {
                    return Some(PseudoElement::TableWrapper);
                }
                if atom == &atom!(":-moz-table-row-group") {
                    return Some(PseudoElement::TableRowGroup);
                }
                if atom == &atom!(":-moz-table-row") {
                    return Some(PseudoElement::TableRow);
                }
                if atom == &atom!(":-moz-canvas") {
                    return Some(PseudoElement::Canvas);
                }
                if atom == &atom!(":-moz-pagebreak") {
                    return Some(PseudoElement::PageBreak);
                }
                if atom == &atom!(":-moz-page") {
                    return Some(PseudoElement::Page);
                }
                if atom == &atom!(":-moz-pagecontent") {
                    return Some(PseudoElement::PageContent);
                }
                if atom == &atom!(":-moz-page-sequence") {
                    return Some(PseudoElement::PageSequence);
                }
                if atom == &atom!(":-moz-scrolled-content") {
                    return Some(PseudoElement::ScrolledContent);
                }
                if atom == &atom!(":-moz-scrolled-canvas") {
                    return Some(PseudoElement::ScrolledCanvas);
                }
                if atom == &atom!(":-moz-scrolled-page-sequence") {
                    return Some(PseudoElement::ScrolledPageSequence);
                }
                if atom == &atom!(":-moz-column-content") {
                    return Some(PseudoElement::ColumnContent);
                }
                if atom == &atom!(":-moz-viewport") {
                    return Some(PseudoElement::Viewport);
                }
                if atom == &atom!(":-moz-viewport-scroll") {
                    return Some(PseudoElement::ViewportScroll);
                }
                if atom == &atom!(":-moz-anonymous-flex-item") {
                    return Some(PseudoElement::AnonymousFlexItem);
                }
                if atom == &atom!(":-moz-anonymous-grid-item") {
                    return Some(PseudoElement::AnonymousGridItem);
                }
                if atom == &atom!(":-moz-ruby") {
                    return Some(PseudoElement::Ruby);
                }
                if atom == &atom!(":-moz-ruby-base") {
                    return Some(PseudoElement::RubyBase);
                }
                if atom == &atom!(":-moz-ruby-base-container") {
                    return Some(PseudoElement::RubyBaseContainer);
                }
                if atom == &atom!(":-moz-ruby-text") {
                    return Some(PseudoElement::RubyText);
                }
                if atom == &atom!(":-moz-ruby-text-container") {
                    return Some(PseudoElement::RubyTextContainer);
                }
                // We cannot generate PseudoElement::MozTreeColumn(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeRow(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeSeparator(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeCell(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeIndentation(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeLine(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeTwisty(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeImage(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeCellText(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeCheckbox(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeProgressmeter(..) from just an atom.
                // We cannot generate PseudoElement::MozTreeDropFeedback(..) from just an atom.
                if atom == &atom!(":-moz-svg-marker-anon-child") {
                    return Some(PseudoElement::MozSVGMarkerAnonChild);
                }
                if atom == &atom!(":-moz-svg-outer-svg-anon-child") {
                    return Some(PseudoElement::MozSVGOuterSVGAnonChild);
                }
                if atom == &atom!(":-moz-svg-foreign-content") {
                    return Some(PseudoElement::MozSVGForeignContent);
                }
                if atom == &atom!(":-moz-svg-text") {
                    return Some(PseudoElement::MozSVGText);
                }
        None
    }

    /// Constructs an atom from a string of text, and whether we're in a
    /// user-agent stylesheet.
    ///
    /// If we're not in a user-agent stylesheet, we will never parse anonymous
    /// box pseudo-elements.
    ///
    /// Returns `None` if the pseudo-element is not recognised.
    #[inline]
    pub fn from_slice(s: &str, in_ua_stylesheet: bool) -> Option<Self> {
        use std::ascii::AsciiExt;

        // We don't need to support tree pseudos because functional
        // pseudo-elements needs arguments, and thus should be created
        // via other methods.
            if in_ua_stylesheet || PseudoElement::After.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("after") {
                    return Some(PseudoElement::After);
                }
            }
            if in_ua_stylesheet || PseudoElement::Before.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("before") {
                    return Some(PseudoElement::Before);
                }
            }
            if in_ua_stylesheet || PseudoElement::Backdrop.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("backdrop") {
                    return Some(PseudoElement::Backdrop);
                }
            }
            if in_ua_stylesheet || PseudoElement::Cue.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("cue") {
                    return Some(PseudoElement::Cue);
                }
            }
            if in_ua_stylesheet || PseudoElement::FirstLetter.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("first-letter") {
                    return Some(PseudoElement::FirstLetter);
                }
            }
            if in_ua_stylesheet || PseudoElement::FirstLine.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("first-line") {
                    return Some(PseudoElement::FirstLine);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSelection.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-selection") {
                    return Some(PseudoElement::MozSelection);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozFocusInner.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-focus-inner") {
                    return Some(PseudoElement::MozFocusInner);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozFocusOuter.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-focus-outer") {
                    return Some(PseudoElement::MozFocusOuter);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozListBullet.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-list-bullet") {
                    return Some(PseudoElement::MozListBullet);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozListNumber.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-list-number") {
                    return Some(PseudoElement::MozListNumber);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozMathAnonymous.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-math-anonymous") {
                    return Some(PseudoElement::MozMathAnonymous);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberWrapper.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-wrapper") {
                    return Some(PseudoElement::MozNumberWrapper);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberText.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-text") {
                    return Some(PseudoElement::MozNumberText);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberSpinBox.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-spin-box") {
                    return Some(PseudoElement::MozNumberSpinBox);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberSpinUp.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-spin-up") {
                    return Some(PseudoElement::MozNumberSpinUp);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozNumberSpinDown.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-number-spin-down") {
                    return Some(PseudoElement::MozNumberSpinDown);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozProgressBar.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-progress-bar") {
                    return Some(PseudoElement::MozProgressBar);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozRangeTrack.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-range-track") {
                    return Some(PseudoElement::MozRangeTrack);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozRangeProgress.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-range-progress") {
                    return Some(PseudoElement::MozRangeProgress);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozRangeThumb.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-range-thumb") {
                    return Some(PseudoElement::MozRangeThumb);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozMeterBar.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-meter-bar") {
                    return Some(PseudoElement::MozMeterBar);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozPlaceholder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-placeholder") {
                    return Some(PseudoElement::MozPlaceholder);
                }
            }
            if in_ua_stylesheet || PseudoElement::Placeholder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("placeholder") {
                    return Some(PseudoElement::Placeholder);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozColorSwatch.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-color-swatch") {
                    return Some(PseudoElement::MozColorSwatch);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozText.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-text") {
                    return Some(PseudoElement::MozText);
                }
            }
            if in_ua_stylesheet || PseudoElement::OofPlaceholder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-oof-placeholder") {
                    return Some(PseudoElement::OofPlaceholder);
                }
            }
            if in_ua_stylesheet || PseudoElement::FirstLetterContinuation.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-first-letter-continuation") {
                    return Some(PseudoElement::FirstLetterContinuation);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozBlockInsideInlineWrapper.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-block-inside-inline-wrapper") {
                    return Some(PseudoElement::MozBlockInsideInlineWrapper);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozMathMLAnonymousBlock.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-mathml-anonymous-block") {
                    return Some(PseudoElement::MozMathMLAnonymousBlock);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozXULAnonymousBlock.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-xul-anonymous-block") {
                    return Some(PseudoElement::MozXULAnonymousBlock);
                }
            }
            if in_ua_stylesheet || PseudoElement::HorizontalFramesetBorder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-hframeset-border") {
                    return Some(PseudoElement::HorizontalFramesetBorder);
                }
            }
            if in_ua_stylesheet || PseudoElement::VerticalFramesetBorder.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-vframeset-border") {
                    return Some(PseudoElement::VerticalFramesetBorder);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozLineFrame.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-line-frame") {
                    return Some(PseudoElement::MozLineFrame);
                }
            }
            if in_ua_stylesheet || PseudoElement::ButtonContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-button-content") {
                    return Some(PseudoElement::ButtonContent);
                }
            }
            if in_ua_stylesheet || PseudoElement::CellContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-cell-content") {
                    return Some(PseudoElement::CellContent);
                }
            }
            if in_ua_stylesheet || PseudoElement::DropDownList.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-dropdown-list") {
                    return Some(PseudoElement::DropDownList);
                }
            }
            if in_ua_stylesheet || PseudoElement::FieldsetContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-fieldset-content") {
                    return Some(PseudoElement::FieldsetContent);
                }
            }
            if in_ua_stylesheet || PseudoElement::FramesetBlank.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-frameset-blank") {
                    return Some(PseudoElement::FramesetBlank);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozDisplayComboboxControlFrame.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-display-comboboxcontrol-frame") {
                    return Some(PseudoElement::MozDisplayComboboxControlFrame);
                }
            }
            if in_ua_stylesheet || PseudoElement::HtmlCanvasContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-html-canvas-content") {
                    return Some(PseudoElement::HtmlCanvasContent);
                }
            }
            if in_ua_stylesheet || PseudoElement::InlineTable.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-inline-table") {
                    return Some(PseudoElement::InlineTable);
                }
            }
            if in_ua_stylesheet || PseudoElement::Table.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table") {
                    return Some(PseudoElement::Table);
                }
            }
            if in_ua_stylesheet || PseudoElement::TableCell.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-cell") {
                    return Some(PseudoElement::TableCell);
                }
            }
            if in_ua_stylesheet || PseudoElement::TableColGroup.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-column-group") {
                    return Some(PseudoElement::TableColGroup);
                }
            }
            if in_ua_stylesheet || PseudoElement::TableCol.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-column") {
                    return Some(PseudoElement::TableCol);
                }
            }
            if in_ua_stylesheet || PseudoElement::TableWrapper.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-wrapper") {
                    return Some(PseudoElement::TableWrapper);
                }
            }
            if in_ua_stylesheet || PseudoElement::TableRowGroup.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-row-group") {
                    return Some(PseudoElement::TableRowGroup);
                }
            }
            if in_ua_stylesheet || PseudoElement::TableRow.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-table-row") {
                    return Some(PseudoElement::TableRow);
                }
            }
            if in_ua_stylesheet || PseudoElement::Canvas.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-canvas") {
                    return Some(PseudoElement::Canvas);
                }
            }
            if in_ua_stylesheet || PseudoElement::PageBreak.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-pagebreak") {
                    return Some(PseudoElement::PageBreak);
                }
            }
            if in_ua_stylesheet || PseudoElement::Page.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-page") {
                    return Some(PseudoElement::Page);
                }
            }
            if in_ua_stylesheet || PseudoElement::PageContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-pagecontent") {
                    return Some(PseudoElement::PageContent);
                }
            }
            if in_ua_stylesheet || PseudoElement::PageSequence.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-page-sequence") {
                    return Some(PseudoElement::PageSequence);
                }
            }
            if in_ua_stylesheet || PseudoElement::ScrolledContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-scrolled-content") {
                    return Some(PseudoElement::ScrolledContent);
                }
            }
            if in_ua_stylesheet || PseudoElement::ScrolledCanvas.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-scrolled-canvas") {
                    return Some(PseudoElement::ScrolledCanvas);
                }
            }
            if in_ua_stylesheet || PseudoElement::ScrolledPageSequence.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-scrolled-page-sequence") {
                    return Some(PseudoElement::ScrolledPageSequence);
                }
            }
            if in_ua_stylesheet || PseudoElement::ColumnContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-column-content") {
                    return Some(PseudoElement::ColumnContent);
                }
            }
            if in_ua_stylesheet || PseudoElement::Viewport.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-viewport") {
                    return Some(PseudoElement::Viewport);
                }
            }
            if in_ua_stylesheet || PseudoElement::ViewportScroll.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-viewport-scroll") {
                    return Some(PseudoElement::ViewportScroll);
                }
            }
            if in_ua_stylesheet || PseudoElement::AnonymousFlexItem.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-anonymous-flex-item") {
                    return Some(PseudoElement::AnonymousFlexItem);
                }
            }
            if in_ua_stylesheet || PseudoElement::AnonymousGridItem.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-anonymous-grid-item") {
                    return Some(PseudoElement::AnonymousGridItem);
                }
            }
            if in_ua_stylesheet || PseudoElement::Ruby.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby") {
                    return Some(PseudoElement::Ruby);
                }
            }
            if in_ua_stylesheet || PseudoElement::RubyBase.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby-base") {
                    return Some(PseudoElement::RubyBase);
                }
            }
            if in_ua_stylesheet || PseudoElement::RubyBaseContainer.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby-base-container") {
                    return Some(PseudoElement::RubyBaseContainer);
                }
            }
            if in_ua_stylesheet || PseudoElement::RubyText.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby-text") {
                    return Some(PseudoElement::RubyText);
                }
            }
            if in_ua_stylesheet || PseudoElement::RubyTextContainer.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-ruby-text-container") {
                    return Some(PseudoElement::RubyTextContainer);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSVGMarkerAnonChild.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-svg-marker-anon-child") {
                    return Some(PseudoElement::MozSVGMarkerAnonChild);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSVGOuterSVGAnonChild.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-svg-outer-svg-anon-child") {
                    return Some(PseudoElement::MozSVGOuterSVGAnonChild);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSVGForeignContent.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-svg-foreign-content") {
                    return Some(PseudoElement::MozSVGForeignContent);
                }
            }
            if in_ua_stylesheet || PseudoElement::MozSVGText.exposed_in_non_ua_sheets() {
                if s.eq_ignore_ascii_case("-moz-svg-text") {
                    return Some(PseudoElement::MozSVGText);
                }
            }

        None
    }

    /// Constructs a tree pseudo-element from the given name and arguments.
    /// "name" must start with "-moz-tree-".
    ///
    /// Returns `None` if the pseudo-element is not recognized.
    #[inline]
    pub fn tree_pseudo_element(name: &str, args: Box<[String]>) -> Option<Self> {
        use std::ascii::AsciiExt;
        debug_assert!(name.starts_with("-moz-tree-"));
        let tree_part = &name[10..];
            if tree_part.eq_ignore_ascii_case("column") {
                return Some(PseudoElement::MozTreeColumn(args));
            }
            if tree_part.eq_ignore_ascii_case("row") {
                return Some(PseudoElement::MozTreeRow(args));
            }
            if tree_part.eq_ignore_ascii_case("separator") {
                return Some(PseudoElement::MozTreeSeparator(args));
            }
            if tree_part.eq_ignore_ascii_case("cell") {
                return Some(PseudoElement::MozTreeCell(args));
            }
            if tree_part.eq_ignore_ascii_case("indentation") {
                return Some(PseudoElement::MozTreeIndentation(args));
            }
            if tree_part.eq_ignore_ascii_case("line") {
                return Some(PseudoElement::MozTreeLine(args));
            }
            if tree_part.eq_ignore_ascii_case("twisty") {
                return Some(PseudoElement::MozTreeTwisty(args));
            }
            if tree_part.eq_ignore_ascii_case("image") {
                return Some(PseudoElement::MozTreeImage(args));
            }
            if tree_part.eq_ignore_ascii_case("cell-text") {
                return Some(PseudoElement::MozTreeCellText(args));
            }
            if tree_part.eq_ignore_ascii_case("checkbox") {
                return Some(PseudoElement::MozTreeCheckbox(args));
            }
            if tree_part.eq_ignore_ascii_case("progressmeter") {
                return Some(PseudoElement::MozTreeProgressmeter(args));
            }
            if tree_part.eq_ignore_ascii_case("drop-feedback") {
                return Some(PseudoElement::MozTreeDropFeedback(args));
            }
        None
    }
}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_char(':')?;
        match *self {
                PseudoElement::After => dest.write_str(":after")?,
                PseudoElement::Before => dest.write_str(":before")?,
                PseudoElement::Backdrop => dest.write_str(":backdrop")?,
                PseudoElement::Cue => dest.write_str(":cue")?,
                PseudoElement::FirstLetter => dest.write_str(":first-letter")?,
                PseudoElement::FirstLine => dest.write_str(":first-line")?,
                PseudoElement::MozSelection => dest.write_str(":-moz-selection")?,
                PseudoElement::MozFocusInner => dest.write_str(":-moz-focus-inner")?,
                PseudoElement::MozFocusOuter => dest.write_str(":-moz-focus-outer")?,
                PseudoElement::MozListBullet => dest.write_str(":-moz-list-bullet")?,
                PseudoElement::MozListNumber => dest.write_str(":-moz-list-number")?,
                PseudoElement::MozMathAnonymous => dest.write_str(":-moz-math-anonymous")?,
                PseudoElement::MozNumberWrapper => dest.write_str(":-moz-number-wrapper")?,
                PseudoElement::MozNumberText => dest.write_str(":-moz-number-text")?,
                PseudoElement::MozNumberSpinBox => dest.write_str(":-moz-number-spin-box")?,
                PseudoElement::MozNumberSpinUp => dest.write_str(":-moz-number-spin-up")?,
                PseudoElement::MozNumberSpinDown => dest.write_str(":-moz-number-spin-down")?,
                PseudoElement::MozProgressBar => dest.write_str(":-moz-progress-bar")?,
                PseudoElement::MozRangeTrack => dest.write_str(":-moz-range-track")?,
                PseudoElement::MozRangeProgress => dest.write_str(":-moz-range-progress")?,
                PseudoElement::MozRangeThumb => dest.write_str(":-moz-range-thumb")?,
                PseudoElement::MozMeterBar => dest.write_str(":-moz-meter-bar")?,
                PseudoElement::MozPlaceholder => dest.write_str(":-moz-placeholder")?,
                PseudoElement::Placeholder => dest.write_str(":placeholder")?,
                PseudoElement::MozColorSwatch => dest.write_str(":-moz-color-swatch")?,
                PseudoElement::MozText => dest.write_str(":-moz-text")?,
                PseudoElement::OofPlaceholder => dest.write_str(":-moz-oof-placeholder")?,
                PseudoElement::FirstLetterContinuation => dest.write_str(":-moz-first-letter-continuation")?,
                PseudoElement::MozBlockInsideInlineWrapper => dest.write_str(":-moz-block-inside-inline-wrapper")?,
                PseudoElement::MozMathMLAnonymousBlock => dest.write_str(":-moz-mathml-anonymous-block")?,
                PseudoElement::MozXULAnonymousBlock => dest.write_str(":-moz-xul-anonymous-block")?,
                PseudoElement::HorizontalFramesetBorder => dest.write_str(":-moz-hframeset-border")?,
                PseudoElement::VerticalFramesetBorder => dest.write_str(":-moz-vframeset-border")?,
                PseudoElement::MozLineFrame => dest.write_str(":-moz-line-frame")?,
                PseudoElement::ButtonContent => dest.write_str(":-moz-button-content")?,
                PseudoElement::CellContent => dest.write_str(":-moz-cell-content")?,
                PseudoElement::DropDownList => dest.write_str(":-moz-dropdown-list")?,
                PseudoElement::FieldsetContent => dest.write_str(":-moz-fieldset-content")?,
                PseudoElement::FramesetBlank => dest.write_str(":-moz-frameset-blank")?,
                PseudoElement::MozDisplayComboboxControlFrame => dest.write_str(":-moz-display-comboboxcontrol-frame")?,
                PseudoElement::HtmlCanvasContent => dest.write_str(":-moz-html-canvas-content")?,
                PseudoElement::InlineTable => dest.write_str(":-moz-inline-table")?,
                PseudoElement::Table => dest.write_str(":-moz-table")?,
                PseudoElement::TableCell => dest.write_str(":-moz-table-cell")?,
                PseudoElement::TableColGroup => dest.write_str(":-moz-table-column-group")?,
                PseudoElement::TableCol => dest.write_str(":-moz-table-column")?,
                PseudoElement::TableWrapper => dest.write_str(":-moz-table-wrapper")?,
                PseudoElement::TableRowGroup => dest.write_str(":-moz-table-row-group")?,
                PseudoElement::TableRow => dest.write_str(":-moz-table-row")?,
                PseudoElement::Canvas => dest.write_str(":-moz-canvas")?,
                PseudoElement::PageBreak => dest.write_str(":-moz-pagebreak")?,
                PseudoElement::Page => dest.write_str(":-moz-page")?,
                PseudoElement::PageContent => dest.write_str(":-moz-pagecontent")?,
                PseudoElement::PageSequence => dest.write_str(":-moz-page-sequence")?,
                PseudoElement::ScrolledContent => dest.write_str(":-moz-scrolled-content")?,
                PseudoElement::ScrolledCanvas => dest.write_str(":-moz-scrolled-canvas")?,
                PseudoElement::ScrolledPageSequence => dest.write_str(":-moz-scrolled-page-sequence")?,
                PseudoElement::ColumnContent => dest.write_str(":-moz-column-content")?,
                PseudoElement::Viewport => dest.write_str(":-moz-viewport")?,
                PseudoElement::ViewportScroll => dest.write_str(":-moz-viewport-scroll")?,
                PseudoElement::AnonymousFlexItem => dest.write_str(":-moz-anonymous-flex-item")?,
                PseudoElement::AnonymousGridItem => dest.write_str(":-moz-anonymous-grid-item")?,
                PseudoElement::Ruby => dest.write_str(":-moz-ruby")?,
                PseudoElement::RubyBase => dest.write_str(":-moz-ruby-base")?,
                PseudoElement::RubyBaseContainer => dest.write_str(":-moz-ruby-base-container")?,
                PseudoElement::RubyText => dest.write_str(":-moz-ruby-text")?,
                PseudoElement::RubyTextContainer => dest.write_str(":-moz-ruby-text-container")?,
                PseudoElement::MozTreeColumn(..) => dest.write_str(":-moz-tree-column")?,
                PseudoElement::MozTreeRow(..) => dest.write_str(":-moz-tree-row")?,
                PseudoElement::MozTreeSeparator(..) => dest.write_str(":-moz-tree-separator")?,
                PseudoElement::MozTreeCell(..) => dest.write_str(":-moz-tree-cell")?,
                PseudoElement::MozTreeIndentation(..) => dest.write_str(":-moz-tree-indentation")?,
                PseudoElement::MozTreeLine(..) => dest.write_str(":-moz-tree-line")?,
                PseudoElement::MozTreeTwisty(..) => dest.write_str(":-moz-tree-twisty")?,
                PseudoElement::MozTreeImage(..) => dest.write_str(":-moz-tree-image")?,
                PseudoElement::MozTreeCellText(..) => dest.write_str(":-moz-tree-cell-text")?,
                PseudoElement::MozTreeCheckbox(..) => dest.write_str(":-moz-tree-checkbox")?,
                PseudoElement::MozTreeProgressmeter(..) => dest.write_str(":-moz-tree-progressmeter")?,
                PseudoElement::MozTreeDropFeedback(..) => dest.write_str(":-moz-tree-drop-feedback")?,
                PseudoElement::MozSVGMarkerAnonChild => dest.write_str(":-moz-svg-marker-anon-child")?,
                PseudoElement::MozSVGOuterSVGAnonChild => dest.write_str(":-moz-svg-outer-svg-anon-child")?,
                PseudoElement::MozSVGForeignContent => dest.write_str(":-moz-svg-foreign-content")?,
                PseudoElement::MozSVGText => dest.write_str(":-moz-svg-text")?,
        }
        match *self {
            PseudoElement::MozTreeColumn(ref args) |
            PseudoElement::MozTreeRow(ref args) |
            PseudoElement::MozTreeSeparator(ref args) |
            PseudoElement::MozTreeCell(ref args) |
            PseudoElement::MozTreeIndentation(ref args) |
            PseudoElement::MozTreeLine(ref args) |
            PseudoElement::MozTreeTwisty(ref args) |
            PseudoElement::MozTreeImage(ref args) |
            PseudoElement::MozTreeCellText(ref args) |
            PseudoElement::MozTreeCheckbox(ref args) |
            PseudoElement::MozTreeProgressmeter(ref args) |
            PseudoElement::MozTreeDropFeedback(ref args) => {
                dest.write_char('(')?;
                let mut iter = args.iter();
                if let Some(first) = iter.next() {
                    serialize_identifier(first, dest)?;
                    for item in iter {
                        dest.write_str(", ")?;
                        serialize_identifier(item, dest)?;
                    }
                }
                dest.write_char(')')
            }
            _ => Ok(()),
        }
    }
}

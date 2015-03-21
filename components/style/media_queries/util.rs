/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Ensures that the next token returned by the given `Parser` instance is
/// either `ParenthesisBlock` or `Function('and')``.
macro_rules! expect_and {
    ($input:ident) => {
        if $input.try(|input| input.expect_ident_matching("and")).is_ok() {
            Ok(())
        } else {
            let position = $input.position();
            let result = $input.expect_function_matching("and");
            $input.reset(position);
            result
        }
    }
}

/// Whitespace equivalent to the various `Parser::expect_*`.
macro_rules! expect_whitespace {
    ($input:ident) => {
        match try!($input.next_including_whitespace()) {
            Token::WhiteSpace(_) => Ok(()),
            _ => Err(())
        }
    }
}

/// Useful macro for debugging CSS parser issues that prints the remaining
/// tokens in a given `Parser` instance.
macro_rules! print_remaining_tokens {
    ($at:expr, $input:ident) => {
        $input.try(|input| -> Result<(),()> {
            print!("{}:", $at);
            while let Ok(token) = input.next() {
                print!(" {:?}", token);
            }
            println!("");
            Err(())
        }).is_ok();
    }
}

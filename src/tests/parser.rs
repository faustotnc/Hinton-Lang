use crate::parser::Parser;

#[test]
fn variable_reassignment_operators() {
   let src = "
      a = 0;   a += 1;  a -= 2; a *= 3;
      a /= 4;  a **= 5; a %= 6; a <<= 7;
      a >>= 8; a &= 9;  a |= 9; a ^= 9;
   ";

   if Parser::parse(src).is_err() {
      panic!("Had error with reassignment.")
   }
}

#[test]
fn reassignment_on_literal() {
   if Parser::parse("2 = 43;").is_ok() {
      panic!("Cannot assign to literal value.")
   }
}

#[test]
fn expect_colon_in_ternary_operator() {
   if Parser::parse("true ? false;").is_ok() {
      panic!("Should expect colon in ternary operator.")
   }
}

#[test]
fn allow_chained_ternary_expressions() {
   if Parser::parse("true ? false : null ? true : false;").is_err() {
      panic!("Should allow chained ternary conditional expressions.")
   }
}

#[test]
fn panic_on_unterminated_string() {
   if Parser::parse("\"hello world").is_ok() {
      panic!("Should emit error on unterminated strings.")
   }

   if Parser::parse("\'hello world").is_ok() {
      panic!("Should emit error on unterminated strings.")
   }

   if Parser::parse("\"I\"m over here!\";").is_ok() {
      panic!("Should emit error on unterminated strings.")
   }

   if Parser::parse("'I'm over here!';").is_ok() {
      panic!("Should emit error on unterminated strings.")
   }
}

#[test]
fn allow_double_quoted_strings() {
   if Parser::parse("\"I am going to the moon tomorrow.\";").is_err() {
      panic!("Should allow double-quoted strings.")
   }
}

#[test]
fn allow_escaped_double_quoted_strings() {
   if Parser::parse("\"He said told me to \\\"stay quiet\\\", yesterday.\";").is_err() {
      panic!("Should allow escaped double-quotes.")
   }
}

#[test]
fn allow_single_quoted_strings() {
   if Parser::parse("'The sky is green!';").is_err() {
      panic!("Should allow single-quoted strings.")
   }
}

#[test]
fn allow_escaped_single_quoted_strings() {
   if Parser::parse("'I\\\'m over here!';").is_err() {
      panic!("Should allow escaped single-quoted strings.")
   }
}

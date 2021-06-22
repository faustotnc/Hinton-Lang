use crate::parser::Parser;

#[test]
fn variable_reassignment_operators() {
   let src = "
        a = 0;   a += 1;  a -= 2; a *= 3;
        a /= 4;  a **= 5; a %= 6; a <<= 7;
        a >>= 8; a &= 9;  a |= 9; a ^= 9;
    ";

   if let Err(_) = Parser::parse(src) {
      panic!("Had error with reassignment.")
   }
}

#[test]
fn reassignment_on_literal() {
   if let Ok(_) = Parser::parse("2 = 43;") {
      panic!("Cannot assign to literal value.")
   }
}

#[test]
fn expect_colon_in_ternary_operator() {
   if let Ok(_) = Parser::parse("true ? false;") {
      panic!("Should expect colon in ternary operator.")
   }
}

#[test]
fn allow_chained_ternary_expressions() {
   if let Err(_) = Parser::parse("true ? false : null ? true : false;") {
      panic!("Should allow chained ternary conditional expressions.")
   }
}

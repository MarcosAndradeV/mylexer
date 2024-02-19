use super::*;


#[test]
fn test() {
    let mut lex = Lexer::new("1 + 2 * 3 asdsda\n ds".to_string().into_bytes());
    loop {
        let tok = lex.next_token();
        println!("{:?}", tok);
        println!("{}", tok);

        if matches!(tok.kind, TokenKind::Null | TokenKind::Invalid)  {
            break;
        }
    }
}
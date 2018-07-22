
pub mod lilit; // synthesized by LALRPOP

#[test]
fn lilit_test() {
    assert!(lilit::TermParser::new().parse("22").is_ok());
}

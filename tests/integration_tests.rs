use ::reee::supervisor::Supervisor;

#[test]
fn a_test() {
    let mut sv = Supervisor::new();

    assert_eq!(0, sv.num_environments());
}

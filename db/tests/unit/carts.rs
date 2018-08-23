use bigneon_db::models::Cart;
use support::project::TestProject;

#[test]
fn add_to_cart() {
    let mut db = TestProject::new();
    let event = db.create_event().finish();
    let ticket_allocation = event.add_ticket_allocation(100, &db).unwrap();
    let user = db.create_user().finish();
    let cart = Cart::create(user.id).commit(&db).unwrap();

    cart.add_item(ticket_allocation.id, 10, &db).unwrap();

    let dbCart = Cart::find_for_user(user.id, &db).unwrap();
    assert_eq!(cart.id, dbCart.id);
    assert_eq!(cart.items(&db).unwrap(), dbCart.items(&db).unwrap());
}

#[test]
fn find_by_user_when_cart_does_not_exist() {
    let mut db = TestProject::new();
    let user = db.create_user().finish();
    let cart_result = Cart::find_for_user(user.id, &db);
    assert_eq!(cart_result.err().unwrap().code, 2000);
}

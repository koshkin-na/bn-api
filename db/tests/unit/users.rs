use bigneon_db::models::{ExternalLogin, Roles, User, UserEditableAttributes};
use bigneon_db::utils::errors;
use bigneon_db::utils::errors::ErrorCode;
use support::project::TestProject;
use uuid::Uuid;
use validator::Validate;

#[test]
fn commit() {
    let project = TestProject::new();
    let first_name = "Jeff";
    let last_name = "Wilco";
    let email = "jeff@tari.com";
    let phone_number = "555-555-5555";
    let password = "examplePassword";
    let user = User::create(first_name, last_name, email, phone_number, password)
        .commit(project.get_connection())
        .unwrap();

    assert_eq!(user.first_name, first_name);
    assert_eq!(user.last_name, last_name);
    assert_eq!(user.email, Some(email.to_string()));
    assert_eq!(user.phone, Some(phone_number.to_string()));
    assert_ne!(user.hashed_pw, password);
    assert_eq!(user.hashed_pw.is_empty(), false);
    assert_eq!(user.id.to_string().is_empty(), false);
}

#[test]
fn commit_duplicate_email() {
    let project = TestProject::new();
    let user1 = project.create_user().finish();
    let first_name = "Jeff";
    let last_name = "Wilco";
    let email = &user1.email.unwrap();
    let phone_number = "555-555-5555";
    let password = "examplePassword";
    let result = User::create(first_name, last_name, email, phone_number, password)
        .commit(project.get_connection());

    assert_eq!(result.is_err(), true);
    assert_eq!(
        result.err().unwrap().code,
        errors::get_error_message(ErrorCode::DuplicateKeyError).0
    );
}

#[test]
fn find() {
    let project = TestProject::new();
    let user = project.create_user().finish();

    let found_user = User::find(user.id, project.get_connection()).expect("User was not found");
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.email, user.email);

    assert!(
        match User::find(Uuid::new_v4(), project.get_connection()) {
            Ok(_user) => false,
            Err(_e) => true,
        },
        "User incorrectly returned when id invalid"
    );
}

#[test]
fn full_name() {
    let project = TestProject::new();

    let first_name = "Bob".to_string();
    let last_name = "Jones".to_string();

    let user = project
        .create_user()
        .with_first_name(first_name.clone())
        .with_last_name(last_name.clone())
        .finish();
    assert_eq!(user.full_name(), format!("{} {}", first_name, last_name));
}

#[test]
fn find_by_email() {
    let project = TestProject::new();
    let user = project.create_user().finish();

    let found_user = User::find_by_email(&user.email.clone().unwrap(), project.get_connection())
        .expect("User was not found");
    assert_eq!(found_user, user);

    let not_found = User::find_by_email("not@real.com", project.get_connection());
    let error = not_found.unwrap_err();
    assert_eq!(
        error.to_string(),
        "[2000] No results\nCaused by: Error loading user, NotFound"
    );
}

#[test]
fn update() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let mut attributes: UserEditableAttributes = Default::default();
    let email = "new_email@tari.com";
    attributes.email = Some(email.clone().into());

    let updated_user = user.update(&attributes.into(), connection).unwrap();
    assert_eq!(updated_user.email, Some(email.into()));
}

#[test]
fn new_user_validate() {
    let email = "abc";
    let user = User::create(
        "First".into(),
        "Last".into(),
        email.into(),
        "123".into(),
        "Password".into(),
    );
    let result = user.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err().inner();

    assert!(errors.contains_key("email"));
    assert_eq!(errors["email"].len(), 1);
    assert_eq!(errors["email"][0].code, "email");
}

#[test]
fn user_editable_attributes_validate() {
    let mut user_parameters: UserEditableAttributes = Default::default();
    user_parameters.email = Some("abc".into());

    let result = user_parameters.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err().inner();

    assert!(errors.contains_key("email"));
    assert_eq!(errors["email"].len(), 1);
    assert_eq!(errors["email"][0].code, "email");
}

#[test]
fn create_from_external_login() {
    let project = TestProject::new();
    let external_id = "123";
    let first_name = "Dennis";
    let last_name = "Miguel";
    let email = "dennis@tari.com";
    let site = "facebook.com";
    let access_token = "abc-123";

    let user = User::create_from_external_login(
        external_id.to_string(),
        first_name.to_string(),
        last_name.to_string(),
        email.to_string(),
        site.to_string(),
        access_token.to_string(),
        project.get_connection(),
    ).unwrap();

    let external_login = ExternalLogin::find_user(external_id, site, project.get_connection())
        .unwrap()
        .unwrap();

    assert_eq!(user.id, external_login.user_id);
    assert_eq!(access_token, external_login.access_token);
    assert_eq!(site, external_login.site);
    assert_eq!(external_id, external_login.external_user_id);

    assert_eq!(Some(email.to_string()), user.email);
    assert_eq!(first_name, user.first_name);
    assert_eq!(last_name, user.last_name);
}

#[test]
fn for_display() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user_id = user.id.clone();
    let display_user = user.for_display();

    assert_eq!(display_user.id, user_id);
}

#[test]
fn add_role() {
    let project = TestProject::new();
    let user = project.create_user().finish();

    user.add_role(Roles::Admin, project.get_connection())
        .unwrap();
    //Try adding a duplicate role to check that it isnt duplicated.
    user.add_role(Roles::Admin, project.get_connection())
        .unwrap();

    let user2 = User::find(user.id, project.get_connection()).unwrap();
    assert_eq!(user2.role, vec!["User", "Admin"]);
}
use argon2rs::argon2i_simple;
use rand::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct User {
    pub username: String,
    password: [u8; 32],
    salt: String,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        let mut data = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut data);
        let salt = hex::encode(&data);

        User {
            username,
            password: argon2i_simple(&password, &salt),
            salt: salt.to_string(),
        }
    }
}

pub fn add_user(users: &mut Vec<User>, user: User) {
    users.push(user);
}

pub fn validate_user<'a>(users: &'a [User], username: &str, password: &str) -> Option<&'a User> {
    match users.iter().find(|u| u.username == username) {
        Some(u) => {
            let hashed = argon2i_simple(&password, &u.salt);
            if hashed == u.password {
                Some(u)
            } else {
                None   
            }
        }
        None => None
    }
}

#[test]
fn test_user_validates () {
    let mut users = Vec::new();
    let user = User::new("user".to_string(), "amazingpassword".to_string());
    add_user(&mut users, user);
    
    assert_eq!(Some("user".to_string()), 
               validate_user(&users,
                             "user".to_string(),
                             "amazingpassword".to_string())
               .map(|u| u.username.to_string() )
    );
}

#[test]
fn test_incorrect_password () {
    let mut users = Vec::new();
    let user = User::new("user".to_string(), "amazingpassword".to_string());
    add_user(&mut users, user);
    
    assert_eq!(None, 
               validate_user(&users,
                             "user".to_string(),
                             "terrible_password".to_string())
    );
}

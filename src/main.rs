#[macro_use]
extern crate nickel;
#[macro_use]
extern crate lazy_static;

use cookie;
use hex;
use hyper::header::{Cookie, Location, SetCookie};
use nickel::status::StatusCode;
use nickel::{FormBody, HttpRouter, Nickel, Request, Response, MiddlewareResult, StaticFilesHandler};
use nickel::extensions::response::Redirect;
use rand::{self, Rng};
use std::collections::HashMap;
use std::error;
use std::sync::RwLock;

mod database;

const SESSION_NAME: &str = "nickelsession";

lazy_static! {
    static ref DATABASE: RwLock<Vec<database::User>> = RwLock::new(Vec::new());
    static ref SESSIONS: RwLock<HashMap<String, database::User>> = RwLock::new(HashMap::new());
}


/// Retrieves the session cookie from the header of the given request.
fn get_session(req: &Request) -> Option<String> {
    if let Some(cookies) = req.origin.headers.get::<Cookie>() {
        cookies
            .iter()
            .map(|header| cookie::Cookie::parse(header).unwrap())
            .find(|cookie| cookie.name() == SESSION_NAME)
            .map(|cookie| String::from(cookie.value()))
    } else {
        None
    }
}


/// Generates a random string to use as a session id.
/// In a real world scenario we would also want to sign it.
fn gen_session_id() -> String {
    let bytes: [u8; 10] = rand::thread_rng().gen();
    hex::encode(bytes)
}


/// Creates the session cookie to be sent to the browser.
fn add_session_cookie(res: &mut Response, session: String) {
    res.set(SetCookie(vec![
        cookie::Cookie::build(SESSION_NAME, session)
            .path("/")
            .permanent()
            .finish()
            .to_string()
    ]));
}


/// Middleware to ensure we are logged in.
/// Redirects to login page if we are not.
fn auth_fn<'mw>(req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
    if let Some(session) = get_session(req) {
        let sessions = SESSIONS.read().unwrap();
        if sessions.contains_key(&session) {
            return res.next_middleware();
        } 
    } 

    println!("Unauthorized access");
    res.redirect("/login")
}


#[allow(dead_code)]
#[allow(unreachable_code)]
fn main() -> Result<(), Box<dyn error::Error>> {
    let mut server = Nickel::new();
    
    // Create routes that dont require authentication.
    let mut unauthenticated = Nickel::router();
    
    // Create routes that do require authentication.
    let mut authenticated = Nickel::router();

    unauthenticated.get(
        "/login",
        middleware! { |_req, res| {
            let data: HashMap<String, String> = HashMap::new();
            return res.render("templates/login.tpl", &data);
        }},
    );

    unauthenticated.post(
        "/login",
        middleware! { |req, mut res| {
            let form = req.form_body().unwrap();

            let username = form.get("username");
            let password = form.get("password");

            if let (Some(username), Some(password)) = (username, password) {
                let database = DATABASE.read().unwrap();
                if let  Some(user) = database::validate_user(&database, username, password) {
                    let mut sessions = SESSIONS.write().unwrap();
                    let session = gen_session_id();
                    sessions.insert(session.clone(), user.clone());
                    add_session_cookie(&mut res, session);
                    
                    return res.redirect("/");
                }
            }

            let data: HashMap<String, String> = HashMap::new();
            return res.render("templates/loginfail.tpl", &data);
        }},
    );

    unauthenticated.get(
        "/register",
        middleware! { |_req, res| {
            let data: HashMap<String, String> = HashMap::new();
            return res.render("templates/register.tpl", &data);
        }},
    );
    
    unauthenticated.post(
        "/register",
        middleware! { |req, mut res| {
            let form = req.form_body().unwrap();

            let username = form.get("username");
            let password = form.get("password");
            
            if let (Some(username), Some(password)) = (username, password) {
                let mut database = DATABASE.write().unwrap();
                database::add_user(&mut database, database::User::new(username.to_string(), password.to_string()));

                res.set(StatusCode::Found)
                    .set(Location("/login".into()));
            }            
            ""
        }},
    );
    
    authenticated.get(
        "/",
        middleware! { |req, res| {
            
            let sessions = SESSIONS.read().unwrap();
            
            // We can safely unwrap these because we know we have a session here.
            let session = get_session(req).unwrap(); 
            let user = sessions.get(&session).unwrap();

            let mut data = HashMap::new();
            data.insert("name", user.username.clone());
            return res.render("templates/main.tpl", &data);
        }},
    );

    
    server.utilize(unauthenticated);
    server.utilize(StaticFilesHandler::new("assets/"));
    server.utilize(auth_fn);
    server.utilize(authenticated);
    
    server.listen("127.0.0.1:6767")?;

    Ok(())
}

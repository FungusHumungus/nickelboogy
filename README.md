# nickelboogy
A quick demo of using the Nickel rust web framework

A simple mock up of a web app using the [http://nickel-org.github.io/](Nickel.rs) web framework. 

The site has a single page `/` that is only accessible to logged in users. If you aren't logged in you are redirected to `/login` where you can login on register a new user.

The session and user database is stored in memory, so is lost each time the app starts.

use rocket::fairing::{Fairing,Info,Kind};
use rocket::http::{ContentType, Header,Method,Status};
use rocket::{Request, Response};

pub struct CORS;
#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info{
            name: "Add CORS header to response",
            kind:Kind::Response,
        }
    }
}
 
async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response){
    response.set_header(Header::new("Access-Control-Allow-Origin","*"));
    response.set_header(Header::new("Access-Control-Allow-Methods","POST,GET, PATCH,OPTIONS"));
    response.set_header(Header::new("Access-Control-Allow-Headers","*"));
    response.set_header(Header::new("Access-Control-Allow-Credentials","true"));

    if request.method == Method::Options {
        let body = "";
        response.set_headers(ContentType::Plain);
        response.set_sized_body(body.len(), std::io::Cursor::new(body));
        response.set_status(Status::Ok);
}
}
use std::{error::Error, net::SocketAddr};

use hyper_req_exts::{
    prelude::Response,
    routerify::{prelude::RequestExt, Router},
    start_server,
};
use reqwest::header::CONTENT_TYPE;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:64380".parse().unwrap();
    eprintln!("Listening on {}", addr);
    start_server(
        addr,
        Router::<String, String>::builder()
            .get("/:callsign", |r| async move {
                let callsign = r.param("callsign").unwrap();
                let name = get_name(callsign).await.unwrap();
                Ok(Response::builder()
                    .header(CONTENT_TYPE, "text/html; charset=UTF-8")
                    .body(if name.is_empty() {
                        format!("{} için kayıt bulunamadı.", callsign)
                    } else {
                        name
                    })
                    .unwrap())
            })
            .get("/", |_| async move {
                Ok(Response::builder()
                    .status(200)
                    .header(CONTENT_TYPE, "text/html; charset=UTF-8")
                    .body("Adres sonuna çağrı işareti girmeyi unuttunuz.".to_string())
                    .unwrap())
            })
            .any(|_| async move {
                Ok(Response::builder()
                    .status(404)
                    .body("".to_string())
                    .unwrap())
            })
            .options("/*", |_| async move {
                Ok(Response::builder()
                    .status(404)
                    .body("".to_string())
                    .unwrap())
            })
            .err_handler(|err| async move {
                eprintln!("Error: {}", err);
                Response::builder()
                    .status(500)
                    .body("Internal Server Error".to_string())
                    .unwrap()
            })
            .build()
            .unwrap(),
    )
    .await;
}
async fn get_name(callsign: &str) -> Result<String, Box<dyn Error>> {
    //curl 'http://www.tacallbook.org/cgi-bin/bul1.cgi?ara=TA3KRT' \
    //-H 'Referer: http://www.tacallbook.org/call.shtml' > out.html
    let url = format!(
        "http://www.tacallbook.org/cgi-bin/bul1.cgi?ara={}",
        callsign
    );
    let body = reqwest::Client::new()
        .get(&url)
        .header("Referer", "http://www.tacallbook.org/call.shtml")
        .send()
        .await?
        .text()
        .await?;
    lazy_static::lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r#">(.*?[\w ]+)</s"#).unwrap();
        // static ref RE: regex::Regex = regex::Regex::new(r#"strong>([\w ]+)</strong>"#).unwrap();
    };
    let name = RE
        .captures(&body)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or_default();
    Ok(name.to_string())
}

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};

#[get("/versions/{rtype}")]
async fn versions(web::Path(rtype): web::Path<String>) -> Result<String> {
    let response: Response = serde_json::from_str(minreq::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
    .send().unwrap().as_str().unwrap()).unwrap();
    let mut r = Vec::new();
    for version in response.versions {
        if version.r#type == rtype {
            r.push(version.id);
        }
    }
    Ok(serde_json::to_string(&r).unwrap())
}

#[get("/server/{version}")]
async fn server(web::Path(version) : web::Path<String>) -> Result<String> {
    let response: Response = serde_json::from_str(minreq::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
    .send().unwrap().as_str().unwrap()).unwrap();
    for version_indiv in response.versions {
        if version_indiv.id == version {
           let response : Value = serde_json::from_str(minreq::get(version_indiv.url).send().unwrap().as_str().unwrap()).unwrap();
           return Ok(response["downloads"]["server"]["url"].to_string());
        }
    }
    Ok("error".to_string())
    
}

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
struct Version {
    id: String,
    r#type: String, // bruh
    url: String,
    time: String,
    releaseTime: String,
}

#[derive(Deserialize, Serialize)]
struct Response {
    versions: Vec<Version>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(versions)
            .service(server)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
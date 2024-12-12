use actix_web::{get, web, HttpResponse, Responder};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Frontmatter {
    title: String,
    file_name: String,
    description: String,
    posted: String,
    tags: Vec<String>,
    author: String,
    estimated_reading_time: u32,
    order: u32,
}

#[derive(Error, Debug)]
pub enum FrontmatterError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("Other error: {0}")]
    OtherError(String),
}

fn find_all_frontmatters() -> Result<Vec<Frontmatter>, FrontmatterError> {
    let mut t = ignore::types::TypesBuilder::new();
    t.add_defaults();
    let toml = t
        .select("toml")
        .build()
        .map_err(|e| FrontmatterError::OtherError(e.to_string()))?;
    let file_walker = WalkBuilder::new("./posts").types(toml).build();
    let mut frontmatters = Vec::new();
    for frontmatter in file_walker {
        match frontmatter {
            Ok(fm) => {
                if fm.path().is_file() {
                    let fm_content = fs::read_to_string(fm.path())?;
                    let frontmatter: Frontmatter = toml::from_str(&fm_content)?;
                    frontmatters.push(frontmatter);
                }
            }
            Err(e) => {
                println!("{:}", e);
                return Err(FrontmatterError::OtherError(
                    "could not locate frontmatter".to_string(),
                ));
            }
        }
    }
    Ok(frontmatters)
}

#[get("/")]
pub async fn index(templates: web::Data<tera::Tera>) -> impl Responder {
    let mut context = tera::Context::new();
    let mut frontmatters = match find_all_frontmatters() {
        Ok(fm) => fm,
        Err(e) => {
            println!("{:?}", e);
            return HttpResponse::InternalServerError()
                .content_type("text/html")
                .body("<p>Something went wrong!</p>");
        }
    };
    frontmatters.sort_by(|a, b| b.order.cmp(&a.order));
    context.insert("posts", &frontmatters);
    match templates.render("home.html", &context) {
        Ok(s) => HttpResponse::Ok().content_type("text/html").body(s),
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError()
                .content_type("text/html")
                .body("<p>Something went wrong!</p>")
        }
    }
}

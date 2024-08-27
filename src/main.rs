#[macro_use]
extern crate rocket;

use rocket::Data;
use rocket::data::ToByteUnit;
use rocket::http::ContentType;
use rocket::tokio::task::spawn_blocking;

async fn render(data: &str, scale: f32) -> (ContentType, Vec<u8>) {
    println!("Screenshot SVG: {} bytes", data.len());

    let tree = {
        let opt = usvg::Options::default();
        usvg::Tree::from_str(data, &opt).unwrap()
    };

    let png = spawn_blocking(move || {
        let pixmap_size = tree.size().to_int_size().scale_by(scale).unwrap();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
        resvg::render(
            &tree,
            tiny_skia::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );
        pixmap.encode_png().unwrap()
    })
        .await
        .unwrap();

    println!("Screenshot PNG: {} bytes", png.len());

    (ContentType::PNG, png)
}

#[post("/api/screenshot?<scale>", format = "image/svg+xml", data = "<data>")]
async fn screenshot_data(data: &str, scale: f32) -> (ContentType, Vec<u8>) {
    render(data, scale).await
}

#[get("/api/screenshot?<url>&<scale>", format = "image/svg+xml")]
async fn screenshot_url(url: &str, scale: f32) -> (ContentType, Vec<u8>) {
    println!("Screenshot URL: {}", url);
    let data = reqwest::get(url).await.unwrap().bytes().await.unwrap();

    render(std::str::from_utf8(data.as_ref()).expect("no string input"), scale).await
}

#[launch]
fn rocket() -> _ {
    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match std::env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    let figment = rocket::Config::figment().merge(("port", port));

    rocket::custom(figment).mount("/", routes![screenshot_data, screenshot_url])
}

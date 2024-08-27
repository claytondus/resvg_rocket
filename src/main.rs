#[macro_use] extern crate rocket;

use rocket::http::ContentType;

#[get("/api/screenshot?<url>&<scale>", format="image/svg+xml")]
async fn index(url: &str, scale: f32) -> (ContentType, Vec<u8>) {
    let req = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    
    let tree = {
        let opt = usvg::Options::default();
        usvg::Tree::from_data(&req, &opt).unwrap()
    };

    let pixmap_size = tree.size().to_int_size().scale_by(scale).unwrap();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let png = pixmap.encode_png().unwrap();

    (ContentType::PNG, png)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}

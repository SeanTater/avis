use std::sync::Arc;
use avis::wordcloud::{WordCloudVisual, CloudAction};
use avis::errors::{Result, AvisError};

fn graphics_main() -> Result<Arc<WordCloudVisual>> {
    let vis = Arc::new(WordCloudVisual::new());
    for i in 0..10 {
        vis.edit(CloudAction::AddWord(format!("Example {}", i)))?;
    }
    Ok(vis)
}


fn main() -> Result<()> {
    let vis = graphics_main()?;
    let vis_clone = vis.clone();
    std::thread::spawn(move || actix::main(vis_clone).unwrap());
    vis.start_app();
    Ok(())
}

mod actix {
    use std::sync::Arc;
    use actix_web::{web, App, HttpRequest, HttpServer, Responder};
    use avis::wordcloud::CloudAction;

    #[actix_web::main]
    pub async fn main(vis: Arc<avis::wordcloud::WordCloudVisual>) -> std::io::Result<()> {
        HttpServer::new(move || {
            App::new()
                .data(vis.clone())
                .route("/", web::get().to(greet))
                .route("/{name}", web::get().to(greet))
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
    }

    async fn greet(req: HttpRequest, vis: web::Data<Arc<avis::wordcloud::WordCloudVisual>>) -> impl Responder {
        let name = req.match_info().get("name").unwrap_or("World");
        vis.edit(CloudAction::AddWord("Actix!".into())).expect("Failed to update visual");
        format!("Hello {}!", &name)
    }
}
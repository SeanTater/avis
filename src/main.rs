use avis::errors::Result;
use avis::visuals::VisualAction;
use avis::visuals::VisualManager;

fn graphics_main() -> Result<VisualManager> {
    let vis = VisualManager::new();
    for i in 0..10 {
        vis.current()
            .edit(VisualAction::AddWord(format!("Example {}", i)))?;
    }
    Ok(vis)
}

fn main() -> Result<()> {
    let vis = graphics_main()?;
    let vis_clone = vis.clone();
    std::thread::spawn(move || actix::main(vis_clone).unwrap());
    vis.current().start_app();
    Ok(())
}

mod actix {
    use actix_files as fs;
    use actix_web::{web, App, HttpRequest, HttpServer, Responder};
    use avis::visuals::{VisualAction, VisualManager};

    #[actix_web::main]
    pub async fn main(vis: VisualManager) -> std::io::Result<()> {
        HttpServer::new(move || {
            App::new()
                .data(vis.clone())
                .route("/", web::get().to(index))
                .route("/greet", web::get().to(greet))
                .route("/exit", web::get().to(exit))
                .route("/{name}", web::get().to(greet))
                .service(fs::Files::new("/static", "./src/frontend/static").show_files_listing())
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
    }

    async fn index() -> std::io::Result<fs::NamedFile> {
        Ok(fs::NamedFile::open("./src/frontend/index.html")?)
    }

    async fn greet(req: HttpRequest, vis: web::Data<VisualManager>) -> impl Responder {
        let name = req.match_info().get("name").unwrap_or("World");
        vis.current()
            .edit(VisualAction::AddWord(format!("Hello {}", name)))
            .expect("Failed to update visual");
        format!("Hello {}!", &name)
    }

    async fn exit(vis: web::Data<VisualManager>) -> impl Responder {
        vis.current()
            .edit(VisualAction::Exit)
            .expect("Failed to update visual");
        "Goodbye!"
    }
}

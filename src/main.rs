use avis::errors::Result;
use avis::visuals::Visual;
use avis::visuals::VisualAction;
use avis::visuals::VisualManager;
use clap::Parser;

fn graphics_main() -> Result<VisualManager> {
    let vis = VisualManager::new();
    for i in 0..10 {
        vis.current()
            .react(VisualAction::AddWord(format!("Example {}", i)))
            .close();
    }
    Ok(vis)
}

fn dummy_graphics_main() -> Result<VisualManager> {
    todo!();
}

#[derive(Parser)]
struct CLIOptions {
    #[clap(long)]
    no_visual: bool
}

fn main() -> Result<()> {
    let args = CLIOptions::parse();
    let vis = if args.no_visual {
        // If we don't start bevy, then we have to start the subscriber.
        tracing_subscriber::fmt::init();
        dummy_graphics_main()?
    } else {
        // Bevy's DefaultPlugins includes tracing_subscriber, which will error if called twice.
        graphics_main()?
    };
    let vis_clone = vis.clone();
    std::thread::spawn(move || actix::main(vis_clone).unwrap());
    vis.current().start()?;
    Ok(())
}

mod actix {
    use std::str::FromStr;

    use actix_files as fs;
    use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse};
    use avis::visuals::{VisualAction, VisualManager, Visual, Visuals};
    use avis::errors::Result;

    #[actix_web::main]
    pub async fn main(vis: VisualManager) -> std::io::Result<()> {
        let server_baHttpServer::new(move || {
            App::new()
                .data(vis.clone())
                .route("/", web::get().to(index))
                .route("/greet", web::get().to(greet))
                .route("/exit", web::get().to(exit))
                .route("/switch/{name}", web::get().to(switch))
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

    async fn switch(manager: web::Data<VisualManager>, name: web::Path<String>) -> Result<impl Responder> {
        let new_visual = Visuals::from_str(&name).unwrap_or_default();
        manager.switch(new_visual).await?;
        
        Ok(format!("Switched to visual {:?}", name))
    }

    async fn greet(req: HttpRequest, vis: web::Data<VisualManager>) -> Result<impl Responder> {
        let name = req.match_info().get("name").unwrap_or("World");
        vis.current()
            .react(VisualAction::AddWord(format!("Hello {}", name)))
            .await??;
        
        Ok(format!("Hello {}!", &name))
    }

    async fn exit(vis: web::Data<VisualManager>) -> Result<impl Responder> {
        vis.current().stop().await??;
        Ok("Goodbye!")
    }
}

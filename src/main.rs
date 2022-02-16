use avis::errors::Result;

fn main() -> Result<()> {
    avis::visuals::wordcloud::WordCloudVisual::new().start()?;
    Ok(())
}
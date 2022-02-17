use avis::errors::Result;

fn main() -> Result<()> {
    avis::visuals::wordcloud::WordCloudVisual::new("words.json".as_ref())?.start()?;
    Ok(())
}
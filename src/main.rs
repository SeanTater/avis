use std::path::PathBuf;

use avis::errors::Result;
use clap::arg;

fn main() -> Result<()> {
    let args = clap::Command::new("avis")
        .arg(arg!(--words <WORDLIST> "JSON file containing list of words to use, see example"))
        .get_matches();
    avis::visuals::wordcloud::WordCloudVisual::new(&args.value_of_t_or_exit::<PathBuf>("words"))?.start()?;
    Ok(())
}
use std::path::PathBuf;

use avis::errors::Result;
use clap::arg;

fn main() -> Result<()> {
    let wordcloudcommand = clap::Command::new("wordcloud")
        .arg(arg!(--words <WORDLIST> "JSON file containing list of words to use, see example"));
    let mapcommand = clap::Command::new("map");
    let args = clap::Command::new("avis")
        .subcommand(wordcloudcommand)
        .subcommand(mapcommand)
        .get_matches();
    match args.subcommand() {
        Some(("wordcloud", subargs)) => {
            avis::visuals::wordcloud::WordCloudVisual::new(
                &subargs.value_of_t_or_exit::<PathBuf>("words"),
            )?
            .start()?;
        }
        Some(("map", _subargs)) => {
            avis::visuals::reliefmap::main()?;
        }
        _ => panic!("Please choose a command"),
    }

    Ok(())
}

use async_compat::{Compat, CompatExt};
use std::path::PathBuf;

use reqwest::Url;
use smol::{io, net, prelude::*, Unblock};

use clap::Parser;

use html5ever::{parse_document, tendril::TendrilSink, Parser as HTMLParser};
use markup5ever_rcdom::RcDom;

/// A smol web scraper.
///
/// Cache your favorite websites to your local machine!
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL to start scraping at.
    #[arg(short, long)]
    start: Url,

    /// Whether to include the assets for the page.
    #[arg(short, long, default_value_t = true)]
    include_assets: bool,

    /// Output directory (...er, file) to save files to.
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    // URL parsing
    // HTTP handling, in an async runtime

    let args = Args::parse();

    smol::block_on(Compat::new(async {
        let resp = reqwest::get(args.start).await?.bytes().await?;
        eprintln!("bytes len: {}", resp.len());

        let f = std::fs::File::create(args.output)?;

        let mut stdout = Unblock::new(f);
        stdout.write_all(&resp).await?;
        stdout.flush().await?;

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut std::io::Cursor::new(&resp))?;

        eprintln!("dom? {:?}", dom.document);

        eprintln!("All done!");
        Ok(())
    }))
}

use anyhow::Context;
use async_compat::{Compat, CompatExt};
use std::{io::Write, path::PathBuf};

use reqwest::Url;
use smol::{io, net, prelude::*, Unblock, Task};

use clap::Parser;

use html5ever::{parse_document, tendril::TendrilSink, Parser as HTMLParser};
use markup5ever_rcdom::{RcDom, Handle, NodeData};

use futures::future;
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

fn download_img(source: &str) -> Task<anyhow::Result<()>> {
    let source = source.to_string();
    smol::spawn(Compat::new(async move {
        let url : Url = source.parse()?;
        let resp = reqwest::get(source).await?.bytes().await?;
        eprintln!("img size: {}", resp.len());
        let filename = url.path_segments().context("failed to parse path segments")?.last ().context ("could not get last segment of path")?;
        let filebuf = std::fs::File::create(format!("/tmp/scrape/{}" ,filename))?;
        let mut filebuf = Unblock::new (filebuf);
        let _ = filebuf.write_all(&resp).await?;
        filebuf.flush().await?;
        Ok(())
    }))
}

fn walk(handle: &Handle, tasks: &mut Vec<Task<anyhow::Result<()>>>) {
    if let NodeData::Element { ref name, ref attrs, .. } = handle.data {
        match name.local.to_ascii_lowercase().as_ref() {
            "img" => {
                eprintln!("found an image");
                for attr in attrs.borrow().iter() {
                    if attr.name.local.to_ascii_lowercase().as_ref() == "src" {
                        eprintln!("src is {}", &attr.value[..]);
                        tasks.push(download_img(&attr.value[..]));
                    }
                }
            },
            _ => { },
        }
    }

    for child in handle.children.borrow().iter() {
        walk(child, tasks);
    }
}

fn main() -> anyhow::Result<()> {
    // URL parsing
    // HTTP handling, in an async runtime

    let args = Args::parse();

    smol::block_on(Compat::new(async {
        let resp = reqwest::get(args.start).await?.bytes().await?;
        eprintln!("bytes len: {}", resp.len());

        let f = std::fs::File::create(args.output)?;
        let _ = std::fs::create_dir_all("/tmp/scrape")?;

        let mut stdout = Unblock::new(f);
        stdout.write_all(&resp).await?;
        stdout.flush().await?;

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut std::io::Cursor::new(&resp))?;

        // eprintln!("dom? {:?}", dom.document);
        let mut tasks = Vec::new();
        walk(&dom.document, &mut tasks);

        future::join_all(tasks).await;

        eprintln!("All done!");
        Ok(())
    }))
}

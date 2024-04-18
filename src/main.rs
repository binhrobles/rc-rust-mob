use std::path::PathBuf;

use smol::{io, net, prelude::*, Unblock};

use clap::Parser;

/// A smol web scraper.
///
/// Cache your favorite websites to your local machine!
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL to start scraping at.
    #[arg(short, long)]
    start: Option<String>,

    /// Whether to include the assets for the page.
    #[arg(short, long, default_value_t = true)]
    include_assets: bool,

    /// Output directory (...er, file) to save files to.
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    smol::block_on(async {
        let mut stream = net::TcpStream::connect("recurse.com:80").await?;
        let req = b"GET / HTTP/1.1\r\nHost: recurse.com\r\nConnection: close\r\n\r\n";
        stream.write_all(req).await?;

        let f = std::fs::File::create(args.output)?;

        let mut stdout = Unblock::new(f);
        io::copy(stream, &mut stdout).await?;
        eprintln!("All done!");
        Ok(())
    })
}

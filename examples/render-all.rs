use std::time::Instant;

use eyre::Result;

fn main() -> Result<()> {
    eprintln!("{:<20} {}", "path", "render time");

    for path in glob::glob("./data/*.tvg")? {
        let path = path?;

        let start = Instant::now();
        tinyvg::render_helper::render(&path, None)?;
        eprintln!("{:<20} {:?}", path.display(), start.elapsed());
    }

    Ok(())
}

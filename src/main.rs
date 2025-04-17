pub mod error;
pub mod parse_data;
mod prelude;
pub mod swiping_img;

use error::Result;
use prelude::{debug_print, read_json};
use std::{fs, path::Path, time::Instant};
use swiping_img::BigImg;

fn main() -> Result<()> {
    let t = Instant::now();
    let data_file = Path::new("./data").join("Crop2.json");

    let work_dir = Path::new("E:/pictures/arknights/0birth");
    fs::create_dir_all(work_dir).map_err(|e| err_new_io!(e))?;

    let data_use = &read_json(data_file)?;

    // let si = BigImg::new(work_dir, &data_use);
    let si = BigImg::builder(work_dir, &data_use)
        .text_background_color([236, 162, 56, 255], [255, 226, 197, 255])
        .text_color([0, 0, 0, 255])
        .build()?;
    debug_print(&si);
    si.run("result.mp4")?;

    println!("cost {} s", t.elapsed().as_secs());
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}

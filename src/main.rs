use std::process::Command;
use std::fs::read_dir;
use std::path::PathBuf;

use clap::Parser;
use plotview::{Image, ImageCropper};
use anyhow::{Context, Result, anyhow, bail};

fn find_files(starts_with: &str, exclude_end: &str) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut current_dir_iter = read_dir(".")
        .context("Couldn't read current directory")?;
    while let Some(Ok(entry)) = current_dir_iter.next() {
        // If we don't have perms to read some file properties we don't crash,
        // we just ignore the file
        let file_type = entry.file_type();
        if file_type.is_err() {
            continue;
        }

        // Check that its a file and stats with the provided pattern
        if file_type?.is_file() {
            let file_name = entry.file_name().into_string().unwrap();
            if file_name.starts_with(starts_with) 
                    && (exclude_end == "" || !file_name.ends_with(exclude_end))
            {
                paths.push(entry.path());
            }
        }
    }

    Ok(paths)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    input_pdf: PathBuf,

    #[arg(short, long, default_value_t = String::from("out"))]
    out_dir: String,
}

fn main() -> Result<()> {
    // Parse the args and check that are valid
    let args = Args::parse();
    if !args.input_pdf.exists() {
        bail!("Input PDF doesn't exist");
    }

    // Check if pdfimages exists
    Command::new("pdfimages").args(&["--help"]).output()
        .map_err(|_| anyhow!("`pdfimages` not present in the path"))?;

    // Produce the images for the input file
    let result = Command::new("pdfimages")
            .args(&[args.input_pdf.to_str().unwrap(), "img"]).output()?;
    if !result.status.success() {
        bail!("`pdfimages` command failed: {}", 
            std::str::from_utf8(&result.stderr).unwrap());
    };

    let mut crops = Vec::new();

    // Transform the generated images to bmp and save it
    let mut path = find_files("img-", "bmp")?[0].clone();

    // Load the image
    let image = Image::new(path)?;

    // Start the image cropper
    crops = ImageCropper::new(image)
        .run()?;

    let out_dir = PathBuf::from(args.out_dir);

    // Clear the out_dir
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir)?;
    }
    std::fs::create_dir(&out_dir)?;

    // Save the image crops
    for crop in &crops {
        crop.save(&out_dir)?;
    }

    // Cleanup, generated bmps
    for path in &find_files("img-", "bmp")? {
        std::fs::remove_file(path)
            .context("Can't remove files for cleanup")?;
    }

    Ok(())
}

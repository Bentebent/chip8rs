use std::{
    env,
    fs,
    path::Path,
};

fn main() {
    // Get the OUT_DIR environment variable
    let out_dir = env::var("OUT_DIR").unwrap();

    // Define the source and destination directories
    let assets_src = Path::new("assets");
    let assets_dest = Path::new(&out_dir).join("assets");

    // Ensure the destination directory exists
    fs::create_dir_all(&assets_dest).unwrap();

    // Copy all files from the source directory to the destination directory
    fn copy_dir_recursive(src: &Path, dst: &Path) {
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let dest = dst.join(entry.file_name());

            if path.is_dir() {
                fs::create_dir_all(&dest).unwrap();
                copy_dir_recursive(&path, &dest);
            } else {
                fs::copy(path, dest).unwrap();
            }
        }
    }

    copy_dir_recursive(assets_src, &assets_dest);

    // Print to notify Cargo of the change (e.g., if files were added or updated)
    println!("cargo:rerun-if-changed=assets/");
}

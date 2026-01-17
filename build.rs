use std::fs;
use std::path::Path;

fn main() {
    // Ensure dashboard/dist exists for rust-embed
    // If not built, the dashboard will just return 404s
    let dist_path = Path::new("dashboard/dist");
    if !dist_path.exists() {
        fs::create_dir_all(dist_path).expect("Failed to create dashboard/dist directory");
    }
}

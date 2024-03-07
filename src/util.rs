use std::path::PathBuf;

pub fn find_project_file_from_current_dir() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?.canonicalize().ok()?;

    loop {
        for item in dir.read_dir().ok()? {
            let item = item.ok()?;
            if item.file_name().to_string_lossy() == crate::PROJECT_FILENAME {
                return Some(item.path());
            }
        }

        if !dir.pop() {
            break;
        }
    }

    None
}

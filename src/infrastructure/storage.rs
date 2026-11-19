use std::{
    fs, io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub const REGISTRY_PATH: &str = "data/tasks.json";
pub const SCRIPTS_DIR: &str = "data/scripts";

pub fn script_path(name: &str) -> PathBuf {
    Path::new(SCRIPTS_DIR).join(format!("{name}.sh"))
}

/// 儲存腳本到 `data/scripts/<name>.sh`，權限設為只有擁有者可讀寫執行。
pub fn save_script(name: &str, content: &str) -> io::Result<PathBuf> {
    fs::create_dir_all(SCRIPTS_DIR)?;

    let path = script_path(name);
    write_atomic(&path, content.as_bytes())?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;

    Ok(path)
}

pub fn read_script(name: &str) -> io::Result<String> {
    fs::read_to_string(script_path(name))
}

pub fn remove_script(name: &str) -> io::Result<()> {
    match fs::remove_file(script_path(name)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

/// 先寫入暫存檔再 rename，避免寫到一半當機留下壞掉的檔案。
pub fn write_atomic(path: &Path, content: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content)?;
    fs::rename(tmp, path)
}

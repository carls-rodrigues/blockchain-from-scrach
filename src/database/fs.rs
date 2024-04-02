use super::write_genesis_to_disk;

pub fn init_data_dir_if_not_exists(data_dir: &str) -> std::io::Result<()> {
    if file_exists(&get_genesis_json_file_path(data_dir).unwrap()) {
        return Ok(());
    };

    let Ok(db_dir) = get_database_dir_path(data_dir) else {
        panic!("Invalid database directory path")
    };
    std::fs::create_dir_all(db_dir)?;
    let Ok(genesis_file_path) = get_genesis_json_file_path(data_dir) else {
        panic!("Invalid genesis file path")
    };
    write_genesis_to_disk(&genesis_file_path);
    let Ok(blocks_db_file_path) = get_blocks_db_file_path(data_dir) else {
        panic!("Invalid blocks db file path")
    };
    write_empty_blocks_db_file(&blocks_db_file_path)?;
    Ok(())
}

pub fn get_database_dir_path(data_dir: &str) -> std::io::Result<String> {
    let path = std::path::Path::new(data_dir).join("src").join("database");
    match path.to_str() {
        Some(path_str) => Ok(path_str.to_string()),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid path",
        )),
    }
}

pub fn get_genesis_json_file_path(data_dir: &str) -> std::io::Result<String> {
    let path = std::path::Path::new(data_dir).join("genesis.json");
    match path.to_str() {
        Some(path_str) => Ok(path_str.to_string()),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid path",
        )),
    }
}

pub fn get_blocks_db_file_path(data_dir: &str) -> std::io::Result<String> {
    let path = std::path::Path::new(data_dir).join("block.db");
    match path.to_str() {
        Some(path_str) => Ok(path_str.to_string()),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid path",
        )),
    }
}

pub fn file_exists(file_path: &str) -> bool {
    std::path::Path::new(file_path).exists()
}

pub fn dir_exists(dir_path: &str) -> bool {
    std::path::Path::new(dir_path).exists()
}

pub fn write_empty_blocks_db_file(data_dir: &str) -> std::io::Result<()> {
    std::fs::write(data_dir, "".as_bytes())?;
    Ok(())
}

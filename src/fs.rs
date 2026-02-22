use std::fs::File;

pub fn get_file_hash(file: &File) -> std::io::Result<[u8; 32]> {
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(file)?;

    return Ok(*hasher.finalize().as_bytes());
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Seek};

    use crate::fs::get_file_hash;

    #[test]
    fn sanity_get_file_hash() {
        let mut file = File::open("./src/tests/fixtures/test_file").unwrap();
        let hash = get_file_hash(&file).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        let hash2 = get_file_hash(&file).unwrap();
        assert_eq!(hash, hash2);
    }
}

use rusty::archive::ArchiveManager;
use std::fs;
use tempfile::TempDir;

#[test]
fn create_list_extract_validate_and_hash() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let work = tmp.path();

    // Prepare files
    let f1 = work.join("a.txt");
    let f2 = work.join("b.txt");
    fs::write(&f1, "hello")?;
    fs::write(&f2, "world")?;

    // Create archive
    let zip = work.join("test.zip");
    let am = ArchiveManager::new();
    am.create_archive(&zip, &[&f1, &f2])?;
    assert!(zip.exists());

    // List
    let mut contents = am.list_archive(&zip)?;
    contents.sort();
    assert_eq!(contents, vec!["a.txt".to_string(), "b.txt".to_string()]);

    // Stats
    let stats = am.get_archive_stats(&zip)?;
    assert_eq!(stats.file_count, 2);
    assert!(stats.total_uncompressed_size > 0);

    // Validate
    assert!(am.validate_archive(&zip)?);

    // Extract
    let out = work.join("out");
    fs::create_dir(&out)?;
    am.extract_archive(&zip, &out)?;
    assert_eq!(fs::read_to_string(out.join("a.txt"))?, "hello");
    assert_eq!(fs::read_to_string(out.join("b.txt"))?, "world");

    // Hash
    let h1 = am.calculate_file_hash(&f1)?;
    let h1b = am.calculate_file_hash(&f1)?;
    assert_eq!(h1.len(), 64);
    assert_eq!(h1, h1b);

    Ok(())
}


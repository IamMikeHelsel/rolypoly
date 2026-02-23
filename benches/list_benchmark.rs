use rolypoly::archive::ArchiveManager;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;

fn create_large_archive(path: &Path, file_count: usize) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for i in 0..file_count {
        zip.start_file(format!("path/to/nested/directory/structure/file_{}.txt", i), options)?;
        zip.write_all(b"a")?;
    }
    zip.finish()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("large.zip");
    let file_count = 200_000;

    println!("Creating archive with {} files...", file_count);
    let start_create = Instant::now();
    create_large_archive(&archive_path, file_count)?;
    println!("Creation took: {:?}", start_create.elapsed());

    let manager = ArchiveManager::new();

    // Warmup
    // manager.list_archive_with_callback(&archive_path, |_| {})?;

    println!("Benchmarking list_archive (allocating)...");
    let start = Instant::now();
    let _contents = manager.list_archive(&archive_path)?;
    let duration = start.elapsed();
    println!("Time taken (allocating): {:?}", duration);
    println!("Average time per file: {:?}", duration / file_count as u32);

    println!("Benchmarking list_archive_with_callback (streaming)...");
    let start_cb = Instant::now();
    let mut count = 0;
    manager.list_archive_with_callback(&archive_path, |_name| {
        count += 1;
    })?;
    let duration_cb = start_cb.elapsed();
    println!("Time taken (streaming): {:?}", duration_cb);
    println!("Average time per file: {:?}", duration_cb / file_count as u32);

    println!("Improvement: {:.2}x", duration.as_secs_f64() / duration_cb.as_secs_f64());

    Ok(())
}

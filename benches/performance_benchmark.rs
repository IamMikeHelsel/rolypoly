use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;

#[derive(Debug)]
struct BenchmarkResult {
    operation: String,
    tool: String,
    file_count: usize,
    total_size_mb: f64,
    time_ms: u128,
    throughput_mbps: f64,
    compression_ratio: Option<f64>,
}

impl BenchmarkResult {
    fn new(
        operation: String,
        tool: String,
        file_count: usize,
        total_size_mb: f64,
        time_ms: u128,
    ) -> Self {
        let throughput_mbps = if time_ms > 0 {
            (total_size_mb * 1000.0) / time_ms as f64
        } else {
            0.0
        };

        Self {
            operation,
            tool,
            file_count,
            total_size_mb,
            time_ms,
            throughput_mbps,
            compression_ratio: None,
        }
    }

    fn with_compression_ratio(mut self, ratio: f64) -> Self {
        self.compression_ratio = Some(ratio);
        self
    }
}

fn create_test_files(dir: &Path) -> Result<f64, Box<dyn std::error::Error>> {
    let mut total_size = 0u64;

    // Small text files (100 files, ~1KB each)
    for i in 0..100 {
        let content =
            format!("This is test file {} with some repeated content. {}", i, "A".repeat(800));
        let file_path = dir.join(format!("small_{}.txt", i));
        fs::write(&file_path, &content)?;
        total_size += content.len() as u64;
    }

    // Medium files (10 files, ~100KB each)
    for i in 0..10 {
        let content = format!("Medium file {} content: {}", i, "B".repeat(100000));
        let file_path = dir.join(format!("medium_{}.txt", i));
        fs::write(&file_path, &content)?;
        total_size += content.len() as u64;
    }

    // Large files (3 files, ~1MB each)
    for i in 0..3 {
        let content = format!("Large file {} content: {}", i, "C".repeat(1048576));
        let file_path = dir.join(format!("large_{}.txt", i));
        fs::write(&file_path, &content)?;
        total_size += content.len() as u64;
    }

    // Binary-like file (low compressibility)
    let binary_content: Vec<u8> = (0..1024 * 512).map(|i| (i % 256) as u8).collect();
    let binary_path = dir.join("binary.dat");
    fs::write(&binary_path, &binary_content)?;
    total_size += binary_content.len() as u64;

    // Highly compressible file
    let compressible_content = "AAAAAAAAAA\n".repeat(100000);
    let compressible_path = dir.join("compressible.txt");
    fs::write(&compressible_path, &compressible_content)?;
    total_size += compressible_content.len() as u64;

    Ok(total_size as f64 / 1024.0 / 1024.0) // Convert to MB
}

fn benchmark_rolypoly_create(
    test_dir: &Path,
    archive_path: &Path,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    // Build rolypoly if needed
    if !Path::new("./target/release/rolypoly").exists() {
        Command::new("cargo").args(["build", "--release"]).status()?;
    }

    let files: Vec<_> = fs::read_dir(test_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();

    let total_size_mb = create_test_files(test_dir)?;

    let start = Instant::now();

    let mut cmd = Command::new("./target/release/rolypoly");
    cmd.arg("create").arg(archive_path);
    if let Ok(level) = env::var("RP_LEVEL") {
        if !level.is_empty() {
            cmd.arg("--level").arg(level);
        }
    }
    for file in &files {
        cmd.arg(file);
    }

    let output = cmd.output()?;
    let elapsed = start.elapsed();

    if !output.status.success() {
        return Err(
            format!("rolypoly create failed: {}", String::from_utf8_lossy(&output.stderr)).into()
        );
    }

    let compressed_size = fs::metadata(archive_path)?.len() as f64 / 1024.0 / 1024.0;
    let compression_ratio = compressed_size / total_size_mb;

    Ok(BenchmarkResult::new(
        "create".to_string(),
        "rolypoly".to_string(),
        files.len(),
        total_size_mb,
        elapsed.as_millis(),
    )
    .with_compression_ratio(compression_ratio))
}

fn benchmark_rolypoly_extract(
    archive_path: &Path,
    extract_dir: &Path,
    file_count: usize,
    total_size_mb: f64,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    fs::create_dir_all(extract_dir)?;

    let start = Instant::now();

    let output = Command::new("./target/release/rolypoly")
        .args(["extract", archive_path.to_str().unwrap(), "-o", extract_dir.to_str().unwrap()])
        .output()?;

    let elapsed = start.elapsed();

    if !output.status.success() {
        return Err(format!(
            "rolypoly extract failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(BenchmarkResult::new(
        "extract".to_string(),
        "rolypoly".to_string(),
        file_count,
        total_size_mb,
        elapsed.as_millis(),
    ))
}

fn benchmark_system_zip_create(
    test_dir: &Path,
    archive_path: &Path,
    file_count: usize,
    total_size_mb: f64,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let output = Command::new("zip")
        .args(["-r", archive_path.to_str().unwrap(), "."])
        .current_dir(test_dir)
        .output()?;

    let elapsed = start.elapsed();

    if !output.status.success() {
        return Err(
            format!("System zip failed: {}", String::from_utf8_lossy(&output.stderr)).into()
        );
    }

    let compressed_size = fs::metadata(archive_path)?.len() as f64 / 1024.0 / 1024.0;
    let compression_ratio = compressed_size / total_size_mb;

    Ok(BenchmarkResult::new(
        "create".to_string(),
        "system-zip".to_string(),
        file_count,
        total_size_mb,
        elapsed.as_millis(),
    )
    .with_compression_ratio(compression_ratio))
}

fn benchmark_system_zip_extract(
    archive_path: &Path,
    extract_dir: &Path,
    file_count: usize,
    total_size_mb: f64,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    fs::create_dir_all(extract_dir)?;

    let start = Instant::now();

    let output = Command::new("unzip")
        .args(["-q", archive_path.to_str().unwrap(), "-d", extract_dir.to_str().unwrap()])
        .output()?;

    let elapsed = start.elapsed();

    if !output.status.success() {
        return Err(
            format!("System unzip failed: {}", String::from_utf8_lossy(&output.stderr)).into()
        );
    }

    Ok(BenchmarkResult::new(
        "extract".to_string(),
        "system-zip".to_string(),
        file_count,
        total_size_mb,
        elapsed.as_millis(),
    ))
}

fn run_benchmarks() -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().join("test_files");
    fs::create_dir(&test_dir)?;

    println!("Creating test files...");
    let total_size_mb = create_test_files(&test_dir)?;
    let file_count = fs::read_dir(&test_dir)?.count();

    println!("Test dataset: {} files, {:.2} MB", file_count, total_size_mb);
    println!("Running benchmarks...\n");

    let mut results = Vec::new();

    // Benchmark RolyPoly
    println!("Benchmarking RolyPoly...");
    let rolypoly_archive = temp_dir.path().join("rolypoly_test.zip");
    let rolypoly_create_result = benchmark_rolypoly_create(&test_dir, &rolypoly_archive)?;
    println!(
        "  Create: {:.0}ms ({:.2} MB/s)",
        rolypoly_create_result.time_ms, rolypoly_create_result.throughput_mbps
    );
    results.push(rolypoly_create_result);

    let rolypoly_extract_dir = temp_dir.path().join("rolypoly_extract");
    let rolypoly_extract_result = benchmark_rolypoly_extract(
        &rolypoly_archive,
        &rolypoly_extract_dir,
        file_count,
        total_size_mb,
    )?;
    println!(
        "  Extract: {:.0}ms ({:.2} MB/s)",
        rolypoly_extract_result.time_ms, rolypoly_extract_result.throughput_mbps
    );
    results.push(rolypoly_extract_result);

    // Benchmark system zip if available
    if Command::new("zip").arg("--version").output().is_ok() {
        println!("\nBenchmarking system zip...");
        let zip_archive = temp_dir.path().join("system_test.zip");
        if let Ok(zip_create_result) =
            benchmark_system_zip_create(&test_dir, &zip_archive, file_count, total_size_mb)
        {
            println!(
                "  Create: {:.0}ms ({:.2} MB/s)",
                zip_create_result.time_ms, zip_create_result.throughput_mbps
            );
            results.push(zip_create_result);

            let zip_extract_dir = temp_dir.path().join("zip_extract");
            if let Ok(zip_extract_result) = benchmark_system_zip_extract(
                &zip_archive,
                &zip_extract_dir,
                file_count,
                total_size_mb,
            ) {
                println!(
                    "  Extract: {:.0}ms ({:.2} MB/s)",
                    zip_extract_result.time_ms, zip_extract_result.throughput_mbps
                );
                results.push(zip_extract_result);
            }
        }
    } else {
        println!("\nSystem zip not available, skipping comparison");
    }

    Ok(results)
}

fn print_summary(results: &[BenchmarkResult]) {
    println!("\n{}", "=".repeat(80));
    println!("PERFORMANCE BENCHMARK SUMMARY");
    println!("{}", "=".repeat(80));

    println!(
        "{:<12} {:<12} {:<8} {:<10} {:<8} {:<10} {:<10}",
        "Tool", "Operation", "Files", "Size(MB)", "Time(ms)", "MB/s", "Ratio"
    );
    println!("{}", "-".repeat(80));

    for result in results {
        let ratio_str = if let Some(ratio) = result.compression_ratio {
            format!("{:.1}%", ratio * 100.0)
        } else {
            "-".to_string()
        };

        println!(
            "{:<12} {:<12} {:<8} {:<10.2} {:<8} {:<10.2} {:<10}",
            result.tool,
            result.operation,
            result.file_count,
            result.total_size_mb,
            result.time_ms,
            result.throughput_mbps,
            ratio_str
        );
    }

    println!("\nSystem Information:");
    println!("- Platform: {}", std::env::consts::OS);
    println!("- Architecture: {}", std::env::consts::ARCH);

    if let Ok(output) = Command::new("sysctl").args(["-n", "hw.ncpu"]).output()
        && let Ok(cpu_count) = String::from_utf8(output.stdout)
    {
        println!("- CPU Cores: {}", cpu_count.trim());
    }
}

#[test]
fn performance_benchmark() {
    match run_benchmarks() {
        Ok(results) => {
            print_summary(&results);

            // Assert some basic performance expectations
            let rolypoly_create =
                results.iter().find(|r| r.tool == "rolypoly" && r.operation == "create");
            let rolypoly_extract =
                results.iter().find(|r| r.tool == "rolypoly" && r.operation == "extract");

            if let Some(create_result) = rolypoly_create {
                assert!(
                    create_result.throughput_mbps > 1.0,
                    "Create throughput should be > 1 MB/s"
                );
                if let Some(ratio) = create_result.compression_ratio {
                    assert!(ratio < 1.0, "Should achieve some compression");
                }
            }

            if let Some(extract_result) = rolypoly_extract {
                assert!(
                    extract_result.throughput_mbps > 5.0,
                    "Extract throughput should be > 5 MB/s"
                );
            }
        }
        Err(e) => {
            eprintln!("Benchmark failed: {}", e);
            panic!("Benchmark execution failed");
        }
    }
}

fn main() {
    match run_benchmarks() {
        Ok(results) => print_summary(&results),
        Err(e) => eprintln!("Benchmark failed: {}", e),
    }
}

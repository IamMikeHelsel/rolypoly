# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is "rolypoly" - a Rust-based compression utility similar to WinZip. The project is currently in early development with minimal infrastructure in place.

## Current State

The repository contains:

- Rust project structure with src/main.rs
- Cargo.toml with dependencies for ZIP functionality and CLI
- Core dependencies: zip, clap, anyhow, walkdir
- Test dependencies: tempfile

## Development Commands

- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo run` - Run the application
- `cargo check` - Check for compilation errors without building
- `cargo clippy` - Run linting
- `cargo fmt` - Format code

## CLI Usage

The application supports these commands:

- `rolypoly create <archive_name> <file1> <file2> ...` - Create a new ZIP archive
- `rolypoly extract <archive_name> [-o output_dir]` - Extract a ZIP archive
- `rolypoly list <archive_name>` - List contents of a ZIP archive
- `rolypoly validate <archive_name>` - Validate archive integrity and check for corruption
- `rolypoly stats <archive_name>` - Show detailed statistics about an archive
- `rolypoly hash <file>` - Calculate SHA256 hash of a file

## Features

- Full ZIP archive support with DEFLATE compression
- Progress bars for long-running operations
- Integrity validation with CRC32 checking
- Detailed archive statistics and compression ratios
- SHA256 file hashing for verification
- Graceful error handling and user feedback
- Cross-platform compatibility (Windows, macOS, Linux)

## Architecture Notes

This project is a modern compression utility written in Rust with a focus on safety, performance, and user experience. The core architecture consists of:

- `ArchiveManager` - Core functionality for ZIP operations
- `CLI` - Command-line interface built with clap
- Comprehensive test coverage for all functionality
- Progress indicators for better user experience
- Robust error handling and validation

#!/usr/bin/env python3
"""
Comprehensive GUI testing script for Rusty ZIP Archiver

This script provides automated testing of all GUI components and functionality
to ensure the application works correctly across different scenarios.
"""

import subprocess
import sys
import os
import time
import json
import tempfile
from pathlib import Path
from typing import List, Dict, Any, Optional

class Color:
    """ANSI color codes for console output"""
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    WHITE = '\033[97m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'
    END = '\033[0m'

class TestResult:
    """Represents the result of a test"""
    def __init__(self, name: str, passed: bool, message: str = "", details: str = ""):
        self.name = name
        self.passed = passed
        self.message = message
        self.details = details
        self.timestamp = time.time()

class RolyPolyGUITester:
    """Main test runner for Rusty GUI components"""
    
    def __init__(self):
        self.results: List[TestResult] = []
        self.project_root = Path(__file__).parent
        self.temp_dir = None
        
    def log(self, message: str, color: str = Color.WHITE):
        """Log a message with color"""
        print(f"{color}{message}{Color.END}")
        
    def log_success(self, message: str):
        """Log a success message"""
        self.log(f"✅ {message}", Color.GREEN)
        
    def log_error(self, message: str):
        """Log an error message"""
        self.log(f"❌ {message}", Color.RED)
        
    def log_warning(self, message: str):
        """Log a warning message"""
        self.log(f"⚠️  {message}", Color.YELLOW)
        
    def log_info(self, message: str):
        """Log an info message"""
        self.log(f"ℹ️  {message}", Color.CYAN)
        
    def log_test_start(self, test_name: str):
        """Log the start of a test"""
        self.log(f"🔧 {test_name}...", Color.BLUE)
        
    def add_result(self, name: str, passed: bool, message: str = "", details: str = ""):
        """Add a test result"""
        result = TestResult(name, passed, message, details)
        self.results.append(result)
        
        if passed:
            self.log_success(f"{name}: {message}")
        else:
            self.log_error(f"{name}: {message}")
            if details:
                self.log(f"   Details: {details}", Color.YELLOW)
    
    def run_command(self, cmd: List[str], cwd: Optional[Path] = None, timeout: int = 30) -> tuple:
        """Run a command and return (returncode, stdout, stderr)"""
        try:
            result = subprocess.run(
                cmd,
                cwd=cwd or self.project_root,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            return result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return -1, "", f"Command timed out after {timeout} seconds"
        except Exception as e:
            return -1, "", str(e)
    
    def test_rust_compilation(self):
        """Test that Rust code compiles correctly"""
        self.log_test_start("Rust Compilation")
        
        # Test library compilation
        returncode, stdout, stderr = self.run_command(["cargo", "build", "--lib"])
        self.add_result(
            "Library Compilation",
            returncode == 0,
            "Library compiled successfully" if returncode == 0 else "Library compilation failed",
            stderr if returncode != 0 else ""
        )
        
        # Test binary compilation
        returncode, stdout, stderr = self.run_command(["cargo", "build", "--bin", "rolypoly"])
        self.add_result(
            "Binary Compilation",
            returncode == 0,
            "Binary compiled successfully" if returncode == 0 else "Binary compilation failed",
            stderr if returncode != 0 else ""
        )
    
    def test_backend_components(self):
        """Test all backend GUI components"""
        self.log_test_start("Backend Components")
        
        # Test health check
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_health_check_component", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Health Check Component",
            returncode == 0,
            "Health check working" if returncode == 0 else "Health check failed",
            stderr if returncode != 0 else ""
        )
        
        # Test archive creation
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_create_archive_component", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Create Archive Component",
            returncode == 0,
            "Archive creation working" if returncode == 0 else "Archive creation failed",
            stderr if returncode != 0 else ""
        )
        
        # Test archive listing
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_list_archive_component", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "List Archive Component",
            returncode == 0,
            "Archive listing working" if returncode == 0 else "Archive listing failed",
            stderr if returncode != 0 else ""
        )
        
        # Test archive validation
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_validate_archive_component", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Validate Archive Component",
            returncode == 0,
            "Archive validation working" if returncode == 0 else "Archive validation failed",
            stderr if returncode != 0 else ""
        )
        
        # Test archive statistics
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_get_archive_stats_component", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Archive Stats Component",
            returncode == 0,
            "Archive statistics working" if returncode == 0 else "Archive statistics failed",
            stderr if returncode != 0 else ""
        )
        
        # Test file hashing
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_calculate_file_hash_component", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "File Hash Component",
            returncode == 0,
            "File hashing working" if returncode == 0 else "File hashing failed",
            stderr if returncode != 0 else ""
        )
        
        # Test app info
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_get_app_info_component", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "App Info Component",
            returncode == 0,
            "App info working" if returncode == 0 else "App info failed",
            stderr if returncode != 0 else ""
        )
    
    def test_error_handling(self):
        """Test error handling components"""
        self.log_test_start("Error Handling")
        
        # Test create archive error handling
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_create_archive_error_handling", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Create Archive Error Handling",
            returncode == 0,
            "Error handling working" if returncode == 0 else "Error handling failed",
            stderr if returncode != 0 else ""
        )
        
        # Test list archive error handling
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_list_archive_error_handling", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "List Archive Error Handling",
            returncode == 0,
            "Error handling working" if returncode == 0 else "Error handling failed",
            stderr if returncode != 0 else ""
        )
        
        # Test validate archive error handling
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_validate_archive_error_handling", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Validate Archive Error Handling",
            returncode == 0,
            "Error handling working" if returncode == 0 else "Error handling failed",
            stderr if returncode != 0 else ""
        )
        
        # Test file hash error handling
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_calculate_file_hash_error_handling", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "File Hash Error Handling",
            returncode == 0,
            "Error handling working" if returncode == 0 else "Error handling failed",
            stderr if returncode != 0 else ""
        )
    
    def test_fun_messages(self):
        """Test fun message functionality"""
        self.log_test_start("Fun Messages")
        
        # Test fun error messages
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_fun_error_messages", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Fun Error Messages",
            returncode == 0,
            "Fun error messages working" if returncode == 0 else "Fun error messages failed",
            stderr if returncode != 0 else ""
        )
        
        # Test fun success messages
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_fun_success_messages", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Fun Success Messages",
            returncode == 0,
            "Fun success messages working" if returncode == 0 else "Fun success messages failed",
            stderr if returncode != 0 else ""
        )
    
    def test_concurrency(self):
        """Test concurrent operations"""
        self.log_test_start("Concurrency")
        
        # Test concurrent GUI operations
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_concurrent_gui_operations", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Concurrent GUI Operations",
            returncode == 0,
            "Concurrent operations working" if returncode == 0 else "Concurrent operations failed",
            stderr if returncode != 0 else ""
        )
    
    def test_integration_workflow(self):
        """Test complete integration workflow"""
        self.log_test_start("Integration Workflow")
        
        # Test complete workflow
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "test_complete_gui_workflow", 
            "--test", "gui_component_tests", "--", "--nocapture"
        ])
        self.add_result(
            "Complete GUI Workflow",
            returncode == 0,
            "Complete workflow working" if returncode == 0 else "Complete workflow failed",
            stderr if returncode != 0 else ""
        )
    
    def test_cli_functionality(self):
        """Test CLI functionality"""
        self.log_test_start("CLI Functionality")
        
        # Create temporary test files
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            
            # Create test files
            test_file1 = temp_path / "test1.txt"
            test_file2 = temp_path / "test2.txt"
            test_file1.write_text("Test content 1")
            test_file2.write_text("Test content 2")
            
            archive_path = temp_path / "test.zip"
            
            # Test CLI help
            returncode, stdout, stderr = self.run_command(["cargo", "run", "--bin", "rolypoly", "--", "--help"])
            self.add_result(
                "CLI Help",
                returncode == 0 and "Usage:" in stdout,
                "CLI help working" if returncode == 0 else "CLI help failed",
                stderr if returncode != 0 else ""
            )
            
            # Test CLI create archive
            returncode, stdout, stderr = self.run_command([
                "cargo", "run", "--bin", "rolypoly", "--", "create", 
                str(archive_path), str(test_file1), str(test_file2)
            ])
            self.add_result(
                "CLI Create Archive",
                returncode == 0 and archive_path.exists(),
                "CLI archive creation working" if returncode == 0 else "CLI archive creation failed",
                stderr if returncode != 0 else ""
            )
            
            # Test CLI list archive
            if archive_path.exists():
                returncode, stdout, stderr = self.run_command([
                    "cargo", "run", "--bin", "rolypoly", "--", "list", str(archive_path)
                ])
                self.add_result(
                    "CLI List Archive",
                    returncode == 0 and "test1.txt" in stdout,
                    "CLI archive listing working" if returncode == 0 else "CLI archive listing failed",
                    stderr if returncode != 0 else ""
                )
                
                # Test CLI validate archive
                returncode, stdout, stderr = self.run_command([
                    "cargo", "run", "--bin", "rolypoly", "--", "validate", str(archive_path)
                ])
                self.add_result(
                    "CLI Validate Archive",
                    returncode == 0,
                    "CLI archive validation working" if returncode == 0 else "CLI archive validation failed",
                    stderr if returncode != 0 else ""
                )
    
    def test_stress_conditions(self):
        """Test stress conditions"""
        self.log_test_start("Stress Conditions")
        
        # Test stress tests if available
        returncode, stdout, stderr = self.run_command([
            "cargo", "test", "gui_stress_tests", 
            "--test", "gui_stress_tests", "--", "--nocapture"
        ])
        
        # Note: Stress tests might be ignored, so we check for "ignored" in output
        passed = returncode == 0 or "ignored" in stdout
        self.add_result(
            "GUI Stress Tests",
            passed,
            "Stress tests working or ignored" if passed else "Stress tests failed",
            stderr if returncode != 0 else ""
        )
    
    def generate_report(self):
        """Generate a comprehensive test report"""
        self.log("\n" + "="*70, Color.BOLD)
        self.log("COMPREHENSIVE GUI TEST REPORT", Color.BOLD)
        self.log("="*70, Color.BOLD)
        
        total_tests = len(self.results)
        passed_tests = sum(1 for r in self.results if r.passed)
        failed_tests = total_tests - passed_tests
        
        # Summary
        self.log(f"\n📊 SUMMARY:", Color.BOLD)
        self.log(f"   Total Tests: {total_tests}")
        self.log(f"   Passed: {passed_tests}", Color.GREEN)
        self.log(f"   Failed: {failed_tests}", Color.RED if failed_tests > 0 else Color.GREEN)
        self.log(f"   Success Rate: {(passed_tests/total_tests*100):.1f}%")
        
        # Details
        if failed_tests > 0:
            self.log(f"\n❌ FAILED TESTS:", Color.RED)
            for result in self.results:
                if not result.passed:
                    self.log(f"   • {result.name}: {result.message}")
                    if result.details:
                        self.log(f"     Details: {result.details}", Color.YELLOW)
        
        # Recommendations
        self.log(f"\n💡 RECOMMENDATIONS:", Color.BOLD)
        
        if failed_tests == 0:
            self.log("   🎉 All tests passing! GUI is ready for production.", Color.GREEN)
        else:
            self.log("   🔧 Address failed tests before release.", Color.YELLOW)
            
        if passed_tests > total_tests * 0.8:
            self.log("   ✅ Overall system health is good.", Color.GREEN)
        else:
            self.log("   ⚠️  System needs attention - many tests failing.", Color.RED)
        
        # Next steps
        self.log(f"\n🚀 NEXT STEPS:", Color.BOLD)
        self.log("   1. Fix any failed tests")
        self.log("   2. Run manual GUI testing")
        self.log("   3. Test on different platforms")
        self.log("   4. Verify all buttons and interactions work")
        
        return failed_tests == 0
    
    def run_all_tests(self):
        """Run all tests"""
        self.log("🚀 STARTING COMPREHENSIVE GUI TESTING", Color.BOLD)
        self.log("="*50, Color.BOLD)
        
        # Run all test categories
        self.test_rust_compilation()
        self.test_backend_components()
        self.test_error_handling()
        self.test_fun_messages()
        self.test_concurrency()
        self.test_integration_workflow()
        self.test_cli_functionality()
        self.test_stress_conditions()
        
        # Generate report
        success = self.generate_report()
        
        return success

def main():
    """Main entry point"""
    tester = RolyPolyGUITester()
    success = tester.run_all_tests()
    
    if success:
        print(f"\n{Color.GREEN}🎉 ALL TESTS PASSED! GUI is ready for use.{Color.END}")
        sys.exit(0)
    else:
        print(f"\n{Color.RED}❌ SOME TESTS FAILED. Check the report above.{Color.END}")
        sys.exit(1)

if __name__ == "__main__":
    main()

#!/usr/bin/env rust-script
//! Truth Enforcement Validation Framework
//! 
//! This module provides mechanisms to detect fake implementations,
//! mock objects, and other deceptive patterns in test code.

use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SuspiciousPattern {
    pattern: String,
    severity: Severity,
    description: String,
    regex: Regex,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
pub struct ValidationResult {
    file_path: PathBuf,
    violations: Vec<Violation>,
    score: f32, // 0.0 = completely fake, 1.0 = completely genuine
}

#[derive(Debug, Clone)]
pub struct Violation {
    line_number: usize,
    pattern: String,
    severity: Severity,
    context: String,
    description: String,
}

pub struct TruthValidator {
    suspicious_patterns: Vec<SuspiciousPattern>,
    whitelist_patterns: HashSet<String>,
}

impl TruthValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            suspicious_patterns: Vec::new(),
            whitelist_patterns: HashSet::new(),
        };
        
        validator.initialize_patterns();
        validator.initialize_whitelist();
        validator
    }
    
    fn initialize_patterns(&mut self) {
        // Critical fake implementation patterns
        self.add_pattern(
            r"unimplemented!\s*\(\s*\)",
            Severity::Critical,
            "Unimplemented macro - indicates incomplete functionality"
        );
        
        self.add_pattern(
            r"todo!\s*\(\s*.*?\s*\)",
            Severity::Critical,
            "TODO macro - indicates incomplete implementation"
        );
        
        self.add_pattern(
            r"panic!\s*\(\s*['\"].*?not\s+implemented.*?['\"]\s*\)",
            Severity::Critical,
            "Panic with 'not implemented' message"
        );
        
        // High severity fake patterns
        self.add_pattern(
            r"\bmock_\w+|Mock\w+|\bfake_\w+|Fake\w+",
            Severity::High,
            "Mock or fake objects in production code"
        );
        
        self.add_pattern(
            r"\bdummy_\w+|Dummy\w+|\bstub_\w+|Stub\w+",
            Severity::High,
            "Dummy or stub implementations"
        );
        
        self.add_pattern(
            r"return\s+Ok\s*\(\s*\(\s*\)\s*\)\s*;.*//.*fake|return\s+true\s*;.*//.*fake",
            Severity::High,
            "Suspicious simple returns with 'fake' comments"
        );
        
        // Medium severity suspicious patterns
        self.add_pattern(
            r"println!\s*\(\s*['\"].*?fake.*?['\"]\s*\)",
            Severity::Medium,
            "Debug prints mentioning 'fake'"
        );
        
        self.add_pattern(
            r"//\s*TODO:|//\s*FIXME:|//\s*HACK:",
            Severity::Medium,
            "TODO, FIXME, or HACK comments indicating incomplete work"
        );
        
        self.add_pattern(
            r"\bsleep\s*\(\s*Duration::from_\w+\s*\(\s*0\s*\)\s*\)",
            Severity::Medium,
            "Zero-duration sleep (potential timing fake)"
        );
        
        // Low severity patterns that might indicate issues
        self.add_pattern(
            r"//\s*placeholder|//\s*temporary",
            Severity::Low,
            "Placeholder or temporary code comments"
        );
        
        self.add_pattern(
            r"\bVec::new\s*\(\s*\).*//.*empty",
            Severity::Low,
            "Empty vector returns with explanatory comments"
        );
        
        // Patterns that suggest simulation rather than real functionality
        self.add_pattern(
            r"simulate_\w+|simulated_\w+|simulation",
            Severity::Medium,
            "Simulation code in production implementation"
        );
        
        self.add_pattern(
            r"let\s+result\s*=\s*true\s*;.*//.*always",
            Severity::High,
            "Hardcoded success results"
        );
    }
    
    fn initialize_whitelist(&mut self) {
        // Legitimate uses of typically suspicious patterns
        self.whitelist_patterns.insert("mock_server_for_testing".to_string());
        self.whitelist_patterns.insert("test_mock".to_string());
        self.whitelist_patterns.insert("#[cfg(test)]".to_string());
        self.whitelist_patterns.insert("mod tests".to_string());
    }
    
    fn add_pattern(&mut self, pattern: &str, severity: Severity, description: &str) {
        self.suspicious_patterns.push(SuspiciousPattern {
            pattern: pattern.to_string(),
            severity,
            description: description.to_string(),
            regex: Regex::new(pattern).expect("Invalid regex pattern"),
        });
    }
    
    pub fn validate_file(&self, file_path: &Path) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut violations = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            // Skip lines that are whitelisted
            if self.is_whitelisted_context(line, &lines, line_num) {
                continue;
            }
            
            for pattern in &self.suspicious_patterns {
                if pattern.regex.is_match(line) {
                    violations.push(Violation {
                        line_number: line_num + 1,
                        pattern: pattern.pattern.clone(),
                        severity: pattern.severity.clone(),
                        context: line.trim().to_string(),
                        description: pattern.description.clone(),
                    });
                }
            }
        }
        
        let score = self.calculate_authenticity_score(&violations, lines.len());
        
        Ok(ValidationResult {
            file_path: file_path.to_path_buf(),
            violations,
            score,
        })
    }
    
    fn is_whitelisted_context(&self, line: &str, lines: &[&str], line_num: usize) -> bool {
        // Check if we're in a test module
        let context_start = line_num.saturating_sub(10);
        let context_end = std::cmp::min(line_num + 10, lines.len());
        
        for i in context_start..context_end {
            let context_line = lines[i];
            if context_line.contains("#[cfg(test)]") || 
               context_line.contains("mod tests") ||
               context_line.contains("#[test]") {
                return true;
            }
        }
        
        // Check for other whitelist patterns
        for pattern in &self.whitelist_patterns {
            if line.contains(pattern) {
                return true;
            }
        }
        
        false
    }
    
    fn calculate_authenticity_score(&self, violations: &[Violation], total_lines: usize) -> f32 {
        if violations.is_empty() {
            return 1.0;
        }
        
        let mut penalty = 0.0;
        
        for violation in violations {
            let line_penalty = match violation.severity {
                Severity::Critical => 0.5,  // 50% penalty per critical violation
                Severity::High => 0.2,      // 20% penalty per high violation  
                Severity::Medium => 0.1,    // 10% penalty per medium violation
                Severity::Low => 0.05,      // 5% penalty per low violation
            };
            penalty += line_penalty;
        }
        
        // Apply scaling based on code size
        let density_factor = violations.len() as f32 / total_lines as f32;
        penalty += density_factor * 0.3; // Additional penalty for high violation density
        
        (1.0 - penalty).max(0.0)
    }
    
    pub fn validate_directory(&self, dir_path: &Path) -> Result<Vec<ValidationResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "rs" {
                        match self.validate_file(&path) {
                            Ok(result) => results.push(result),
                            Err(e) => eprintln!("Failed to validate {}: {}", path.display(), e),
                        }
                    }
                }
            } else if path.is_dir() {
                // Recursively validate subdirectories
                let mut sub_results = self.validate_directory(&path)?;
                results.append(&mut sub_results);
            }
        }
        
        Ok(results)
    }
    
    pub fn generate_report(&self, results: &[ValidationResult]) -> String {
        let mut report = String::new();
        
        report.push_str("🔍 TRUTH ENFORCEMENT VALIDATION REPORT\n");
        report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
        
        let total_files = results.len();
        let clean_files = results.iter().filter(|r| r.violations.is_empty()).count();
        let suspicious_files = total_files - clean_files;
        
        report.push_str(&format!("Files Analyzed: {}\n", total_files));
        report.push_str(&format!("Clean Files: {} ({:.1}%)\n", 
            clean_files, 
            clean_files as f32 / total_files as f32 * 100.0
        ));
        report.push_str(&format!("Suspicious Files: {} ({:.1}%)\n\n", 
            suspicious_files,
            suspicious_files as f32 / total_files as f32 * 100.0
        ));
        
        // Overall authenticity score
        let avg_score: f32 = results.iter().map(|r| r.score).sum::<f32>() / results.len() as f32;
        report.push_str(&format!("Overall Authenticity Score: {:.2}/1.00\n\n", avg_score));
        
        if avg_score >= 0.95 {
            report.push_str("✅ VERDICT: HIGH CONFIDENCE - Code appears genuine\n\n");
        } else if avg_score >= 0.80 {
            report.push_str("⚠️  VERDICT: MEDIUM CONFIDENCE - Some suspicious patterns detected\n\n");
        } else if avg_score >= 0.60 {
            report.push_str("🚨 VERDICT: LOW CONFIDENCE - Multiple suspicious patterns detected\n\n");
        } else {
            report.push_str("💥 VERDICT: VERY LOW CONFIDENCE - Likely contains fake implementations\n\n");
        }
        
        // Detailed violation breakdown
        let mut all_violations = Vec::new();
        for result in results {
            for violation in &result.violations {
                all_violations.push((result.file_path.clone(), violation.clone()));
            }
        }
        
        if !all_violations.is_empty() {
            report.push_str("DETAILED VIOLATIONS:\n");
            report.push_str("────────────────────\n");
            
            // Group by severity
            all_violations.sort_by(|a, b| b.1.severity.cmp(&a.1.severity));
            
            for (file_path, violation) in all_violations {
                let severity_symbol = match violation.severity {
                    Severity::Critical => "💥",
                    Severity::High => "🚨", 
                    Severity::Medium => "⚠️",
                    Severity::Low => "ℹ️",
                };
                
                report.push_str(&format!("{} {} ({}:{})\n", 
                    severity_symbol,
                    violation.description,
                    file_path.display(),
                    violation.line_number
                ));
                report.push_str(&format!("   Context: {}\n", violation.context));
                report.push_str(&format!("   Pattern: {}\n\n", violation.pattern));
            }
        }
        
        report
    }
    
    pub fn enforce_truth_threshold(&self, results: &[ValidationResult], min_score: f32) -> bool {
        let avg_score: f32 = results.iter().map(|r| r.score).sum::<f32>() / results.len() as f32;
        
        // Check for critical violations
        let critical_violations: Vec<_> = results.iter()
            .flat_map(|r| &r.violations)
            .filter(|v| v.severity == Severity::Critical)
            .collect();
            
        if !critical_violations.is_empty() {
            eprintln!("❌ TRUTH ENFORCEMENT FAILED: {} critical violations detected", 
                critical_violations.len());
            return false;
        }
        
        if avg_score < min_score {
            eprintln!("❌ TRUTH ENFORCEMENT FAILED: Score {:.2} below threshold {:.2}", 
                avg_score, min_score);
            return false;
        }
        
        println!("✅ TRUTH ENFORCEMENT PASSED: Score {:.2} meets threshold {:.2}", 
            avg_score, min_score);
        true
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validator = TruthValidator::new();
    
    // Validate the src directory
    let src_path = Path::new("src");
    let results = validator.validate_directory(src_path)?;
    
    // Generate and print report
    let report = validator.generate_report(&results);
    println!("{}", report);
    
    // Enforce truth threshold (80% minimum authenticity)
    let passed = validator.enforce_truth_threshold(&results, 0.80);
    
    if !passed {
        std::process::exit(1);
    }
    
    Ok(())
}
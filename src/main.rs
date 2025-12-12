// Cargo.toml dependencies:
// [package]
// name = "cache_manager"
// version = "1.0.0"
// edition = "2021"
//
// [dependencies]
// eframe = "0.24"
// egui = "0.24"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// dirs = "5.0"
//
// [profile.release]
// opt-level = 3
// lto = true
// codegen-units = 1
// strip = true

// Hide console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    cache_threshold_gb: f32,
    auto_clean_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cache_threshold_gb: 10.0,
            auto_clean_enabled: true,
        }
    }
}

struct CacheManager {
    config: Config,
    last_clean_time: Arc<Mutex<Option<Instant>>>,
    cache_size_gb: f32,
    is_cleaning: bool,
    status_message: String,
    cache_dirs: Vec<PathBuf>,
}

impl CacheManager {
    fn new() -> Self {
        let config = Self::load_config().unwrap_or_default();
        let cache_dirs = Self::get_cache_directories();
        
        Self {
            config,
            last_clean_time: Arc::new(Mutex::new(None)),
            cache_size_gb: 0.0,
            is_cleaning: false,
            status_message: String::from("Ready"),
            cache_dirs,
        }
    }

    fn get_cache_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        
        // Windows cache directories only
        if cfg!(target_os = "windows") {
            // Windows Temp
            dirs.push(std::env::temp_dir());
            
            // Local AppData Temp
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                dirs.push(PathBuf::from(local_appdata).join("Temp"));
            }
            
            // Internet Explorer Cache
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                dirs.push(PathBuf::from(local_appdata).join("Microsoft").join("Windows").join("INetCache"));
            }
            
            // Windows Update Cache
            dirs.push(PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download"));
            
            // Prefetch
            dirs.push(PathBuf::from("C:\\Windows\\Prefetch"));
            
            // Browser caches
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                let local = PathBuf::from(local_appdata);
                
                // Chrome Cache
                dirs.push(local.join("Google").join("Chrome").join("User Data").join("Default").join("Cache"));
                
                // Edge Cache
                dirs.push(local.join("Microsoft").join("Edge").join("User Data").join("Default").join("Cache"));
                
                // Firefox Cache
                if let Ok(appdata) = std::env::var("APPDATA") {
                    let firefox_profiles = PathBuf::from(appdata).join("Mozilla").join("Firefox").join("Profiles");
                    if let Ok(entries) = fs::read_dir(&firefox_profiles) {
                        for entry in entries.flatten() {
                            dirs.push(entry.path().join("cache2"));
                        }
                    }
                }
            }
        }
        
        // Filter only existing directories
        dirs.into_iter().filter(|d| d.exists()).collect()
    }

    fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        let data = std::fs::read_to_string(config_path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        let data = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(config_path, data)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        let mut path = if cfg!(target_os = "windows") {
            dirs::config_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        } else {
            std::env::current_dir().unwrap_or_default()
        };
        
        if cfg!(target_os = "windows") {
            path.push("CacheManager");
            std::fs::create_dir_all(&path).ok();
        }
        
        path.push("cache_manager_config.json");
        path
    }

    fn calculate_dir_size(path: &PathBuf) -> u64 {
        let mut total_size: u64 = 0;
        
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    } else if metadata.is_dir() {
                        // Recursive for subdirectories
                        total_size += Self::calculate_dir_size(&entry.path());
                    }
                }
            }
        }
        
        total_size
    }

    fn get_cache_size(&mut self) -> f32 {
        let mut total_size: u64 = 0;
        
        for cache_dir in &self.cache_dirs {
            total_size += Self::calculate_dir_size(cache_dir);
        }

        (total_size as f32) / (1024.0 * 1024.0 * 1024.0) // Convert to GB
    }

    fn clean_directory(path: &PathBuf, cleaned_count: &mut u32, cleaned_size: &mut u64) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let entry_path = entry.path();
                    
                    if metadata.is_file() {
                        let file_size = metadata.len();
                        // Try to delete file, skip if in use
                        if fs::remove_file(&entry_path).is_ok() {
                            *cleaned_count += 1;
                            *cleaned_size += file_size;
                        }
                    } else if metadata.is_dir() {
                        // Clean subdirectories recursively
                        Self::clean_directory(&entry_path, cleaned_count, cleaned_size);
                        // Try to remove empty directory
                        fs::remove_dir(&entry_path).ok();
                    }
                }
            }
        }
    }

    fn clean_cache(&mut self) {
        self.is_cleaning = true;
        self.status_message = String::from("Cleaning cache...");

        let mut cleaned_count = 0;
        let mut cleaned_size: u64 = 0;

        // Clean each cache directory
        for cache_dir in &self.cache_dirs {
            Self::clean_directory(cache_dir, &mut cleaned_count, &mut cleaned_size);
        }

        let cleaned_gb = (cleaned_size as f32) / (1024.0 * 1024.0 * 1024.0);
        self.status_message = format!("✅ Cleaned {} files ({:.2} GB)", cleaned_count, cleaned_gb);

        *self.last_clean_time.lock().unwrap() = Some(Instant::now());
        self.is_cleaning = false;
        self.cache_size_gb = self.get_cache_size();
    }

    fn should_auto_clean(&self) -> bool {
        if !self.config.auto_clean_enabled {
            return false;
        }

        // Check if 30 seconds have passed since last clean
        if let Some(last_time) = *self.last_clean_time.lock().unwrap() {
            if last_time.elapsed() < Duration::from_secs(30) {
                return false;
            }
        }

        // Check if cache exceeds threshold
        self.cache_size_gb >= self.config.cache_threshold_gb
    }
}

impl eframe::App for CacheManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update cache size
        self.cache_size_gb = self.get_cache_size();

        // Auto clean if needed
        if self.should_auto_clean() && !self.is_cleaning {
            self.clean_cache();
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(20, 20, 20)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    
                    ui.heading(
                        egui::RichText::new("🗂️ Cache Manager")
                            .size(28.0)
                            .color(egui::Color32::from_rgb(100, 200, 255))
                    );
                    
                    ui.add_space(30.0);

                    // Display current cache size
                    ui.label(
                        egui::RichText::new(format!("📊 Current cache size: {:.2} GB", self.cache_size_gb))
                            .size(18.0)
                            .color(egui::Color32::WHITE)
                    );

                    ui.add_space(10.0);

                    // Status message
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(150, 150, 150))
                    );

                    ui.add_space(30.0);

                    // Threshold slider
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🎚️ Cache threshold (GB):")
                                .size(16.0)
                                .color(egui::Color32::WHITE)
                        );
                    });

                    ui.add_space(10.0);

                    let mut threshold = self.config.cache_threshold_gb;
                    ui.add(
                        egui::Slider::new(&mut threshold, 1.0..=100.0)
                            .text("GB")
                            .step_by(1.0)
                    );
                    self.config.cache_threshold_gb = threshold;

                    ui.add_space(10.0);
                    
                    ui.label(
                        egui::RichText::new(format!("Auto-clean when cache reaches {:.0} GB", threshold))
                            .size(13.0)
                            .color(egui::Color32::from_rgb(180, 180, 180))
                    );

                    ui.add_space(20.0);

                    // Auto clean checkbox
                    ui.checkbox(
                        &mut self.config.auto_clean_enabled,
                        egui::RichText::new("🔄 Enable auto-clean after 30 seconds")
                            .size(15.0)
                            .color(egui::Color32::WHITE)
                    );

                    ui.add_space(30.0);

                    // Save button
                    if ui.add_sized(
                        [200.0, 40.0],
                        egui::Button::new(
                            egui::RichText::new("💾 Save Configuration")
                                .size(16.0)
                        )
                    ).clicked() {
                        match self.save_config() {
                            Ok(_) => self.status_message = String::from("✅ Configuration saved"),
                            Err(e) => self.status_message = format!("❌ Error: {}", e),
                        }
                    }

                    ui.add_space(10.0);

                    // Manual clean button
                    if ui.add_sized(
                        [200.0, 40.0],
                        egui::Button::new(
                            egui::RichText::new("🧹 Clean Cache Now")
                                .size(16.0)
                        )
                    ).clicked() {
                        self.clean_cache();
                    }

                    ui.add_space(20.0);

                    // Progress indicator
                    if self.is_cleaning {
                        ui.spinner();
                    }

                    ui.add_space(20.0);

                    // Info about last clean time
                    if let Some(last_time) = *self.last_clean_time.lock().unwrap() {
                        let elapsed = last_time.elapsed().as_secs();
                        ui.label(
                            egui::RichText::new(format!("⏱️ Last cleaned: {} seconds ago", elapsed))
                                .size(13.0)
                                .color(egui::Color32::from_rgb(150, 150, 150))
                        );
                    }
                });
            });

        // Request repaint to update UI
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 550.0])
            .with_min_inner_size([400.0, 450.0])
            .with_resizable(true)
            .with_title("Cache Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "Cache Manager",
        options,
        Box::new(|_cc| Box::new(CacheManager::new())),
    )
}
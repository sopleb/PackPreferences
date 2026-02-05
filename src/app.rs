use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::about::AboutScreen;
use crate::config::Config;
use crate::discovery::{self, CharacterFile, FileType};
use crate::esi;
use crate::process::{self, DetectedPrefix};
use crate::settings;
use crate::theme;

/// Represents a selectable item (either a character or user/account)
#[derive(Clone)]
struct SelectableItem {
    file_idx: usize,
    id: u64,
    display_name: String,
    is_default: bool,
}

pub struct PackPreferencesApp {
    config: Config,
    detected_prefixes: Vec<DetectedPrefix>,
    selected_prefix: Option<PathBuf>,
    settings_dir: Option<PathBuf>,
    character_files: Vec<CharacterFile>,
    character_names: HashMap<u64, String>,
    source_selection: Option<usize>,
    target_selections: HashSet<usize>,
    dry_run_mode: bool,
    status_messages: Vec<String>,
    show_backup_manager: bool,
    backups: Vec<PathBuf>,
    pending_confirmation: Option<PendingAction>,
    sync_mode: SyncMode,
    about: AboutScreen,
}

#[derive(Clone, Copy, PartialEq)]
enum SyncMode {
    Characters, // Sync core_char_* files
    Users,      // Sync core_user_* files (accounts)
}

#[derive(Clone)]
enum PendingAction {
    Sync,
    Restore(PathBuf),
}

impl PackPreferencesApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load().unwrap_or_default();

        // Apply custom theme
        theme::apply_pack_theme(&cc.egui_ctx);

        // Set initial window position from config
        if let Some(ctx) = cc.egui_ctx.clone().into() {
            let ctx: egui::Context = ctx;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                config.window_x,
                config.window_y,
            )));
        }

        let mut app = Self {
            config,
            detected_prefixes: Vec::new(),
            selected_prefix: None,
            settings_dir: None,
            character_files: Vec::new(),
            character_names: HashMap::new(),
            source_selection: None,
            target_selections: HashSet::new(),
            dry_run_mode: true,
            status_messages: Vec::new(),
            show_backup_manager: false,
            backups: Vec::new(),
            pending_confirmation: None,
            sync_mode: SyncMode::Users,
            about: AboutScreen::new(),
        };

        // Auto-detect on startup
        app.scan_for_eve();

        app
    }

    fn scan_for_eve(&mut self) {
        self.status_messages.clear();
        self.status_messages
            .push("Scanning for EVE processes...".to_string());

        match process::detect_eve_prefixes() {
            Ok(prefixes) => {
                self.detected_prefixes = prefixes;
                if let Some(first) = self.detected_prefixes.first() {
                    self.status_messages.push(format!(
                        "Found {} EVE instance(s)",
                        self.detected_prefixes.len()
                    ));
                    self.select_prefix(first.path.clone());
                } else {
                    self.status_messages
                        .push("No running EVE instances found".to_string());
                    // Try to use last known prefix
                    if let Some(ref last_path) = self.config.last_prefix_path {
                        let path = PathBuf::from(last_path);
                        if path.exists() {
                            self.status_messages
                                .push("Using last known prefix".to_string());
                            self.select_prefix(path);
                        }
                    }
                }
            }
            Err(e) => {
                self.status_messages.push(format!("Scan failed: {}", e));
            }
        }
    }

    fn select_prefix(&mut self, prefix: PathBuf) {
        self.selected_prefix = Some(prefix.clone());
        self.config.last_prefix_path = Some(prefix.to_string_lossy().to_string());

        // Find settings directories
        match process::find_settings_dirs(&prefix) {
            Ok(dirs) => {
                if let Some(first_dir) = dirs.first() {
                    self.settings_dir = Some(first_dir.clone());
                    self.load_character_files();
                } else {
                    self.status_messages
                        .push("No settings directories found".to_string());
                }
            }
            Err(e) => {
                self.status_messages
                    .push(format!("Failed to find settings: {}", e));
            }
        }

        let _ = self.config.save();
    }

    fn load_character_files(&mut self) {
        let Some(ref settings_dir) = self.settings_dir else {
            return;
        };

        match discovery::discover_character_files(settings_dir) {
            Ok(files) => {
                let char_count = files
                    .iter()
                    .filter(|f| f.file_type == FileType::Character)
                    .count();
                let user_count = files
                    .iter()
                    .filter(|f| f.file_type == FileType::User)
                    .count();
                self.status_messages.push(format!(
                    "Found {} character files, {} user files",
                    char_count, user_count
                ));
                self.character_files = files;
                self.source_selection = None;
                self.target_selections.clear();
                self.resolve_names();

                // Auto-select sync mode based on available files
                if char_count <= 1 && user_count > 1 {
                    self.sync_mode = SyncMode::Users;
                } else if char_count > 1 {
                    self.sync_mode = SyncMode::Characters;
                }
            }
            Err(e) => {
                self.status_messages
                    .push(format!("Failed to load files: {}", e));
            }
        }
    }

    fn resolve_names(&mut self) {
        // Get unique character IDs from char files only (user IDs are not character IDs)
        let char_ids: Vec<u64> = self
            .character_files
            .iter()
            .filter(|f| f.file_type == FileType::Character)
            .map(|f| f.character_id)
            .collect();

        // First, populate from cache
        for id in &char_ids {
            if let Some(name) = self.config.get_cached_name(*id) {
                self.character_names.insert(*id, name.clone());
            }
        }

        // Resolve uncached names
        match esi::resolve_with_cache(&char_ids, &self.config.character_name_cache) {
            Ok(new_names) => {
                for (id, name) in new_names {
                    self.character_names.insert(id, name.clone());
                    self.config.cache_character_name(id, name);
                }
                let _ = self.config.save();

                let resolved = self.character_names.len();
                let total = char_ids.len();
                if total > 0 {
                    self.status_messages
                        .push(format!("Resolved {}/{} character names", resolved, total));
                }
            }
            Err(e) => {
                self.status_messages
                    .push(format!("Name resolution failed: {}", e));
            }
        }
    }

    fn browse_for_prefix(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Wine Prefix (drive_c directory)")
            .pick_folder()
        {
            self.select_prefix(path);
        }
    }

    fn get_selectable_items(&self) -> Vec<SelectableItem> {
        let target_type = match self.sync_mode {
            SyncMode::Characters => FileType::Character,
            SyncMode::Users => FileType::User,
        };

        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for (idx, file) in self.character_files.iter().enumerate() {
            if file.file_type == target_type
                && !seen.contains(&(file.character_id, file.is_default))
            {
                seen.insert((file.character_id, file.is_default));

                let display_name = if file.is_default {
                    match target_type {
                        FileType::Character => "Default (new characters)".to_string(),
                        FileType::User => "Default (new accounts)".to_string(),
                    }
                } else if target_type == FileType::Character {
                    self.character_names
                        .get(&file.character_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Character {}", file.character_id))
                } else {
                    format!("Account {}", file.character_id)
                };

                result.push(SelectableItem {
                    file_idx: idx,
                    id: file.character_id,
                    display_name,
                    is_default: file.is_default,
                });
            }
        }

        result
    }

    fn select_all_targets(&mut self) {
        let items = self.get_selectable_items();
        for item in items {
            if Some(item.file_idx) != self.source_selection {
                self.target_selections.insert(item.file_idx);
            }
        }
    }

    fn select_none_targets(&mut self) {
        self.target_selections.clear();
    }

    fn perform_sync(&mut self) {
        let Some(source_idx) = self.source_selection else {
            self.status_messages.push("No source selected".to_string());
            return;
        };

        if self.target_selections.is_empty() {
            self.status_messages.push("No targets selected".to_string());
            return;
        }

        let Some(ref settings_dir) = self.settings_dir else {
            self.status_messages
                .push("No settings directory".to_string());
            return;
        };

        // Create backup first (unless dry run)
        if !self.dry_run_mode {
            match settings::create_backup(settings_dir) {
                Ok(backup_path) => {
                    self.status_messages.push(format!(
                        "Created backup: {}",
                        backup_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    ));
                }
                Err(e) => {
                    self.status_messages.push(format!("Backup failed: {}", e));
                    return;
                }
            }
        }

        // Get the source file
        let source_file = &self.character_files[source_idx];

        // Get target files (same file type as source)
        let target_ids: HashSet<u64> = self
            .target_selections
            .iter()
            .map(|&i| self.character_files[i].character_id)
            .collect();

        let target_files: Vec<&CharacterFile> = self
            .character_files
            .iter()
            .filter(|f| {
                f.file_type == source_file.file_type
                    && target_ids.contains(&f.character_id)
                    && f.character_id != source_file.character_id
            })
            .collect();

        // Sync
        match settings::sync_settings(source_file, &target_files, self.dry_run_mode) {
            Ok(results) => {
                let mut total_synced = 0;
                for result in results {
                    if result.success {
                        total_synced += 1;
                        let target_name = result
                            .target_file
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        self.status_messages
                            .push(format!("{}: {}", result.message, target_name));
                    } else {
                        self.status_messages.push(result.message);
                    }
                }

                let action = if self.dry_run_mode {
                    "Would sync"
                } else {
                    "Synced"
                };
                self.status_messages
                    .push(format!("{} {} files", action, total_synced));
            }
            Err(e) => {
                self.status_messages.push(format!("Sync error: {}", e));
            }
        }
    }

    fn load_backups(&mut self) {
        if let Some(ref settings_dir) = self.settings_dir {
            match settings::list_backups(settings_dir) {
                Ok(backups) => {
                    self.backups = backups;
                }
                Err(e) => {
                    self.status_messages
                        .push(format!("Failed to list backups: {}", e));
                }
            }
        }
    }

    fn restore_backup(&mut self, backup_path: PathBuf) {
        let Some(ref settings_dir) = self.settings_dir else {
            return;
        };

        match settings::restore_backup(&backup_path, settings_dir) {
            Ok(()) => {
                self.status_messages
                    .push("Backup restored successfully".to_string());
                self.load_character_files();
            }
            Err(e) => {
                self.status_messages.push(format!("Restore failed: {}", e));
            }
        }
    }
}

impl eframe::App for PackPreferencesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show about screen if open
        self.about.show(ctx);

        // Handle pending confirmations
        if let Some(action) = self.pending_confirmation.clone() {
            egui::Window::new("Confirm")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    match &action {
                        PendingAction::Sync => {
                            ui.label("Are you sure you want to sync settings?");
                            if !self.dry_run_mode {
                                ui.label("This will overwrite target settings.");
                                ui.label("A backup will be created first.");
                            }
                        }
                        PendingAction::Restore(path) => {
                            ui.label("Are you sure you want to restore this backup?");
                            ui.label(format!(
                                "{}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            ));
                        }
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            match action {
                                PendingAction::Sync => self.perform_sync(),
                                PendingAction::Restore(path) => self.restore_backup(path),
                            }
                            self.pending_confirmation = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_confirmation = None;
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // App title with about button
            ui.horizontal(|ui| {
                theme::styled_title(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("About").clicked() {
                        self.about.open = true;
                    }
                });
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Prefix selection
            ui.horizontal(|ui| {
                ui.label("Wine Prefix:");
                let prefix_text = self
                    .selected_prefix
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Not selected".to_string());
                ui.add(egui::TextEdit::singleline(&mut prefix_text.as_str()).desired_width(300.0));

                if ui.button("Browse").clicked() {
                    self.browse_for_prefix();
                }
                if ui.button("Scan").clicked() {
                    self.scan_for_eve();
                }
            });

            ui.separator();

            // Sync mode selection
            ui.horizontal(|ui| {
                ui.label("Sync Mode:");
                let char_count = self
                    .character_files
                    .iter()
                    .filter(|f| f.file_type == FileType::Character)
                    .count();
                let user_count = self
                    .character_files
                    .iter()
                    .filter(|f| f.file_type == FileType::User)
                    .count();

                if ui
                    .radio_value(
                        &mut self.sync_mode,
                        SyncMode::Characters,
                        format!("Characters ({})", char_count),
                    )
                    .changed()
                {
                    self.source_selection = None;
                    self.target_selections.clear();
                }
                if ui
                    .radio_value(
                        &mut self.sync_mode,
                        SyncMode::Users,
                        format!("Accounts/Users ({})", user_count),
                    )
                    .changed()
                {
                    self.source_selection = None;
                    self.target_selections.clear();
                }
            });

            ui.separator();

            let items = self.get_selectable_items();
            let type_label = match self.sync_mode {
                SyncMode::Characters => "Character",
                SyncMode::Users => "Account",
            };

            // Source selection (exclude default files - can't copy FROM defaults)
            ui.heading(format!("Source {} (copy FROM):", type_label));
            egui::ScrollArea::vertical()
                .id_salt("source_scroll")
                .max_height(120.0)
                .show(ui, |ui| {
                    let source_items: Vec<_> = items.iter().filter(|i| !i.is_default).collect();
                    if source_items.is_empty() {
                        ui.label(format!("No {} files found", type_label.to_lowercase()));
                    }
                    for item in source_items {
                        let selected = self.source_selection == Some(item.file_idx);
                        if ui
                            .radio(selected, format!("{}  [{}]", item.display_name, item.id))
                            .clicked()
                        {
                            self.source_selection = Some(item.file_idx);
                            self.target_selections.remove(&item.file_idx);
                        }
                    }
                });

            ui.separator();

            // Target selection
            ui.heading(format!("Target {}s (copy TO):", type_label));
            ui.horizontal(|ui| {
                if ui.button("Select All").clicked() {
                    self.select_all_targets();
                }
                if ui.button("Select None").clicked() {
                    self.select_none_targets();
                }
            });

            egui::ScrollArea::vertical()
                .id_salt("target_scroll")
                .max_height(150.0)
                .show(ui, |ui| {
                    for item in &items {
                        // Can't select source as target
                        if self.source_selection == Some(item.file_idx) {
                            continue;
                        }

                        let label = if item.is_default {
                            item.display_name.clone()
                        } else {
                            format!("{}  [{}]", item.display_name, item.id)
                        };

                        let mut selected = self.target_selections.contains(&item.file_idx);
                        if ui.checkbox(&mut selected, label).changed() {
                            if selected {
                                self.target_selections.insert(item.file_idx);
                            } else {
                                self.target_selections.remove(&item.file_idx);
                            }
                        }
                    }
                });

            ui.separator();

            // Options and actions
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.dry_run_mode, "Dry Run Mode");
                ui.add_space(20.0);

                let sync_enabled =
                    self.source_selection.is_some() && !self.target_selections.is_empty();
                if ui
                    .add_enabled(sync_enabled, egui::Button::new("Sync Settings"))
                    .clicked()
                {
                    self.pending_confirmation = Some(PendingAction::Sync);
                }

                if ui.button("Manage Backups").clicked() {
                    self.show_backup_manager = !self.show_backup_manager;
                    if self.show_backup_manager {
                        self.load_backups();
                    }
                }
            });

            // Backup manager
            if self.show_backup_manager {
                ui.separator();
                ui.heading("Backups:");
                egui::ScrollArea::vertical()
                    .id_salt("backup_scroll")
                    .max_height(100.0)
                    .show(ui, |ui| {
                        if self.backups.is_empty() {
                            ui.label("No backups found");
                        }
                        for backup in self.backups.clone() {
                            ui.horizontal(|ui| {
                                let name = backup
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                ui.label(&name);
                                if ui.button("Restore").clicked() {
                                    self.pending_confirmation =
                                        Some(PendingAction::Restore(backup));
                                }
                            });
                        }
                    });
            }

            ui.separator();

            // Status area
            ui.heading("Status:");
            egui::ScrollArea::vertical()
                .id_salt("status_scroll")
                .max_height(100.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for msg in &self.status_messages {
                        ui.label(msg);
                    }
                });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.config.save();
    }
}

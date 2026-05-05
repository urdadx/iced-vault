use iced::widget::{Space, container, row, text};
use iced::{Element, Length, Size, Subscription, Task, Theme, event, time};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::components::dialogs::{DialogForms, DialogKind};
use crate::components::ui::{VaultTheme, load_fonts, p_font};
use crate::components::{dialogs, sidebar};
use crate::db::{Database, DbError};
use crate::export;
use crate::importers::{ImportFormat, ImportPreview, parse_import};
use crate::models::{CardPayload, ItemPayload, LoginPayload, Vault, VaultItem, VaultItemDetails};
use crate::screens;
use crate::screens::{AutoLockDuration, ExportFormat};
use directories::ProjectDirs;
use rusqlite::Connection;

const WINDOW_WIDTH: f32 = 820.0;
const WINDOW_HEIGHT: f32 = 570.0;
const SIDEBAR_WIDTH: f32 = 188.0;

pub(crate) fn run() -> iced::Result {
    let mut app = iced::application(VaultApp::default, VaultApp::update, VaultApp::view)
        .window(iced::window::Settings {
            size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            icon: app_icon(),
            ..Default::default()
        })
        .subscription(VaultApp::subscription)
        .theme(app_theme)
        .default_font(p_font())
        .centered();

    for font in load_fonts() {
        app = app.font(font);
    }

    app.run()
}

fn app_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(include_bytes!("icons/app_icon.png"), None).ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Screen {
    #[default]
    Browse,
    Import,
    Settings,
}

impl Screen {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Browse => "Browse",
            Self::Import => "Import",
            Self::Settings => "Settings",
        }
    }

    pub(crate) const fn icon_path(self) -> &'static str {
        match self {
            Self::Browse => "src/icons/browse_icon.svg",
            Self::Import => "src/icons/import_icon.svg",
            Self::Settings => "src/icons/settings_icon.svg",
        }
    }
}

pub(crate) struct VaultApp {
    database: Option<Database>,
    needs_master_setup: bool,
    active_screen: Screen,
    vault_records: Vec<Vault>,
    vaults: Vec<String>,
    vault_items: Vec<VaultItem>,
    selected_item: Option<VaultItemDetails>,
    show_password: bool,
    selected_vault: Option<String>,
    browse_query: String,
    new_item_type: Option<&'static str>,
    browse_sort: Option<&'static str>,
    active_dialog: Option<DialogKind>,
    new_vault_name: String,
    login_website: String,
    login_username: String,
    login_password: String,
    cardholder_name: String,
    card_number: String,
    expiration_date: String,
    security_code: String,
    import_result_message: String,
    master_password: String,
    master_password_confirmation: String,
    status_message: Option<String>,
    loading_message: Option<&'static str>,
    theme: VaultTheme,
    auto_lock: AutoLockDuration,
    export_format: ExportFormat,
    export_path: Option<String>,
    old_master_password: String,
    new_master_password: String,
    confirm_master_password: String,
    last_activity: std::time::Instant,
}

impl Default for VaultApp {
    fn default() -> Self {
        let (needs_master_setup, status_message) = match Database::needs_setup() {
            Ok(needs_setup) => (needs_setup, None),
            Err(error) => (true, Some(error.to_string())),
        };

        Self {
            database: None,
            needs_master_setup,
            active_screen: Screen::Browse,
            vault_records: Vec::new(),
            vaults: Vec::new(),
            vault_items: Vec::new(),
            selected_item: None,
            show_password: false,
            selected_vault: None,
            browse_query: String::new(),
            new_item_type: None,
            browse_sort: Some("Recent"),
            active_dialog: None,
            new_vault_name: String::new(),
            login_website: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            cardholder_name: String::new(),
            card_number: String::new(),
            expiration_date: String::new(),
            security_code: String::new(),
            import_result_message: String::new(),
            master_password: String::new(),
            master_password_confirmation: String::new(),
            status_message,
            loading_message: None,
            theme: VaultTheme::default(),
            auto_lock: AutoLockDuration::default(),
            export_format: ExportFormat::default(),
            export_path: None,
            old_master_password: String::new(),
            new_master_password: String::new(),
            confirm_master_password: String::new(),
            last_activity: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Navigate(Screen),
    VaultSelected(String),
    BrowseQueryChanged(String),
    NewItemTypeSelected(&'static str),
    BrowseSortSelected(&'static str),
    AddVaultPressed,
    AddVaultNameChanged(String),
    LoginWebsiteChanged(String),
    LoginUsernameChanged(String),
    LoginPasswordChanged(String),
    CardholderNameChanged(String),
    CardNumberChanged(String),
    ExpirationDateChanged(String),
    SecurityCodeChanged(String),
    CreateVaultConfirmed,
    CreateLoginConfirmed,
    CreateCardConfirmed,
    PerformUnlock,
    PerformCreateVault(String),
    PerformCreateLogin,
    PerformCreateCard,
    PerformUpdateLogin,
    PerformUpdateCard,
    UpdateLoginConfirmed,
    UpdateCardConfirmed,
    PerformLoadItems,
    BackToVaultList,
    EditSelectedItem,
    DeleteSelectedItem,
    PerformDeleteItem,
    CloseDialog,
    ThemePressed,
    OpenGithub,
    MasterPasswordChanged(String),
    MasterPasswordConfirmationChanged(String),
    UnlockConfirmed,
    OpenItem(String),
    TogglePasswordVisibility,
    AutoLockChanged(AutoLockDuration),
    ChangeMasterPasswordPressed,
    ExportFormatChanged(ExportFormat),
    ExportVault,
    ExportFilePicked(Option<PathBuf>),
    ExportCompleted(Option<String>, Result<usize, String>),
    InstallExtensionPressed,
    OldMasterPasswordChanged(String),
    NewMasterPasswordChanged(String),
    ConfirmMasterPasswordChanged(String),
    ChangeMasterPasswordConfirmed,
    ImportProviderSelected(&'static str),
    ImportFilePicked(Option<(&'static str, PathBuf)>),
    ImportParsed(Result<ParsedImport, String>),
    AutoLockTick,
    UserActivity,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedImport {
    preview: ImportPreview,
    file_name: Option<String>,
}

impl VaultApp {
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(screen) => {
                self.active_screen = screen;
                self.selected_item = None;
            }
            Message::VaultSelected(vault) => {
                self.selected_vault = Some(vault);
                self.selected_item = None;
                self.start_loading("Loading vault items...");
                return Task::done(Message::PerformLoadItems);
            }
            Message::BrowseQueryChanged(query) => self.browse_query = query,
            Message::NewItemTypeSelected(item_type) => match item_type {
                "Login" => {
                    self.new_item_type = None;
                    self.open_dialog(DialogKind::NewLogin);
                }
                "Card" => {
                    self.new_item_type = None;
                    self.open_dialog(DialogKind::NewCard);
                }
                _ => self.new_item_type = Some(item_type),
            },
            Message::BrowseSortSelected(sort) => self.browse_sort = Some(sort),
            Message::OpenItem(item_id) => self.open_item(&item_id),
            Message::BackToVaultList => {
                self.selected_item = None;
                self.show_password = false;
            }
            Message::TogglePasswordVisibility => self.show_password = !self.show_password,
            Message::EditSelectedItem => self.open_edit_selected_item(),
            Message::DeleteSelectedItem => {
                self.start_loading("Deleting item...");
                return Task::done(Message::PerformDeleteItem);
            }
            Message::AddVaultPressed => self.open_dialog(DialogKind::NewVault),
            Message::AddVaultNameChanged(name) => self.new_vault_name = name,
            Message::LoginWebsiteChanged(website) => self.login_website = website,
            Message::LoginUsernameChanged(username) => self.login_username = username,
            Message::LoginPasswordChanged(password) => self.login_password = password,
            Message::CardholderNameChanged(name) => self.cardholder_name = name,
            Message::CardNumberChanged(number) => self.card_number = number,
            Message::ExpirationDateChanged(date) => self.expiration_date = date,
            Message::SecurityCodeChanged(code) => self.security_code = code,
            Message::CreateVaultConfirmed => {
                let name = self.new_vault_name.trim();

                if !name.is_empty() {
                    let vault_name = String::from(name);
                    self.start_loading("Creating vault...");
                    return Task::done(Message::PerformCreateVault(vault_name));
                }
            }
            Message::CreateLoginConfirmed => {
                if !self.login_website.trim().is_empty()
                    && !self.login_username.trim().is_empty()
                    && !self.login_password.is_empty()
                {
                    self.start_loading("Saving login...");
                    return Task::done(Message::PerformCreateLogin);
                }
            }
            Message::CreateCardConfirmed => {
                if !self.cardholder_name.trim().is_empty()
                    && !self.card_number.trim().is_empty()
                    && !self.expiration_date.trim().is_empty()
                    && !self.security_code.trim().is_empty()
                {
                    self.start_loading("Saving card...");
                    return Task::done(Message::PerformCreateCard);
                }
            }
            Message::PerformCreateVault(vault_name) => self.create_vault(vault_name),
            Message::PerformCreateLogin => self.create_login(),
            Message::PerformCreateCard => self.create_card(),
            Message::PerformUpdateLogin => self.update_login(),
            Message::PerformUpdateCard => self.update_card(),
            Message::UpdateLoginConfirmed => {
                if !self.login_website.trim().is_empty()
                    && !self.login_username.trim().is_empty()
                    && !self.login_password.is_empty()
                {
                    self.start_loading("Saving changes...");
                    return Task::done(Message::PerformUpdateLogin);
                }
            }
            Message::UpdateCardConfirmed => {
                if !self.cardholder_name.trim().is_empty()
                    && !self.card_number.trim().is_empty()
                    && !self.expiration_date.trim().is_empty()
                    && !self.security_code.trim().is_empty()
                {
                    self.start_loading("Saving changes...");
                    return Task::done(Message::PerformUpdateCard);
                }
            }
            Message::PerformLoadItems => self.refresh_items(),
            Message::PerformDeleteItem => self.delete_selected_item(),
            Message::CloseDialog => {
                self.close_dialog();
                self.last_activity = Instant::now();
            }
            Message::UserActivity => self.last_activity = Instant::now(),
            Message::ThemePressed => self.theme = self.theme.toggle(),
            Message::OpenGithub => {
                let _ = open::that("https://github.com/urdadx/iced-vault");
            }
            Message::AutoLockChanged(duration) => {
                self.auto_lock = duration;
                if let Some(db) = &self.database {
                    let seconds = match duration {
                        AutoLockDuration::OneMinute => 60,
                        AutoLockDuration::FiveMinutes => 300,
                        AutoLockDuration::TenMinutes => 600,
                        AutoLockDuration::ThirtyMinutes => 1800,
                        AutoLockDuration::OneHour => 3600,
                        AutoLockDuration::Never => 0,
                    };
                    let _ = db.set_auto_lock_duration(seconds);
                }
            }
            Message::ChangeMasterPasswordPressed => {
                self.active_dialog = Some(DialogKind::ChangeMasterPassword);
            }
            Message::ExportFormatChanged(format) => self.export_format = format,
            Message::ExportVault => {
                if self.selected_vault_id().is_none() {
                    self.status_message = Some(String::from("Select a vault first."));
                    return Task::none();
                }
                self.status_message = None;
                self.start_loading("Opening export dialog...");
                let format = self.export_format;
                let vault_name = self
                    .selected_vault
                    .as_deref()
                    .unwrap_or("vault")
                    .to_string();
                return Task::perform(
                    pick_export_file(format, vault_name),
                    Message::ExportFilePicked,
                );
            }
            Message::ExportFilePicked(selection) => match selection {
                Some(path) => {
                    self.start_loading("Exporting vault...");
                    let vault_id = self.selected_vault_id().unwrap().to_owned();
                    let format = self.export_format;

                    let result = self.export_vault(&vault_id, format, &path);

                    self.finish_loading();
                    match result {
                        Ok(count) => {
                            self.status_message =
                                Some(format!("Exported {count} items successfully."));
                        }
                        Err(error) => {
                            self.status_message = Some(error);
                        }
                    }
                }
                None => {
                    self.finish_loading();
                    self.status_message = Some(String::from("Export cancelled."));
                }
            },
            Message::InstallExtensionPressed => {
                self.status_message = Some(String::from("Browser extension not implemented yet."));
            }
            Message::ExportCompleted(_path, result) => {
                self.finish_loading();
                match result {
                    Ok(count) => {
                        self.status_message = Some(format!("Exported {count} items successfully."));
                    }
                    Err(error) => {
                        self.status_message = Some(error);
                    }
                }
            }
            Message::OldMasterPasswordChanged(password) => self.old_master_password = password,
            Message::NewMasterPasswordChanged(password) => self.new_master_password = password,
            Message::ConfirmMasterPasswordChanged(password) => {
                self.confirm_master_password = password
            }
            Message::ChangeMasterPasswordConfirmed => {
                if self.new_master_password != self.confirm_master_password {
                    self.status_message = Some(String::from("Passwords do not match."));
                } else if self.new_master_password.len() < 8 {
                    self.status_message =
                        Some(String::from("Password must be at least 8 characters."));
                } else {
                    let result = self
                        .database
                        .as_ref()
                        .ok_or_else(|| String::from("Database not available."))
                        .and_then(|db| {
                            db.change_master_password(
                                &self.old_master_password,
                                &self.new_master_password,
                            )
                            .map_err(|e| e.to_string())
                        });

                    match result {
                        Ok(_) => {
                            self.close_dialog();
                            self.lock_vault();
                            self.status_message = Some(String::from(
                                "Master password changed. Please unlock again.",
                            ));
                        }
                        Err(error) => {
                            self.status_message = Some(error);
                        }
                    }
                }
            }
            Message::MasterPasswordChanged(password) => self.master_password = password,
            Message::MasterPasswordConfirmationChanged(password) => {
                self.master_password_confirmation = password
            }
            Message::UnlockConfirmed => {
                if self.unlock_requested() {
                    return Task::done(Message::PerformUnlock);
                }
            }
            Message::PerformUnlock => self.unlock_database(),
            Message::ImportProviderSelected(provider) => {
                if self.selected_vault_id().is_none() {
                    self.status_message = Some(String::from("Create a vault before importing."));
                    return Task::none();
                }

                self.status_message = None;
                self.start_loading("Opening file picker...");
                return Task::perform(pick_import_file(provider), Message::ImportFilePicked);
            }
            Message::ImportFilePicked(selection) => match selection {
                Some((provider, path)) => {
                    self.start_loading("Uploading and parsing file...");
                    return Task::perform(
                        read_and_parse_import(provider, path),
                        Message::ImportParsed,
                    );
                }
                None => {
                    self.finish_loading();
                    self.status_message = Some(String::from("Import cancelled."));
                }
            },
            Message::ImportParsed(result) => {
                self.complete_import(result);
            }
            Message::AutoLockTick => self.check_auto_lock(),
        }

        Task::none()
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        if self.database.is_none() {
            return Subscription::none();
        }

        let mut subscriptions = vec![event::listen_with(track_user_activity)];

        if self.auto_lock != AutoLockDuration::Never {
            subscriptions.push(time::every(Duration::from_secs(1)).map(|_| Message::AutoLockTick));
        }

        Subscription::batch(subscriptions)
    }

    fn check_auto_lock(&mut self) {
        if self.database.is_none() {
            return;
        }

        if self.auto_lock == AutoLockDuration::Never {
            self.last_activity = Instant::now();
            return;
        }

        let duration = match self.auto_lock {
            AutoLockDuration::OneMinute => std::time::Duration::from_secs(60),
            AutoLockDuration::FiveMinutes => std::time::Duration::from_secs(300),
            AutoLockDuration::TenMinutes => std::time::Duration::from_secs(600),
            AutoLockDuration::ThirtyMinutes => std::time::Duration::from_secs(1800),
            AutoLockDuration::OneHour => std::time::Duration::from_secs(3600),
            AutoLockDuration::Never => return,
        };

        if Instant::now().duration_since(self.last_activity) >= duration {
            self.lock_vault();
        }
    }

    fn lock_vault(&mut self) {
        self.database = None;
        self.vault_records.clear();
        self.vaults.clear();
        self.vault_items.clear();
        self.selected_item = None;
        self.master_password.clear();
        self.status_message = Some(String::from("Vault locked due to inactivity."));
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        if self.database.is_none() {
            return screens::master_password(
                self.needs_master_setup,
                &self.master_password,
                &self.master_password_confirmation,
                self.status_message.as_deref(),
                self.loading_message,
            );
        }

        let content = if let Some(message) = self
            .loading_message
            .filter(|_| self.active_dialog.is_none())
        {
            container(text(message).size(14))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            match self.active_screen {
                Screen::Browse => screens::browse(
                    &self.browse_query,
                    self.new_item_type,
                    self.browse_sort,
                    &self.vault_items,
                    self.selected_item.as_ref(),
                    self.show_password,
                ),
                Screen::Import => screens::import(self.status_message.as_deref()),
                Screen::Settings => screens::settings(screens::SettingsViewState {
                    current_theme: self.theme,
                    auto_lock: self.auto_lock,
                    export_format: self.export_format,
                    status_message: self.status_message.as_deref(),
                }),
            }
        };

        let content_panel = container(content)
            .padding([16.0, 18.0])
            .width(Length::Fill)
            .height(Length::Fill);

        let sidebar = sidebar::view(
            self.vaults.as_slice(),
            self.selected_vault.as_ref(),
            self.active_screen,
            self.theme,
        );

        let base = container(
            row![
                container(sidebar)
                    .width(SIDEBAR_WIDTH)
                    .height(Length::Fill)
                    .padding([0.0, 10.0]),
                container(Space::new().width(1))
                    .width(1)
                    .height(Length::Fill)
                    .style(sidebar::divider_style),
                content_panel,
            ]
            .spacing(0)
            .height(Length::Fill),
        )
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill);

        dialogs::wrap(
            base,
            self.active_dialog,
            DialogForms {
                new_vault_name: &self.new_vault_name,
                login_website: &self.login_website,
                login_username: &self.login_username,
                login_password: &self.login_password,
                cardholder_name: &self.cardholder_name,
                card_number: &self.card_number,
                expiration_date: &self.expiration_date,
                security_code: &self.security_code,
                import_result_message: &self.import_result_message,
                loading_message: self.loading_message,
                old_master_password: &self.old_master_password,
                new_master_password: &self.new_master_password,
                confirm_master_password: &self.confirm_master_password,
            },
        )
    }

    fn open_dialog(&mut self, dialog: DialogKind) {
        self.active_dialog = Some(dialog);

        match dialog {
            DialogKind::NewVault => self.new_vault_name.clear(),
            DialogKind::NewLogin => {
                self.login_website.clear();
                self.login_username.clear();
                self.login_password.clear();
            }
            DialogKind::EditLogin => {}
            DialogKind::NewCard => {
                self.cardholder_name.clear();
                self.card_number.clear();
                self.expiration_date.clear();
                self.security_code.clear();
            }
            DialogKind::EditCard => {}
            DialogKind::ImportComplete => {}
            DialogKind::ChangeMasterPassword => {
                self.old_master_password.clear();
                self.new_master_password.clear();
                self.confirm_master_password.clear();
            }
        }
    }

    fn close_dialog(&mut self) {
        if self.is_loading() {
            return;
        }

        self.active_dialog = None;
        self.new_vault_name.clear();
        self.login_website.clear();
        self.login_username.clear();
        self.login_password.clear();
        self.cardholder_name.clear();
        self.card_number.clear();
        self.expiration_date.clear();
        self.security_code.clear();
        self.old_master_password.clear();
        self.new_master_password.clear();
        self.confirm_master_password.clear();
    }

    fn complete_import(&mut self, result: Result<ParsedImport, String>) {
        match result.and_then(|parsed| self.import_preview(parsed)) {
            Ok(imported_count) => {
                self.finish_loading();
                self.refresh_items();
                self.import_result_message =
                    format!("{imported_count} passwords imported successfully.");
                self.open_dialog(DialogKind::ImportComplete);
            }
            Err(error) => {
                self.status_message = Some(error);
                self.finish_loading();
            }
        }
    }

    fn export_vault(
        &mut self,
        vault_id: &str,
        format: ExportFormat,
        path: &Path,
    ) -> Result<usize, String> {
        let Some(database) = &self.database else {
            return Err(String::from("Database is locked."));
        };

        let items = database
            .list_items_with_payloads(vault_id)
            .map_err(format_db_error)?;

        export::export_items(&items, format, path)
    }

    fn import_preview(&mut self, parsed: ParsedImport) -> Result<usize, String> {
        let Some(vault_id) = self.selected_vault_id().map(ToOwned::to_owned) else {
            return Err(String::from("Create a vault before importing."));
        };
        let Some(database) = &self.database else {
            return Err(String::from("Database is locked."));
        };

        let imported_count = parsed.preview.imported_count();

        database
            .create_items(&vault_id, parsed.preview.items, None)
            .map_err(format_db_error)?;

        let skipped_count = parsed.preview.skipped.len();
        let file_detail = parsed
            .file_name
            .map(|file_name| format!(" from {file_name}"))
            .unwrap_or_default();

        self.status_message = Some(if skipped_count == 0 {
            format!("Imported {imported_count} items{file_detail}.")
        } else {
            format!("Imported {imported_count} items{file_detail}; skipped {skipped_count} rows.")
        });

        Ok(imported_count)
    }

    fn unlock_database(&mut self) {
        match Database::open(&self.master_password) {
            Ok(database) => {
                self.database = Some(database);
                self.master_password.clear();
                self.master_password_confirmation.clear();
                self.status_message = None;
                self.needs_master_setup = false;
                self.refresh_vaults();
                self.load_settings();
                self.last_activity = Instant::now();
                self.finish_loading();
            }
            Err(DbError::InvalidMasterPassword) => {
                self.status_message = Some(String::from("You entered a wrong master password"));
                self.finish_loading();
            }
            Err(error) => {
                self.status_message = Some(error.to_string());
                self.finish_loading();
            }
        }
    }

    fn load_settings(&mut self) {
        if let Some(db) = &self.database {
            if let Ok(duration) = db.get_auto_lock_duration() {
                self.auto_lock = match duration {
                    60 => AutoLockDuration::OneMinute,
                    300 => AutoLockDuration::FiveMinutes,
                    600 => AutoLockDuration::TenMinutes,
                    1800 => AutoLockDuration::ThirtyMinutes,
                    3600 => AutoLockDuration::OneHour,
                    0 => AutoLockDuration::Never,
                    _ => AutoLockDuration::TenMinutes,
                };
            }
        }
    }

    fn unlock_requested(&mut self) -> bool {
        if let Err(message) = self.validate_master_password() {
            self.status_message = Some(message);
            return false;
        }

        self.status_message = None;
        self.start_loading(if self.needs_master_setup {
            "Creating encrypted vault..."
        } else {
            "Unlocking..."
        });

        true
    }

    fn validate_master_password(&self) -> Result<(), String> {
        if self.master_password.is_empty() {
            return Err(String::from("Enter a master password."));
        }

        if self.needs_master_setup {
            if self.master_password.chars().count() < 8 {
                return Err(String::from(
                    "Use at least 8 characters for the master password.",
                ));
            }

            if self.master_password != self.master_password_confirmation {
                return Err(String::from("Master passwords do not match."));
            }
        }

        Ok(())
    }

    fn create_vault(&mut self, vault_name: String) {
        let result = self
            .database
            .as_ref()
            .ok_or_else(|| String::from("Database is locked."))
            .and_then(|database| database.create_vault(&vault_name).map_err(format_db_error));

        match result {
            Ok(vault) => {
                self.selected_vault = Some(vault.name);
                self.finish_loading();
                self.close_dialog();
                self.refresh_vaults();
            }
            Err(error) => {
                self.status_message = Some(error);
                self.finish_loading();
            }
        }
    }

    fn create_login(&mut self) {
        let Some(vault_id) = self.selected_vault_id().map(ToOwned::to_owned) else {
            self.status_message = Some(String::from("Create a vault before adding items."));
            self.finish_loading();
            return;
        };
        let payload = ItemPayload::Login(LoginPayload::manual(
            self.login_website.trim().to_owned(),
            self.login_username.trim().to_owned(),
            self.login_password.clone(),
        ));

        self.create_item(&vault_id, payload);
    }

    fn create_card(&mut self) {
        let Some(vault_id) = self.selected_vault_id().map(ToOwned::to_owned) else {
            self.status_message = Some(String::from("Create a vault before adding items."));
            self.finish_loading();
            return;
        };
        let payload = ItemPayload::Card(CardPayload::manual(
            self.cardholder_name.trim().to_owned(),
            self.card_number.trim().to_owned(),
            self.expiration_date.trim().to_owned(),
            self.security_code.trim().to_owned(),
        ));

        self.create_item(&vault_id, payload);
    }

    fn open_item(&mut self, item_id: &str) {
        let result = self
            .database
            .as_ref()
            .ok_or_else(|| String::from("Database is locked."))
            .and_then(|database| database.get_item(item_id).map_err(format_db_error));

        match result {
            Ok(Some(item)) => {
                self.status_message = None;
                self.selected_item = Some(item);
            }
            Ok(None) => self.status_message = Some(String::from("Item not found.")),
            Err(error) => self.status_message = Some(error),
        }
    }

    fn open_edit_selected_item(&mut self) {
        let Some(item) = self.selected_item.clone() else {
            return;
        };

        match item.payload {
            ItemPayload::Login(login) => {
                self.login_website = login.urls.first().cloned().unwrap_or(item.title);
                self.login_username = login.username;
                self.login_password = login.password;
                self.active_dialog = Some(DialogKind::EditLogin);
            }
            ItemPayload::Card(card) => {
                self.cardholder_name = card.cardholder_name;
                self.card_number = card.number;
                self.expiration_date = card.expiration_date;
                self.security_code = card.security_code;
                self.active_dialog = Some(DialogKind::EditCard);
            }
        }
    }

    fn update_login(&mut self) {
        let Some(selected_item) = self.selected_item.clone() else {
            self.finish_loading();
            return;
        };

        let payload = match selected_item.payload {
            ItemPayload::Login(mut login) => {
                login.name = self.login_website.trim().to_owned();
                login.urls = vec![self.login_website.trim().to_owned()];
                login.username = self.login_username.trim().to_owned();
                login.password = self.login_password.clone();
                ItemPayload::Login(login)
            }
            ItemPayload::Card(_) => {
                self.finish_loading();
                return;
            }
        };

        self.update_selected_item(&selected_item.id, payload);
    }

    fn update_card(&mut self) {
        let Some(selected_item) = self.selected_item.clone() else {
            self.finish_loading();
            return;
        };

        let payload = match selected_item.payload {
            ItemPayload::Card(mut card) => {
                card.cardholder_name = self.cardholder_name.trim().to_owned();
                card.number = self.card_number.trim().to_owned();
                card.expiration_date = self.expiration_date.trim().to_owned();
                card.security_code = self.security_code.trim().to_owned();
                ItemPayload::Card(card)
            }
            ItemPayload::Login(_) => {
                self.finish_loading();
                return;
            }
        };

        self.update_selected_item(&selected_item.id, payload);
    }

    fn update_selected_item(&mut self, item_id: &str, payload: ItemPayload) {
        let result = self
            .database
            .as_ref()
            .ok_or_else(|| String::from("Database is locked."))
            .and_then(|database| {
                database
                    .update_item(item_id, payload)
                    .map_err(format_db_error)
            });

        match result {
            Ok(Some(item)) => {
                self.finish_loading();
                self.close_dialog();
                self.selected_item = Some(item);
                self.refresh_items();
            }
            Ok(None) => {
                self.status_message = Some(String::from("Item not found."));
                self.finish_loading();
            }
            Err(error) => {
                self.status_message = Some(error);
                self.finish_loading();
            }
        }
    }

    fn delete_selected_item(&mut self) {
        let Some(item_id) = self.selected_item.as_ref().map(|item| item.id.clone()) else {
            self.finish_loading();
            return;
        };

        let result = self
            .database
            .as_ref()
            .ok_or_else(|| String::from("Database is locked."))
            .and_then(|database| database.delete_item(&item_id).map_err(format_db_error));

        match result {
            Ok(()) => {
                self.selected_item = None;
                self.refresh_items();
            }
            Err(error) => {
                self.status_message = Some(error);
                self.finish_loading();
            }
        }
    }

    fn create_item(&mut self, vault_id: &str, payload: ItemPayload) {
        let result = self
            .database
            .as_ref()
            .ok_or_else(|| String::from("Database is locked."))
            .and_then(|database| {
                database
                    .create_item(vault_id, payload, None)
                    .map_err(format_db_error)
            });

        match result {
            Ok(_) => {
                self.finish_loading();
                self.close_dialog();
                self.refresh_items();
            }
            Err(error) => {
                self.status_message = Some(error);
                self.finish_loading();
            }
        }
    }

    fn refresh_vaults(&mut self) {
        let result = self.load_vaults();

        if let Err(error) = result {
            self.status_message = Some(format_db_error(error));
        }

        self.finish_loading();
    }

    fn load_vaults(&mut self) -> Result<(), DbError> {
        let Some(database) = &self.database else {
            return Ok(());
        };
        let mut vaults = database.list_vaults()?;

        if vaults.is_empty() {
            vaults.push(database.create_vault("Personal")?);
        }

        let selected_exists = self
            .selected_vault
            .as_ref()
            .is_some_and(|selected| vaults.iter().any(|vault| vault.name == *selected));

        if !selected_exists {
            self.selected_vault = vaults.first().map(|vault| vault.name.clone());
        }

        self.vaults = vaults.iter().map(|vault| vault.name.clone()).collect();
        self.vault_records = vaults;
        self.load_items()?;

        Ok(())
    }

    fn refresh_items(&mut self) {
        if let Err(error) = self.load_items() {
            self.status_message = Some(format_db_error(error));
        }

        self.finish_loading();
    }

    fn load_items(&mut self) -> Result<(), DbError> {
        let Some(database) = &self.database else {
            return Ok(());
        };
        let vault_id = self.selected_vault_id();
        self.vault_items = database.list_items(vault_id)?;

        Ok(())
    }

    fn selected_vault_id(&self) -> Option<&str> {
        let selected = self.selected_vault.as_ref()?;
        self.vault_records
            .iter()
            .find(|vault| vault.name == *selected)
            .map(|vault| vault.id.as_str())
    }

    fn start_loading(&mut self, message: &'static str) {
        self.loading_message = Some(message);
    }

    fn finish_loading(&mut self) {
        self.loading_message = None;
    }

    fn is_loading(&self) -> bool {
        self.loading_message.is_some()
    }
}

fn app_theme(app: &VaultApp) -> Theme {
    app.theme.to_iced_theme()
}

fn track_user_activity(
    event: iced::Event,
    _status: event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        iced::Event::Keyboard(_)
        | iced::Event::Mouse(_)
        | iced::Event::Touch(_)
        | iced::Event::InputMethod(_) => Some(Message::UserActivity),
        iced::Event::Window(_) => None,
    }
}

fn format_db_error(error: DbError) -> String {
    error.to_string()
}

async fn pick_import_file(provider: &'static str) -> Option<(&'static str, PathBuf)> {
    let dialog = rfd::AsyncFileDialog::new()
        .set_title(format!("Select {} export file", provider_label(provider)));
    let dialog = match provider {
        "bitwarden" => dialog.add_filter("Bitwarden export", &["csv", "json"]),
        _ => dialog.add_filter("CSV export", &["csv"]),
    };

    dialog
        .pick_file()
        .await
        .map(|file| (provider, file.path().to_path_buf()))
}

async fn read_and_parse_import(
    provider: &'static str,
    path: PathBuf,
) -> Result<ParsedImport, String> {
    let format = import_format_for_file(provider, &path)?;
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("Could not read import file: {error}"))?;
    let preview = parse_import(format, &contents).map_err(|error| error.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned);

    Ok(ParsedImport { preview, file_name })
}

fn import_format_for_file(provider: &'static str, path: &Path) -> Result<ImportFormat, String> {
    match provider {
        "bitwarden" => match file_extension(path).as_deref() {
            Some("json") => Ok(ImportFormat::BitwardenJson),
            Some("csv") => Ok(ImportFormat::BitwardenCsv),
            _ => Err(String::from("Select a Bitwarden CSV or JSON export.")),
        },
        "chrome" => csv_format(path, ImportFormat::ChromeCsv, "Chrome"),
        "edge" => csv_format(path, ImportFormat::EdgeCsv, "Edge"),
        "1Password" => csv_format(path, ImportFormat::OnePasswordCsv, "1Password"),
        "proton" => csv_format(path, ImportFormat::ProtonCsv, "Proton"),
        "safari" => csv_format(path, ImportFormat::SafariCsv, "Safari"),
        _ => Err(String::from("Unsupported import provider.")),
    }
}

fn csv_format(
    path: &Path,
    format: ImportFormat,
    provider_label: &'static str,
) -> Result<ImportFormat, String> {
    match file_extension(path).as_deref() {
        Some("csv") => Ok(format),
        _ => Err(format!("Select a {provider_label} CSV export.")),
    }
}

fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
}

fn provider_label(provider: &'static str) -> &'static str {
    match provider {
        "bitwarden" => "Bitwarden",
        "chrome" => "Chrome",
        "edge" => "Edge",
        "1Password" => "1Password",
        "proton" => "Proton",
        "safari" => "Safari",
        _ => provider,
    }
}

async fn pick_export_file(format: ExportFormat, vault_name: String) -> Option<PathBuf> {
    let (title, ext) = match format {
        ExportFormat::Csv => ("Export as CSV", "csv"),
        ExportFormat::Zip => ("Export as ZIP", "zip"),
    };
    let file_name = export_file_name(&vault_name, ext);

    rfd::AsyncFileDialog::new()
        .set_title(title)
        .add_filter("Export file", &[ext])
        .set_file_name(&file_name)
        .save_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}

fn export_file_name(vault_name: &str, ext: &str) -> String {
    let sanitized = vault_name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let base = if sanitized.is_empty() {
        "vault"
    } else {
        &sanitized
    };

    format!("{base}.{ext}")
}

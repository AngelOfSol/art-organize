use std::io::Result;
use winreg::{enums::HKEY_CURRENT_USER, RegKey};
pub fn install() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes =
        hkcu.open_subkey_with_flags(r"SOFTWARE\Classes", winreg::enums::KEY_ALL_ACCESS)?;

    create_background_folder_submenu(&classes)?;
    create_background_folder_context_menu(&classes)?;
    create_file_context_menu(&classes)?;
    Ok(())
}

fn make_command(subcommand: &'static str) -> Result<String> {
    let local_exe = std::env::current_exe()?;
    Ok(format!(
        r#""{}" {}"#,
        local_exe.to_str().unwrap(),
        subcommand
    ))
}

fn create_file_context_menu(classes: &RegKey) -> Result<()> {
    create_shell_entry(
        &classes,
        ShellEntry {
            name: r"*\shell\ArtOrganize",
            label: "Add to ArtOrganize",
            command: make_command(r#"add "%1""#)?,
        },
    )?;

    Ok(())
}

fn create_background_folder_submenu(classes: &RegKey) -> Result<()> {
    let (background, _) = classes.create_subkey(r"Directory\Background\shell\ArtOrganize")?;
    background.set_value(r"MUIVerb", &String::from(r"ArtOrganize"))?;
    background.set_value(
        r"ExtendedSubCommandsKey",
        &String::from(r"ArtOrganize.Background"),
    )?;
    Ok(())
}
fn create_background_folder_context_menu(classes: &RegKey) -> Result<()> {
    let (background, _) = classes.create_subkey(r"ArtOrganize.Background\shell")?;

    create_shell_entry(
        &background,
        ShellEntry {
            name: "init",
            label: "Initialize DB",
            command: make_command(r#"init "%V""#)?,
        },
    )?;

    Ok(())
}

struct ShellEntry {
    name: &'static str,
    label: &'static str,
    command: String,
}

fn create_shell_entry(background: &RegKey, entry: ShellEntry) -> Result<()> {
    let (backup, _) = background.create_subkey(entry.name)?;
    backup.set_value("", &String::from(entry.label))?;
    let (backup_command, _) = backup.create_subkey("command")?;
    backup_command.set_value("", &entry.command)?;
    Ok(())
}

pub fn remove() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes =
        hkcu.open_subkey_with_flags(r"SOFTWARE\Classes", winreg::enums::KEY_ALL_ACCESS)?;
    let _ = classes.delete_subkey_all(r"Directory\Background\shell\ArtOrganize");
    let _ = classes.delete_subkey_all(r"ArtOrganize.Background");
    let _ = classes.delete_subkey_all(r"*\shell\ArtOrganize");

    Ok(())
}

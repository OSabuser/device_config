//! # Модуль отображения меню
//!
//! ## Навигация
//! > Для навигации по меню используется клавиши стрелка вверх/вниз
//! Выбор эелемента в меню происходит клавишей Enter
//! `В случае отсутствия данных клавиш в системе, можно эмулировать их с помощью библиотеки enigo`
//!
//! ## Параметры меню
//! > Параметры меню подтягиваются из файла-схемы TOML с помощью крейта config_lib
use config_lib::device_config::DeviceConfig;
use crossterm::style::Color;
use inquire::Select;
use terminal_menu::{TerminalMenuItem, button, label, menu, mut_menu, run};

pub enum MainMenuStates {
    ConfigurationState,
    ExitState,
}

/// Отображение главного меню
pub fn show_main_dialog(config: &mut DeviceConfig) -> Result<MainMenuStates, String> {
    // Header
    let mut menu_structure = generate_menu_header("Меню настроек индикатора");

    // Текущие значения
    menu_structure.extend(generate_current_values(config)?);

    // Элементы меню
    menu_structure.extend(generate_menu_items(config)?);

    let main_menu = menu(menu_structure);

    // Отрисовка и навигация по меню
    run(&main_menu);

    // Получение названия выбранного пункта меню
    let mut_menu_instance = mut_menu(&main_menu);
    let selected_menu_item = mut_menu_instance.selected_item_name();

    // Получение ключа параметра в HashMap, соответствующего названию пункта меню
    let selected_parameter_name = config
        .get_parameters_names()?
        .into_iter()
        .find_map(|par_name| {
            // Получение ключа параметра в HashMap, в случае если описание параметра совпадает с названием пункта меню
            if let Ok(par_desc) = config.get_parameter_description(&par_name) {
                return if par_desc == selected_menu_item {
                    Some(par_name)
                } else {
                    None
                };
            }
            None
        });

    if let Some(key) = selected_parameter_name {
        show_configuration_dialog(config, &key)?;
        return Ok(MainMenuStates::ConfigurationState);
    }

    // Сохранение настроек в TOML-файл схему
    config.save_parameters_values()?;

    return Ok(MainMenuStates::ExitState);
}

/// Отображение диалога для выбора значения параметра
fn show_configuration_dialog(config: &mut DeviceConfig, par_name: &str) -> Result<(), String> {
    let possible_values = config.get_parameter_possible_values(par_name)?;
    let answer = Select::new("Выберите значение параметра", possible_values.clone()).prompt();

    match answer {
        Ok(selection) => {
            config.set_parameter_value(par_name, selection)?;
            Ok(())
        }
        Err(e) => return Err(e.to_string()),
    }
}

/// Генерация структуры меню: Титул
fn generate_menu_header(title: &str) -> Vec<TerminalMenuItem> {
    vec![
        label("----------------------").colorize(Color::DarkGreen),
        label(title).colorize(Color::DarkGreen),
        label(format!(
            "{} v{}",
            env!("CARGO_CRATE_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .colorize(Color::DarkGreen),
        label(format!("{}", env!("CARGO_PKG_AUTHORS"))).colorize(Color::DarkGreen),
        label("-----------------------").colorize(Color::DarkGreen),
    ]
}

/// Генерация структуры меню: Текущие значения настраиваемых параметров
fn generate_current_values(config: &DeviceConfig) -> Result<Vec<TerminalMenuItem>, String> {
    let parameter_names = config.get_parameters_names()?;

    let mut menu_structure = vec![];

    for parameter_name in &parameter_names {
        let parameter_value = config.get_parameter_value(&parameter_name)?;
        let parameter_description = config.get_parameter_description(&parameter_name)?;
        menu_structure.push(
            label(format!("{}: {}", parameter_description, parameter_value))
                .colorize(Color::DarkYellow),
        );
    }

    menu_structure.push(label("-----------------------").colorize(Color::DarkYellow));

    Ok(menu_structure)
}

/// Генерация структуры меню: Элементы меню
fn generate_menu_items(config: &DeviceConfig) -> Result<Vec<TerminalMenuItem>, String> {
    let parameter_names = config.get_parameters_names()?;

    let mut menu_structure = vec![];

    menu_structure.push(label("-----------------------").colorize(Color::DarkGrey));

    // Элементы меню; имя каждого параметра соответствует полю description
    let mut menu_structure = menu_structure;
    for parameter_name in &parameter_names {
        let parameter_description = config.get_parameter_description(&parameter_name)?;
        menu_structure.push(button(parameter_description).colorize(Color::White));
    }
    menu_structure.push(button("Выход").colorize(Color::White));

    menu_structure.push(label("-----------------------").colorize(Color::DarkGrey));

    Ok(menu_structure)
}

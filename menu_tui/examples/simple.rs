use menu_tui::menu_navigation::NavigationManager;
use menu_tui::menu_process::DeviceMenu;

use std::thread;
use std::time::Duration;

/// Запуск меню в режиме имитации нажатия кнопок
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создаём меню
    let mut menu = DeviceMenu::new(
        "menu_tui/examples/simple_config.toml",
        "menu_tui/assets/style.toml",
    );

    // Получаем менеджер навигации для управления от GPIO
    let nav_manager = menu.get_navigation_manager();

    menu.show_main_menu();

    menu.launch_idling_watchdog(5);

    // Симуляция GPIO обработчиков в отдельных потоках
    simulate_gpio_handlers(nav_manager);

    // Запускаем главный цикл обработки событий
    menu.run();

    let device_config = menu.get_schema_config()?;

    println!("Итоговая конфигурация:");
    for parameter in device_config.get_parameters_names()? {
        let value = device_config.get_parameter_value(&parameter).unwrap();
        println!("{}: {}", parameter, value);
    }

    return Ok(());
}

// Симуляция обработчиков GPIO кнопок
fn simulate_gpio_handlers(nav_manager: NavigationManager) {
    // Кнопка 1 (GPIO): Навигация вниз
    let nav_down = nav_manager.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));

        loop {
            thread::sleep(Duration::from_millis(1000));

            // Симуляция нажатия кнопки GPIO
            nav_down.navigate_down();
        }
    });

    // Кнопка 2 (GPIO): Выбор/подтверждение
    let nav_select = nav_manager.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));

        loop {
            thread::sleep(Duration::from_millis(3000));

            // Симуляция нажатия кнопки GPIO
            nav_select.select_item();
        }
    });
}

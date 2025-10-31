use crate::menu_process::MenuAppState;
use cursive::{
    Cursive,
    align::HAlign,
    view::{Nameable, Resizable},
    views::{Button, CircularFocus, Dialog, ListView, SelectView},
};

/// Отображение диалога выбора значения параметров устройства
pub(crate) fn show_config_view(siv: &mut Cursive) {
    // Получение текущего состояния приложения
    let app_state: MenuAppState = siv
        .take_user_data()
        .expect("Не удалось выполнить take_user_data");

    // Обновляем current_view
    app_state.navigation_manager.set_current_view("params_menu");

    let mut parameter_list = ListView::new();

    // Разделитель
    parameter_list.add_delimiter();

    // Создание списка параметров c выпадающими списками возможных значений
    for parameter in app_state.inner_config.parameters.iter() {
        let key = parameter.key.clone();
        let current_value = parameter.selected_value.clone();

        let mut select_view = SelectView::new().popup().h_align(HAlign::Left);

        let mut selected_index = 0;
        for (index, option) in parameter.options.iter().enumerate() {
            select_view.add_item(option, option.clone());
            if option == &current_value {
                selected_index = index;
            }
        }
        // Установка курсора на текущем значении параметра
        select_view.set_selection(selected_index);

        // Добавление коллбэка для обновления выбранного значения
        let key_for_callback = key.clone();

        select_view.set_on_submit(move |s, selected_value: &String| {
            let mut state: MenuAppState = s
                .take_user_data()
                .expect("Не удалось выполнить take_user_data");

            state
                .inner_config
                .update_parameter(&key_for_callback, selected_value.clone());
            s.set_user_data(state);
        });

        let parameter_view = select_view.with_name(key);

        parameter_list.add_child(&parameter.description, parameter_view);
    }

    // Разделитель
    parameter_list.add_delimiter();

    // Кнопка возврата в главное меню
    parameter_list.add_child(
        "",
        Button::new("Назад в главное меню", |s| {
            let state: MenuAppState = s
                .take_user_data()
                .expect("Не удалось выполнить take_user_data");
            state.navigation_manager.set_current_view("main_menu");
            s.set_user_data(state);
            s.pop_layer();
        }),
    );

    // Обёртка для циклической навигации с помощью одной кнопки
    let circular_list = CircularFocus::new(parameter_list)
        .with_wrap_arrows(true)
        .with_name("params_menu");

    let parameter_dialog = Dialog::around(circular_list)
        .title("⚙ Параметры устройства")
        .fixed_width(50);

    // Обновление текущих значений параметров
    siv.set_user_data(app_state);

    siv.add_layer(parameter_dialog);
}

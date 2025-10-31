use crate::views::config_view::show_config_view;
use crate::views::exit_view::show_exit_view;
use crate::views::update_view::show_update_view;
use cursive::{
    Cursive,
    view::{Nameable, Resizable},
    views::{CircularFocus, Dialog, LinearLayout, SelectView},
};

/// Отображение главного меню
pub(crate) fn show_main_view(siv: &mut Cursive) {
    siv.pop_layer();

    let main_menu = SelectView::new()
        .item("Параметры устройства", 1)
        .item("Обновление прошивки", 2)
        .item("Сохранение и выход", 3)
        .h_align(cursive::align::HAlign::Center)
        .on_submit(|s: &mut Cursive, menu_item_index| menu_item_selected(s, *menu_item_index));

    // Обёртка для циклической навигации с помощью одной кнопки
    let circular_menu = CircularFocus::new(main_menu).with_wrap_arrows(true);

    let layout = LinearLayout::vertical().child(
        Dialog::around(circular_menu.with_name("main_menu"))
            .title(format!(
                "{} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .fixed_size((50, 5)),
    );

    siv.add_layer(layout);
}

/// Обработка выбора определенного пункта в главном меню
fn menu_item_selected(siv: &mut Cursive, menu_item_index: i32) {
    match menu_item_index {
        1 => show_config_view(siv),
        2 => show_update_view(siv),
        3 => show_exit_view(siv),
        _ => {}
    }
}

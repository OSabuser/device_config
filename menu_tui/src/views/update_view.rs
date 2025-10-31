use cursive::{Cursive, views::Dialog};

/// Отображение диалога обновления прошивки
pub(crate) fn show_update_view(siv: &mut Cursive) {
    siv.add_layer(Dialog::info("Обновление прошивки пока не реализовано"));
}

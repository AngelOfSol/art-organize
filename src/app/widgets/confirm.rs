use imgui::{im_str, PopupModal, Ui};

pub fn confirm_delete_popup(ui: &Ui<'_>) -> bool {
    let mut ret = false;
    PopupModal::new(im_str!("Confirm Delete"))
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .always_auto_resize(true)
        .build(ui, || {
            ui.text(im_str!("Are you sure you want to delete this?"));

            if ui.button(im_str!("Yes, delete.")) {
                ret = true;
                ui.close_current_popup();
            }
            ui.same_line();

            if ui.button(im_str!("Cancel")) {
                ui.close_current_popup();
            }
        });

    ret
}

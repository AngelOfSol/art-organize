use imgui::{im_str, ImStr, Selectable, Ui};

pub fn view(ui: &Ui, label: &ImStr, raw_color: [f32; 4]) -> bool {
    let ret = Selectable::new(im_str!("?"))
        .size([ui.text_line_height_with_spacing(); 2])
        .build(ui);
    ui.same_line();
    ui.text_colored(raw_color, label);
    ret
}

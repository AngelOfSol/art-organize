use chrono::NaiveDate;
use imgui::{im_str, ImStr, Ui};

pub fn view(label: &str, value: &NaiveDate, ui: &Ui<'_>) {
    ui.text(im_str!("{}: {}", label, value.format("%-m/%-d/%-Y")));
}

pub fn edit(label: &ImStr, value: &NaiveDate, ui: &Ui<'_>) -> Option<NaiveDate> {
    let mut buf = im_str!("{}", value.format("%-m/%-d/%-Y"));

    if ui
        .input_text(label, &mut buf)
        .enter_returns_true(true)
        .build()
    {
        NaiveDate::parse_from_str(buf.to_str(), "%m/%d/%Y").ok()
    } else {
        None
    }
}

use chrono::NaiveDate;
use imgui::{im_str, ImStr, Ui};

pub const DATE_FORMAT: &str = "%-m/%-d/%-Y";

pub fn view(label: &str, value: &NaiveDate, ui: &Ui<'_>) {
    ui.text(im_str!("{}: {}", label, value.format(DATE_FORMAT)));
}

pub fn edit(label: &ImStr, value: &NaiveDate, ui: &Ui<'_>) -> Option<NaiveDate> {
    let mut buf = im_str!("{}", value.format(DATE_FORMAT));

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

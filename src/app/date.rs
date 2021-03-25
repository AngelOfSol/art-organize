use chrono::{Date, Datelike, Local, NaiveDate, TimeZone};
use imgui::{im_str, ImStr, Ui};

pub fn view(label: &str, value: &Date<Local>, ui: &Ui<'_>) {
    ui.text(im_str!("{}: {}", label, value.format("%-m/%-d/%-Y")));
}

pub fn edit(label: &ImStr, value: &Date<Local>, ui: &Ui<'_>) -> Option<Date<Local>> {
    let mut buf = im_str!("{}", value.format("%-m/%-d/%-Y"));

    if ui
        .input_text(label, &mut buf)
        .enter_returns_true(true)
        .build()
    {
        let local = NaiveDate::parse_from_str(buf.to_str(), "%m/%d/%Y").ok()?;
        if local.year() >= 2000 {
            Local
                .ymd_opt(local.year(), local.month(), local.day())
                .latest()
        } else {
            None
        }
    } else {
        None
    }
}

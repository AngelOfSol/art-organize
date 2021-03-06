use egui::{text_edit::CCursorRange, *};
use egui_demo_lib::easy_mark::MemoizedEasymarkHighlighter;

#[allow(clippy::ptr_arg)]
pub fn easy_mark_editor(ui: &mut egui::Ui, code: &mut String) {
    let response = {
        let mut layouter = |ui: &egui::Ui, easymark: &str, wrap_width: f32| {
            let mut layout_job =
                MemoizedEasymarkHighlighter::default().highlight(ui.visuals(), easymark);
            layout_job.wrap_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };

        ui.add(egui::TextEdit::multiline(code).layouter(&mut layouter))
    };

    if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
        if let Some(mut ccursor_range) = state.ccursor_range() {
            let any_change = shortcuts(ui, code, &mut ccursor_range);
            if any_change {
                state.set_ccursor_range(Some(ccursor_range));
                state.store(ui.ctx(), response.id);
            }
        }
    }
}

fn shortcuts(ui: &Ui, code: &mut dyn TextBuffer, ccursor_range: &mut CCursorRange) -> bool {
    let mut any_change = false;
    for event in &ui.input().events {
        if let Event::Key {
            key,
            pressed: true,
            modifiers,
        } = event
        {
            if modifiers.command_only() {
                match &key {
                    // toggle *bold*
                    Key::B => {
                        toggle_surrounding(code, ccursor_range, "*");
                        any_change = true;
                    }
                    // toggle `code`
                    Key::C => {
                        toggle_surrounding(code, ccursor_range, "`");
                        any_change = true;
                    }
                    // toggle /italics/
                    Key::I => {
                        toggle_surrounding(code, ccursor_range, "/");
                        any_change = true;
                    }
                    // toggle $lowered$
                    Key::L => {
                        toggle_surrounding(code, ccursor_range, "$");
                        any_change = true;
                    }
                    // toggle ^raised^
                    Key::R => {
                        toggle_surrounding(code, ccursor_range, "^");
                        any_change = true;
                    }
                    // toggle ~strikethrough~
                    Key::S => {
                        toggle_surrounding(code, ccursor_range, "~");
                        any_change = true;
                    }
                    // toggle _underline_
                    Key::U => {
                        toggle_surrounding(code, ccursor_range, "_");
                        any_change = true;
                    }
                    _ => {}
                }
            }
        }
    }
    any_change
}

/// E.g. toggle *strong* with `toggle(&mut text, &mut cursor, "*")`
fn toggle_surrounding(
    code: &mut dyn TextBuffer,
    ccursor_range: &mut CCursorRange,
    surrounding: &str,
) {
    let [primary, secondary] = ccursor_range.sorted();

    let surrounding_ccount = surrounding.chars().count();

    let prefix_crange = primary.index.saturating_sub(surrounding_ccount)..primary.index;
    let suffix_crange = secondary.index..secondary.index.saturating_add(surrounding_ccount);
    let already_surrounded = code.char_range(prefix_crange.clone()) == surrounding
        && code.char_range(suffix_crange.clone()) == surrounding;

    if already_surrounded {
        code.delete_char_range(suffix_crange);
        code.delete_char_range(prefix_crange);
        ccursor_range.primary.index -= surrounding_ccount;
        ccursor_range.secondary.index -= surrounding_ccount;
    } else {
        code.insert_text(surrounding, secondary.index);
        let advance = code.insert_text(surrounding, primary.index);

        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }
}

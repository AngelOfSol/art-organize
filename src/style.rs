use imgui::Style;

pub fn modify_style(style: &mut Style) {
    style.tab_rounding = 0.0;
    style.grab_rounding = 0.0;
    style.child_rounding = 0.0;
    style.frame_rounding = 0.0;
    style.popup_rounding = 0.0;
    style.window_rounding = 0.0;
    style.scrollbar_rounding = 0.0;

    style[imgui::StyleColor::Text] = [0.75, 0.75, 0.75, 1.00];
    style[imgui::StyleColor::TextDisabled] = [0.35, 0.35, 0.35, 1.00];
    style[imgui::StyleColor::WindowBg] = [0.00, 0.00, 0.00, 0.94];
    style[imgui::StyleColor::ChildBg] = [0.00, 0.00, 0.00, 0.00];
    style[imgui::StyleColor::PopupBg] = [0.08, 0.08, 0.08, 0.94];
    style[imgui::StyleColor::Border] = [0.00, 0.00, 0.00, 0.50];
    style[imgui::StyleColor::BorderShadow] = [0.00, 0.00, 0.00, 0.00];
    style[imgui::StyleColor::FrameBg] = [0.00, 0.00, 0.00, 0.54];
    style[imgui::StyleColor::FrameBgHovered] = [0.37, 0.14, 0.14, 0.67];
    style[imgui::StyleColor::FrameBgActive] = [0.39, 0.20, 0.20, 0.67];
    style[imgui::StyleColor::TitleBg] = [0.04, 0.04, 0.04, 1.00];
    style[imgui::StyleColor::TitleBgActive] = [0.48, 0.16, 0.16, 1.00];
    style[imgui::StyleColor::TitleBgCollapsed] = [0.48, 0.16, 0.16, 1.00];
    style[imgui::StyleColor::MenuBarBg] = [0.14, 0.14, 0.14, 1.00];
    style[imgui::StyleColor::ScrollbarBg] = [0.02, 0.02, 0.02, 0.53];
    style[imgui::StyleColor::ScrollbarGrab] = [0.31, 0.31, 0.31, 1.00];
    style[imgui::StyleColor::ScrollbarGrabHovered] = [0.41, 0.41, 0.41, 1.00];
    style[imgui::StyleColor::ScrollbarGrabActive] = [0.51, 0.51, 0.51, 1.00];
    style[imgui::StyleColor::CheckMark] = [0.56, 0.10, 0.10, 1.00];
    style[imgui::StyleColor::SliderGrab] = [1.00, 0.19, 0.19, 0.40];
    style[imgui::StyleColor::SliderGrabActive] = [0.89, 0.00, 0.19, 1.00];
    style[imgui::StyleColor::Button] = [1.00, 0.19, 0.19, 0.40];
    style[imgui::StyleColor::ButtonHovered] = [0.80, 0.17, 0.00, 1.00];
    style[imgui::StyleColor::ButtonActive] = [0.89, 0.00, 0.19, 1.00];
    style[imgui::StyleColor::Header] = [0.33, 0.35, 0.36, 0.53];
    style[imgui::StyleColor::HeaderHovered] = [0.76, 0.28, 0.44, 0.67];
    style[imgui::StyleColor::HeaderActive] = [0.47, 0.47, 0.47, 0.67];
    style[imgui::StyleColor::Separator] = [0.32, 0.32, 0.32, 1.00];
    style[imgui::StyleColor::SeparatorHovered] = [0.32, 0.32, 0.32, 1.00];
    style[imgui::StyleColor::SeparatorActive] = [0.32, 0.32, 0.32, 1.00];
    style[imgui::StyleColor::ResizeGrip] = [1.00, 1.00, 1.00, 0.85];
    style[imgui::StyleColor::ResizeGripHovered] = [1.00, 1.00, 1.00, 0.60];
    style[imgui::StyleColor::ResizeGripActive] = [1.00, 1.00, 1.00, 0.90];
    style[imgui::StyleColor::Tab] = [0.07, 0.07, 0.07, 0.51];
    style[imgui::StyleColor::TabHovered] = [0.86, 0.23, 0.43, 0.67];
    style[imgui::StyleColor::TabActive] = [0.19, 0.19, 0.19, 0.57];
    style[imgui::StyleColor::TabUnfocused] = [0.05, 0.05, 0.05, 0.90];
    style[imgui::StyleColor::TabUnfocusedActive] = [0.13, 0.13, 0.13, 0.74];
    style[imgui::StyleColor::PlotLines] = [0.61, 0.61, 0.61, 1.00];
    style[imgui::StyleColor::PlotLinesHovered] = [1.00, 0.43, 0.35, 1.00];
    style[imgui::StyleColor::PlotHistogram] = [0.90, 0.70, 0.00, 1.00];
    style[imgui::StyleColor::PlotHistogramHovered] = [1.00, 0.60, 0.00, 1.00];
    style[imgui::StyleColor::TableHeaderBg] = [0.19, 0.19, 0.20, 1.00];
    style[imgui::StyleColor::TableBorderStrong] = [0.31, 0.31, 0.35, 1.00];
    style[imgui::StyleColor::TableBorderLight] = [0.23, 0.23, 0.25, 1.00];
    style[imgui::StyleColor::TableRowBg] = [0.00, 0.00, 0.00, 0.00];
    style[imgui::StyleColor::TableRowBgAlt] = [1.00, 1.00, 1.00, 0.07];
    style[imgui::StyleColor::TextSelectedBg] = [0.26, 0.59, 0.98, 0.35];
    style[imgui::StyleColor::DragDropTarget] = [1.00, 1.00, 0.00, 0.90];
    style[imgui::StyleColor::NavHighlight] = [0.26, 0.59, 0.98, 1.00];
    style[imgui::StyleColor::NavWindowingHighlight] = [1.00, 1.00, 1.00, 0.70];
    style[imgui::StyleColor::NavWindowingDimBg] = [0.80, 0.80, 0.80, 0.20];
    style[imgui::StyleColor::ModalWindowDimBg] = [0.80, 0.80, 0.80, 0.35];
}
use imgui::{Style, StyleColor::*};

pub fn modify_style(style: &mut Style) {
    style.tab_rounding = 0.0;
    style.grab_rounding = 0.0;
    style.child_rounding = 0.0;
    style.frame_rounding = 0.0;
    style.popup_rounding = 0.0;
    style.window_rounding = 0.0;
    style.scrollbar_rounding = 0.0;

    for color in style.colors.iter_mut() {
        let alpha = color[3];
        color[3] = 1.0;

        for rgb in color.iter_mut().take(3) {
            *rgb *= alpha;
        }
    }
    // unactive windows shoudl still havethe active style
    style[TitleBg] = style[TitleBgActive];

    // style[Text] = [0.75, 0.75, 0.75, 1.00];
    // style[TextDisabled] = [0.35, 0.35, 0.35, 1.00];
    // style[WindowBg] = [0.00, 0.00, 0.00, 1.0];
    // style[ChildBg] = [0.00, 0.00, 0.00, 0.00];
    // style[PopupBg] = [0.08, 0.08, 0.08, 0.94];
    // style[Border] = [0.00, 0.00, 0.00, 0.50];
    // style[BorderShadow] = [0.00, 0.00, 0.00, 0.00];
    // style[FrameBg] = [0.00, 0.00, 0.00, 0.54];
    // style[FrameBgHovered] = [0.37, 0.14, 0.14, 0.67];
    // style[FrameBgActive] = [0.39, 0.20, 0.20, 0.67];
    // style[TitleBgActive] = [0.48, 0.16, 0.16, 1.00];
    // style[TitleBg] = [0.04, 0.04, 0.04, 1.00];
    // style[TitleBgCollapsed] = [0.48, 0.16, 0.16, 1.00];
    // style[MenuBarBg] = [0.14, 0.14, 0.14, 1.00];
    // style[ScrollbarBg] = [0.02, 0.02, 0.02, 0.53];
    // style[ScrollbarGrab] = [0.31, 0.31, 0.31, 1.00];
    // style[ScrollbarGrabHovered] = [0.41, 0.41, 0.41, 1.00];
    // style[ScrollbarGrabActive] = [0.51, 0.51, 0.51, 1.00];
    // style[CheckMark] = [0.56, 0.10, 0.10, 1.00];
    // style[SliderGrab] = [1.00, 0.19, 0.19, 0.40];
    // style[SliderGrabActive] = [0.89, 0.00, 0.19, 1.00];
    // style[Button] = [1.00, 0.19, 0.19, 0.40];
    // style[ButtonHovered] = [0.80, 0.17, 0.00, 1.00];
    // style[ButtonActive] = [0.89, 0.00, 0.19, 1.00];
    // style[Header] = [0.33, 0.35, 0.36, 0.53];
    // style[HeaderHovered] = [0.76, 0.28, 0.44, 0.67];
    // style[HeaderActive] = [0.47, 0.47, 0.47, 0.67];
    // style[Separator] = [0.32, 0.32, 0.32, 1.00];
    // style[SeparatorHovered] = [0.32, 0.32, 0.32, 1.00];
    // style[SeparatorActive] = [0.32, 0.32, 0.32, 1.00];
    // style[ResizeGrip] = [1.00, 1.00, 1.00, 0.85];
    // style[ResizeGripHovered] = [1.00, 1.00, 1.00, 0.60];
    // style[ResizeGripActive] = [1.00, 1.00, 1.00, 0.90];
    // style[Tab] = [0.07, 0.07, 0.07, 0.51];
    // style[TabHovered] = [0.86, 0.23, 0.43, 0.67];
    // style[TabActive] = [0.19, 0.19, 0.19, 0.57];
    // style[TabUnfocused] = [0.05, 0.05, 0.05, 0.90];
    // style[TabUnfocusedActive] = [0.13, 0.13, 0.13, 0.74];
    // style[PlotLines] = [0.61, 0.61, 0.61, 1.00];
    // style[PlotLinesHovered] = [1.00, 0.43, 0.35, 1.00];
    // style[PlotHistogram] = [0.90, 0.70, 0.00, 1.00];
    // style[PlotHistogramHovered] = [1.00, 0.60, 0.00, 1.00];
    // style[TableHeaderBg] = [0.19, 0.19, 0.20, 1.00];
    // style[TableBorderStrong] = [0.31, 0.31, 0.35, 1.00];
    // style[TableBorderLight] = [0.23, 0.23, 0.25, 1.00];
    // style[TableRowBg] = [0.00, 0.00, 0.00, 0.00];
    // style[TableRowBgAlt] = [1.00, 1.00, 1.00, 0.07];
    // style[TextSelectedBg] = [0.26, 0.59, 0.98, 0.35];
    // style[DragDropTarget] = [1.00, 1.00, 0.00, 0.90];
    // style[NavHighlight] = [0.26, 0.59, 0.98, 1.00];
    // style[NavWindowingHighlight] = [1.00, 1.00, 1.00, 0.70];
    // style[NavWindowingDimBg] = [0.80, 0.80, 0.80, 0.20];
    // style[ModalWindowDimBg] = [0.80, 0.80, 0.80, 0.35];
}

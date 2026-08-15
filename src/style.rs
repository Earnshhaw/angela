use iced::theme::Palette;
use iced::widget::{button, container, pane_grid, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Theme};
use iced_aw::context_menu;

pub fn rose_pine() -> Theme {
    Theme::custom(
        "Rosé Pine".to_string(),
        Palette {
            background: Color::from_rgb8(0x19, 0x17, 0x24), // base
            text: Color::from_rgb8(0xe0, 0xde, 0xf4),       // text
            primary: Color::from_rgb8(0xc4, 0xa7, 0xe7),    // iris
            success: Color::from_rgb8(0x9c, 0xcf, 0xd8),    // foam
            warning: Color::from_rgb8(0xf6, 0xc1, 0x77),    // gold
            danger: Color::from_rgb8(0xeb, 0x6f, 0x92),     // love
        },
    )
}

pub fn win11_dark() -> Theme {
    Theme::custom(
        "Win11 Dark".to_string(),
        Palette {
            background: Color::from_rgb8(0x20, 0x20, 0x20),
            text: Color::from_rgb8(0xff, 0xff, 0xff),
            primary: Color::from_rgb8(0x60, 0xcd, 0xff),
            success: Color::from_rgb8(0x6c, 0xcb, 0x5f),
            warning: Color::from_rgb8(0xff, 0xb9, 0x00),
            danger: Color::from_rgb8(0xff, 0x99, 0xa4),
        },
    )
}

pub fn win11_light() -> Theme {
    Theme::custom(
        "Win11 Light".to_string(),
        Palette {
            background: Color::from_rgb8(0xf3, 0xf3, 0xf3),
            text: Color::from_rgb8(0x1a, 0x1a, 0x1a),
            primary: Color::from_rgb8(0x00, 0x5f, 0xb8), // Win11 accent blue
            success: Color::from_rgb8(0x0f, 0x7b, 0x0f),
            warning: Color::from_rgb8(0x9d, 0x5d, 0x00),
            danger: Color::from_rgb8(0xc4, 0x2b, 0x1c),
        },
    )
}

pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        background: Some(Background::Color(palette.primary.base.color)),
        text_color: palette.primary.base.text,
        border: Border {
            radius: 6.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.primary.weak.color)),
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: Color {
                a: 0.4,
                ..base.text_color
            },
            background: Some(Background::Color(Color {
                a: 0.2,
                ..palette.primary.base.color
            })),
            ..base
        },
    }
}

pub fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        text_color: palette.background.base.text,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.background.strong.color)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.primary.weak.color)),
            border: Border {
                color: palette.primary.base.color,
                ..base.border
            },
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: Color {
                a: 0.4,
                ..base.text_color
            },
            ..base
        },
    }
}

pub fn row_style(is_hovered: bool, is_pressed: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme: &Theme| {
        let palette = theme.extended_palette();
        let background = if is_pressed {
            Some(Background::Color(palette.primary.weak.color))
        } else if is_hovered {
            Some(Background::Color(palette.background.weak.color))
        } else {
            None
        };
        container::Style {
            text_color: Some(palette.background.base.text),
            background,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

pub fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: palette.danger.base.color,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: true,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.danger.base.color)),
            text_color: palette.danger.base.text,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.danger.weak.color)),
            text_color: palette.danger.base.text,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: Color {
                a: 0.4,
                ..base.text_color
            },
            ..base
        },
    }
}

pub fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();

    let base = text_input::Style {
        background: Background::Color(palette.background.weak.color),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        icon: palette.background.base.text,
        placeholder: Color {
            a: 0.5,
            ..palette.background.base.text
        },
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: palette.primary.weak.color,
                ..base.border
            },
            ..base
        },
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: palette.primary.base.color,
                ..base.border
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(palette.background.weak.color),
            value: Color {
                a: 0.4,
                ..base.value
            },
            ..base
        },
    }
}

pub fn pane_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(Background::Color(palette.background.base.color)),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn sidebar_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn menu_container(
    theme: &Theme,
    _status: context_menu::Status,
) -> iced_aw::context_menu::Style {
    let palette = theme.extended_palette();

    context_menu::Style {
        background: Background::Color(palette.background.strong.color),
    }
}

pub fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let palette = theme.extended_palette();

    let rail = scrollable::Rail {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(palette.background.strong.color),
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        },
    };

    let hovered_rail = scrollable::Rail {
        scroller: scrollable::Scroller {
            background: Background::Color(palette.primary.base.color),
            ..rail.scroller
        },
        ..rail
    };

    let auto_scroll = scrollable::AutoScroll {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: u32::MAX.into(),
            width: 1.0,
            color: palette.background.base.text,
        },
        shadow: Shadow {
            color: Color {
                a: 0.7,
                ..Color::BLACK
            },
            offset: iced::Vector::ZERO,
            blur_radius: 2.0,
        },
        icon: palette.background.base.text,
    };

    match status {
        scrollable::Status::Active { .. } => scrollable::Style {
            container: pane_container(theme),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll,
        },
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered,
            ..
        } => scrollable::Style {
            container: pane_container(theme),
            vertical_rail: if is_vertical_scrollbar_hovered {
                hovered_rail
            } else {
                rail
            },
            horizontal_rail: rail,
            gap: None,
            auto_scroll,
        },
        scrollable::Status::Dragged { .. } => scrollable::Style {
            container: pane_container(theme),
            vertical_rail: hovered_rail,
            horizontal_rail: hovered_rail,
            gap: None,
            auto_scroll,
        },
    }
}

pub fn pane_grid_style(theme: &Theme) -> pane_grid::Style {
    let palette = theme.extended_palette();

    pane_grid::Style {
        hovered_region: pane_grid::Highlight {
            background: Background::Color(Color {
                a: 0.15,
                ..palette.primary.base.color
            }),
            border: Border {
                radius: 6.0.into(),
                width: 2.0,
                color: palette.primary.base.color,
            },
        },
        picked_split: pane_grid::Line {
            color: palette.primary.base.color,
            width: 2.0,
        },
        hovered_split: pane_grid::Line {
            color: palette.primary.weak.color,
            width: 2.0,
        },
    }
}

pub fn sort_button_active(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style {
        background: Some(palette.primary.strong.color.into()),
        text_color: palette.primary.strong.text,
        border: iced::Border {
            radius: 0.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.primary.base.color.into()),
            ..base
        },
        _ => base,
    }
}

pub fn sort_button_inactive(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style {
        background: Some(palette.background.weak.color.into()),
        text_color: palette.background.weak.text,
        border: iced::Border {
            radius: 0.0.into(),
            ..Default::default()
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.background.strong.color.into()),
            ..base
        },
        _ => base,
    }
}

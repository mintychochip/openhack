use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::app::{Tab, ThemeApp};

/// Draw the complete theme editor UI.
pub fn draw(f: &mut Frame<'_>, app: &mut ThemeApp) {
    app.check_status_expiry();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tab bar
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Help bar
        ])
        .margin(1)
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_tab_bar(f, chunks[1], app);
    draw_content(f, chunks[2], app);
    draw_help_bar(f, chunks[3], app);
}

fn draw_header(f: &mut Frame<'_>, area: Rect, app: &ThemeApp) {
    let api_status = if app.api_connected {
        Span::styled("API: ✓ connected", Style::default().fg(Color::Green))
    } else {
        Span::styled("API: ✗ offline", Style::default().fg(Color::Red))
    };

    let header_text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                " OpenHack Theme Editor ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            api_status,
        ]),
    ]);

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Left);
    f.render_widget(header, area);
}

fn draw_tab_bar(f: &mut Frame<'_>, area: Rect, app: &ThemeApp) {
    let tabs: Vec<Span> = Tab::all()
        .iter()
        .enumerate()
        .flat_map(|(i, tab)| {
            let is_active = *tab == app.active_tab;
            let style = if is_active {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let sep = if i > 0 {
                vec![Span::raw(" | ")]
            } else {
                vec![]
            };
            [
                sep,
                vec![Span::styled(format!(" {} ", tab.label()), style)],
            ]
            .concat()
        })
        .collect();

    let tab_bar = Paragraph::new(Line::from(tabs))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);
    f.render_widget(tab_bar, area);
}

fn draw_content(f: &mut Frame<'_>, area: Rect, app: &mut ThemeApp) {
    match app.active_tab {
        Tab::Branding => draw_branding(f, area, app),
        Tab::Colors => draw_colors(f, area, app),
        Tab::Typography => draw_typography(f, area, app),
        Tab::Preview => draw_preview(f, area, app),
        Tab::CustomCss => draw_custom_css(f, area, app),
    }

    // Draw status popup if present
    if let Some(ref status) = app.status_message {
        let popup_area = centered_rect(60, 20, f.area());
        let clear = Block::default().style(Style::default().bg(Color::Black));
        f.render_widget(clear, popup_area);

        let status_text = Paragraph::new(status.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Status "),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(status_text, popup_area);
    }
}

fn draw_branding(f: &mut Frame<'_>, area: Rect, app: &ThemeApp) {
    let fields = [("Name", &app.config.name), ("Tagline", &app.config.tagline), ("Logo URL", &app.config.logo_url)];
    let mut lines = vec![];
    for (i, (label, value)) in fields.iter().enumerate() {
        let is_selected = i == app.selected_field;
        let is_editing = is_selected && app.editing;
        let display_val = if is_editing {
            app.input_buffer.as_str()
        } else {
            value.as_str()
        };
        let prefix = if is_selected { "▶ " } else { "  " };
        let label_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let value_style = if is_editing {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(vec![
            Span::raw(prefix),
            Span::styled(format!("{label}: "), label_style),
            Span::styled(display_val.to_string(), value_style),
        ]));
        lines.push(Line::from(""));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Branding ");
    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, area);
}

fn draw_colors(f: &mut Frame<'_>, area: Rect, app: &ThemeApp) {
    let mut lines = vec![];

    // Preset selector
    let preset_selected = 0 == app.selected_field;
    let preset_editing = preset_selected && app.editing;
    let preset_val = if preset_editing {
        app.input_buffer.clone()
    } else {
        app.config.daisyui_theme_preset.clone()
    };
    lines.push(Line::from(vec![
        Span::raw(if preset_selected { "▶ " } else { "  " }),
        Span::styled(
            "Preset: ",
            if preset_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        Span::styled(preset_val, if preset_editing { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) }),
    ]));
    lines.push(Line::from(""));

    // Show all presets as a wrapped list for reference
    let presets_str = super::presets::DAISYUI_PRESETS.join(", ");
    lines.push(Line::from(vec![
        Span::styled("Available: ", Style::default().fg(Color::DarkGray)),
        Span::styled(presets_str, Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    if app.config.daisyui_theme_preset == "custom" {
        let color_fields = [("Primary", &app.config.primary_color)];
        for (i, (label, value)) in color_fields.iter().enumerate() {
            let idx = i + 1;
            let is_selected = idx == app.selected_field;
            let is_editing = is_selected && app.editing;
            let display_val = if is_editing {
                app.input_buffer.as_str()
            } else {
                value.as_str()
            };
            lines.push(Line::from(vec![
                Span::raw(if is_selected { "▶ " } else { "  " }),
                Span::styled(
                    format!("{label}: "),
                    if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(display_val.to_string(), if is_editing { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) }),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("Using preset colors — select 'custom' to edit individual colors.", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let block = Block::default().borders(Borders::ALL).title(" Colors ");
    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, area);
}

fn draw_typography(f: &mut Frame<'_>, area: Rect, app: &ThemeApp) {
    let mut lines = vec![];

    // Font preset selector
    let preset_name = app.font_preset_name();
    let preset_selected = 0 == app.selected_field;
    let preset_editing = preset_selected && app.editing;
    let preset_val = if preset_editing {
        app.input_buffer.clone()
    } else {
        preset_name.to_string()
    };
    lines.push(Line::from(vec![
        Span::raw(if preset_selected { "▶ " } else { "  " }),
        Span::styled(
            "Font Preset: ",
            if preset_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        Span::styled(preset_val, if preset_editing { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) }),
    ]));
    lines.push(Line::from(""));

    // Show available presets
    let names: Vec<&str> = super::presets::font_presets().iter().map(|p| p.name).collect();
    lines.push(Line::from(vec![
        Span::styled("Available: ", Style::default().fg(Color::DarkGray)),
        Span::styled(names.join(", "), Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    if preset_name == "Custom" {
        let custom_fields = [
            ("Display URL", &app.config.font_config.display.url),
            ("Display Family", &app.config.font_config.display.family),
            ("Heading URL", &app.config.font_config.heading.url),
            ("Heading Family", &app.config.font_config.heading.family),
            ("Body URL", &app.config.font_config.body.url),
            ("Body Family", &app.config.font_config.body.family),
        ];
        for (i, (label, value)) in custom_fields.iter().enumerate() {
            let idx = i + 1;
            let is_selected = idx == app.selected_field;
            let is_editing = is_selected && app.editing;
            let display_val = if is_editing {
                app.input_buffer.as_str()
            } else {
                value.as_str()
            };
            lines.push(Line::from(vec![
                Span::raw(if is_selected { "▶ " } else { "  " }),
                Span::styled(
                    format!("{label}: "),
                    if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(display_val.to_string(), if is_editing { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::White) }),
            ]));
        }
    } else {
        let fc = &app.config.font_config;
        lines.push(Line::from(vec![
            Span::styled("Display: ", Style::default().fg(Color::Gray)),
            Span::styled(&fc.display.family, Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Heading: ", Style::default().fg(Color::Gray)),
            Span::styled(&fc.heading.family, Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Body:    ", Style::default().fg(Color::Gray)),
            Span::styled(&fc.body.family, Style::default().fg(Color::White)),
        ]));
    }

    let block = Block::default().borders(Borders::ALL).title(" Typography ");
    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, area);
}

fn draw_preview(f: &mut Frame<'_>, area: Rect, app: &ThemeApp) {
    let fc = &app.config.font_config;
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                &app.config.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(&app.config.tagline, Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("━━━ Font Preview ━━━", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Display:  ", Style::default().fg(Color::Magenta)),
            Span::styled(&fc.display.family, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Heading:  ", Style::default().fg(Color::Magenta)),
            Span::styled(&fc.heading.family, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("Body:     ", Style::default().fg(Color::Magenta)),
            Span::styled(&fc.body.family, Style::default()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("━━━ Theme ━━━", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Preset: ", Style::default().fg(Color::Magenta)),
            Span::styled(&app.config.daisyui_theme_preset, Style::default()),
        ]),
    ];

    let block = Block::default().borders(Borders::ALL).title(" Preview ");
    let para = Paragraph::new(Text::from(lines)).block(block).alignment(Alignment::Center);
    f.render_widget(para, area);
}

fn draw_custom_css(f: &mut Frame<'_>, area: Rect, app: &ThemeApp) {
    let is_editing = app.selected_field == 0 && app.editing;
    let content = if is_editing {
        app.input_buffer.clone()
    } else {
        app.config.custom_css.clone()
    };

    let lines: Vec<Line> = content
        .lines()
        .map(|l| {
            Line::from(vec![Span::styled(
                l.to_string(),
                if is_editing {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                },
            )])
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Custom CSS ")
        .border_style(if is_editing {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        });

    let para = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_help_bar(f: &mut Frame<'_>, area: Rect, _app: &ThemeApp) {
    let help = Paragraph::new(
        "Tab←→ tabs  ↑↓ navigate  Enter edit  Esc cancel  S save  R reset  Q quit"
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, area);
}

/// Create a centered rectangle with given percentage width/height.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, DeployTarget, Screen, ServiceName};

const SERVICE_INFO: [(ServiceName, &str, u32); 11] = [
    (ServiceName::Auth, "Login, OAuth, JWT, MFA", 64),
    (ServiceName::Core, "Teams, projects, events, phases", 64),
    (ServiceName::Gateway, "API reverse proxy + SSE", 48),
    (ServiceName::Judging, "Rubrics, scores, assignments", 96),
    (ServiceName::Leaderboard, "Rankings, voting, formulas", 48),
    (ServiceName::Mail, "Email sending", 64),
    (ServiceName::Notify, "Discord/Slack/webhooks", 64),
    (ServiceName::Ai, "Chat, RAG, embeddings", 64),
    (ServiceName::Analytics, "Charts, reports, export", 64),
    (ServiceName::Sponsors, "Booths, prizes, submissions", 64),
    (ServiceName::Media, "File uploads, S3/local", 96),
];

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_title(f, chunks[0], app);
    draw_content(f, chunks[1], app);
    draw_help(f, chunks[2], app);
}

fn draw_title(f: &mut Frame, area: Rect, app: &App) {
    let idx = app.screen_index();
    let total = App::total_screen_count();
    let title = if app.screen == Screen::Migrate {
        Paragraph::new(Line::from(vec![Span::styled(
            " OpenHack Migrate ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]))
        .block(Block::default().borders(Borders::BOTTOM))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(
                " OpenHack Installer ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Step {idx}/{total}"),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .block(Block::default().borders(Borders::BOTTOM))
    };
    f.render_widget(title, area);
}

fn draw_help(f: &mut Frame, area: Rect, app: &App) {
    let hint = match app.screen {
        Screen::Welcome => "Enter to start",
        Screen::DeployTarget | Screen::Database | Screen::Storage | Screen::Email => {
            "\u{2191}\u{2193} navigate  Enter confirm  Esc back"
        }
        Screen::Services => "\u{2191}\u{2193} navigate  Space toggle  Enter confirm  Esc back",
        Screen::Secrets => "Tab next field  R regenerate JWT  Enter confirm  Esc back",
        Screen::CostEstimate => "Enter continue  Esc back",
        Screen::Review => "Enter deploy  S save config only  Esc back",
        Screen::Progress => "Deploying...",
        Screen::Done => "M migrate  q quit",
        Screen::Migrate => {
            "\u{2191}\u{2193} navigate  Space toggle target  Enter migrate  Esc back"
        }
    };
    let help = Paragraph::new(Span::styled(hint, Style::default().fg(Color::DarkGray)));
    f.render_widget(help, area);
}

fn draw_content(f: &mut Frame, area: Rect, app: &App) {
    match app.screen {
        Screen::Welcome => draw_welcome(f, area, app),
        Screen::DeployTarget => draw_deploy_target(f, area, app),
        Screen::Services => draw_services(f, area, app),
        Screen::Database => draw_database(f, area, app),
        Screen::Storage => draw_storage(f, area, app),
        Screen::Email => draw_email(f, area, app),
        Screen::Secrets => draw_secrets(f, area, app),
        Screen::CostEstimate => draw_cost_estimate(f, area, app),
        Screen::Review => draw_review(f, area, app),
        Screen::Progress => draw_progress(f, area, app),
        Screen::Done => draw_done(f, area, app),
        Screen::Migrate => draw_migrate(f, area, app),
    }
}

fn draw_welcome(f: &mut Frame, area: Rect, app: &App) {
    let _ = app;
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  OpenHack Installer",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Self-hosted hackathon management platform"),
        Line::from("  11 Rust microservices \u{00b7} <1GB RAM \u{00b7} 100% Rust"),
        Line::from(""),
        Line::from(Span::styled(
            "  [Press Enter to start]",
            Style::default().fg(Color::Yellow),
        )),
    ];
    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

fn draw_deploy_target(f: &mut Frame, area: Rect, app: &App) {
    let options = [
        (
            "Self-hosted (Docker Compose)",
            "Free, runs on any VPS or localhost",
        ),
        ("Serverless (AWS Lambda)", "Pay-per-request, ~$15/mo dev"),
        ("Serverless (GCP Cloud Run)", "Pay-per-request, ~$15/mo dev"),
        (
            "Serverless (Azure Container Apps)",
            "Pay-per-request, ~$15/mo dev",
        ),
        ("Kubernetes (Helm)", "For existing K8s clusters"),
    ];
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let marker = if app.selected == i {
                "\u{25c9}"
            } else {
                "\u{25cb}"
            };
            let style = if app.selected == i {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{marker} "), style),
                Span::styled(*name, style),
                Span::styled(format!("  - {desc}"), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();
    let list = List::new(items).block(Block::default().title(" How do you want to run OpenHack? "));
    f.render_widget(list, area);
}

fn draw_services(f: &mut Frame, area: Rect, app: &App) {
    let _all = ServiceName::all();
    let items: Vec<ListItem> = SERVICE_INFO
        .iter()
        .enumerate()
        .map(|(i, (svc, desc, mem))| {
            let enabled = app.is_service_enabled(*svc);
            let is_core = svc.is_core_service();
            let marker = if enabled { "\u{25c9}" } else { "\u{25cb}" };
            let style = if app.selected == i {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let core_tag = if is_core { " [core]" } else { "" };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{marker} "), style),
                Span::styled(format!("{:<12}", format!("{svc:?}")), style),
                Span::styled(
                    format!(" - {desc}{core_tag}"),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!(" {mem}M"), Style::default().fg(Color::Cyan)),
            ]))
        })
        .collect();
    let total = app.total_memory_mb();
    let list = List::new(items).block(
        Block::default().title(format!(" Which services? (Space toggle)  Total: {total}M ")),
    );
    f.render_widget(list, area);
}

fn draw_database(f: &mut Frame, area: Rect, app: &App) {
    let lines = vec![
        Line::from(format!("  Database URL: {}", app.database_url())),
        Line::from(format!("  Password: {}", "*".repeat(app.db_password.len()))),
        Line::from(format!("  Environment: {}", app.environment)),
        Line::from(""),
        Line::from(Span::styled(
            "  Enter to continue",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let para = Paragraph::new(lines).block(Block::default().title(" Database Configuration "));
    f.render_widget(para, area);
}

fn draw_storage(f: &mut Frame, area: Rect, app: &App) {
    let providers = [
        "Local filesystem",
        "S3-compatible (MinIO)",
        "AWS S3",
        "GCP Cloud Storage",
        "Azure Blob",
    ];
    let items: Vec<ListItem> = providers
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let marker = if app.selected == i {
                "\u{25c9}"
            } else {
                "\u{25cb}"
            };
            let _style = if app.selected == i {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(format!("{marker} {name}")))
        })
        .collect();
    let list = List::new(items).block(Block::default().title(" File Storage "));
    f.render_widget(list, area);
}

fn draw_email(f: &mut Frame, area: Rect, app: &App) {
    let providers = ["AWS SES", "SendGrid", "SMTP relay (Postfix)", "Disabled"];
    let items: Vec<ListItem> = providers
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let marker = if app.selected == i {
                "\u{25c9}"
            } else {
                "\u{25cb}"
            };
            let _style = if app.selected == i {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(format!("{marker} {name}")))
        })
        .collect();
    let list = List::new(items).block(Block::default().title(" Email Delivery "));
    f.render_widget(list, area);
}

fn draw_secrets(f: &mut Frame, area: Rect, app: &App) {
    let lines = vec![
        Line::from(format!(
            "  JWT Secret:     {}  [R] regenerate",
            app.jwt_secret
        )),
        Line::from(format!("  OpenAI Key:     {}", app.openai_api_key)),
        Line::from(format!("  GitHub ID:      {}", app.github_client_id)),
        Line::from(format!("  GitHub Secret:  {}", app.github_client_secret)),
        Line::from(format!("  Discord Webhook:{}", app.webhook_secret)),
        Line::from(format!("  Redis URL:      {}", app.redis_url)),
        Line::from(""),
        Line::from(Span::styled(
            "  Tab to cycle  R to regenerate JWT  Enter to continue",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let para = Paragraph::new(lines).block(Block::default().title(" API Keys & Secrets "));
    f.render_widget(para, area);
}

fn draw_cost_estimate(f: &mut Frame, area: Rect, app: &App) {
    let (total, lines) = app.estimate_monthly_cost();
    let target = &app.deploy_target;

    let mut draw_lines: Vec<Line> = vec![
        Line::from(format!("  Target: {target}")),
        Line::from(format!(
            "  Services: {} enabled",
            app.enabled_services.len()
        )),
        Line::from(""),
    ];

    for (name, cost) in &lines {
        draw_lines.push(Line::from(vec![
            Span::styled(format!("  {name:<40}"), Style::default()),
            Span::styled(format!("${cost:.2}/mo"), Style::default().fg(Color::Cyan)),
        ]));
    }

    draw_lines.push(Line::from(""));
    draw_lines.push(Line::from(vec![
        Span::styled("  Estimated total: ", Style::default()),
        Span::styled(
            format!("${total:.2}/mo"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    if matches!(app.deploy_target, DeployTarget::AwsLambda) {
        draw_lines.push(Line::from(""));
        draw_lines.push(Line::from(Span::styled(
            "  Note: Lambda scales to zero when idle — dev costs are often < $1/mo",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let para = Paragraph::new(draw_lines).block(Block::default().title(" Estimated Monthly Cost "));
    f.render_widget(para, area);
}

fn draw_review(f: &mut Frame, area: Rect, app: &App) {
    let enabled: Vec<String> = app
        .enabled_services
        .iter()
        .map(|s| format!("{s:?}"))
        .collect();
    let total = app.total_memory_mb();
    let lines = vec![
        Line::from(format!("  Target:      {}", app.deploy_target)),
        Line::from(format!("  Environment: {}", app.environment)),
        Line::from(format!(
            "  Services:    {} ({})",
            enabled.join(", "),
            enabled.len()
        )),
        Line::from(format!("  Memory:      {total}M limits")),
        Line::from(format!("  Database:    {}", app.database_url())),
        Line::from(format!("  Storage:     {}", app.storage_provider)),
        Line::from(format!("  Email:       {}", app.mail_provider)),
        Line::from(format!(
            "  Redis:       {}",
            if app.uses_redis() {
                &app.redis_url
            } else {
                "disabled"
            }
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Enter = Deploy  |  S = Save config only",
            Style::default().fg(Color::Yellow),
        )),
    ];
    let para = Paragraph::new(lines).block(Block::default().title(" Review Configuration "));
    f.render_widget(para, area);
}

fn draw_progress(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .deploy_steps
        .iter()
        .map(|step| {
            let icon = match step.status {
                crate::app::DeployStatus::Pending => "  ",
                crate::app::DeployStatus::Running => "\u{2807}",
                crate::app::DeployStatus::Success => "\u{2713}",
                crate::app::DeployStatus::Failed => "\u{2717}",
            };
            let color = match step.status {
                crate::app::DeployStatus::Pending => Color::DarkGray,
                crate::app::DeployStatus::Running => Color::Yellow,
                crate::app::DeployStatus::Success => Color::Green,
                crate::app::DeployStatus::Failed => Color::Red,
            };
            ListItem::new(Line::from(Span::styled(
                format!(" {icon} {}", step.name),
                Style::default().fg(color),
            )))
        })
        .collect();
    let list = List::new(items).block(Block::default().title(" Deploying OpenHack... "));
    f.render_widget(list, area);
}

fn draw_done(f: &mut Frame, area: Rect, _app: &App) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  OpenHack is running!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  API:     http://localhost:8000"),
        Line::from("  Auth:    http://localhost:3001"),
        Line::from("  Core:    http://localhost:3002"),
        Line::from(""),
        Line::from("  Next: Create an admin user:"),
        Line::from("  curl -X POST http://localhost:3001/api/auth/register \\"),
        Line::from(r#"    -H "Content-Type: application/json" \"#),
        Line::from(
            r#"    -d '{"name":"admin","email":"admin@hack.dev","password":"changeme","roles":["admin"]}'"#,
        ),
        Line::from(""),
        Line::from(Span::styled(
            "  M = Migrate services   q = Quit",
            Style::default().fg(Color::Yellow),
        )),
    ];
    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

fn draw_migrate(f: &mut Frame, area: Rect, app: &App) {
    let Some(ref state) = app.deploy_state else {
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No deploy state found.",
                Style::default().fg(Color::Red),
            )),
            Line::from("  Run a deploy first, then return here."),
        ];
        let para = Paragraph::new(lines);
        f.render_widget(para, area);
        return;
    };

    let current = &state.deploy_target.current;
    let keys = state.service_keys_ordered();
    let migration_count = state.migration_count();

    let mut lines: Vec<Line> = vec![
        Line::from(format!("  Default target: {current}")),
        Line::from(""),
    ];

    for (i, key) in keys.iter().enumerate() {
        let svc_state = state.services.get(*key);
        let target: &str = svc_state.map_or(current, |s| s.target.as_str());
        let endpoint = svc_state.map_or("", |s| s.endpoint.as_str());
        let changed = target != current;

        let marker = if app.migrate_selected == i {
            "\u{25c9}"
        } else {
            "\u{25cb}"
        };
        let base_style = if app.migrate_selected == i {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let target_style = if changed {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{marker} "), base_style),
            Span::styled(format!("{key:<12}"), base_style),
            Span::styled(format!(" \u{2192} {target:<16}"), target_style),
            Span::styled(format!(" {endpoint}"), Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));
    if migration_count > 0 {
        lines.push(Line::from(Span::styled(
            format!("  {migration_count} service(s) changed \u{2014} Enter to migrate"),
            Style::default().fg(Color::Yellow),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  No changes \u{2014} Space to toggle a service target",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let para = Paragraph::new(lines)
        .block(Block::default().title(" Migrate Services (Space = toggle target) "));
    f.render_widget(para, area);
}

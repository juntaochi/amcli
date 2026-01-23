use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::config::Language;
use crate::ui::Theme;

#[derive(Debug, Clone)]
pub struct SettingsMenu {
    pub is_open: bool,
    pub selected_index: usize,
    items: Vec<SettingsItem>,
}

#[derive(Debug, Clone)]
pub enum SettingsItem {
    Language { current: Language },
    Theme { current_index: usize, total_themes: usize },
    Mosaic { enabled: bool },
    Close,
}

impl SettingsMenu {
    pub fn new(language: Language, theme_index: usize, total_themes: usize, mosaic: bool) -> Self {
        let items = vec![
            SettingsItem::Language { current: language },
            SettingsItem::Theme {
                current_index: theme_index,
                total_themes,
            },
            SettingsItem::Mosaic { enabled: mosaic },
            SettingsItem::Close,
        ];

        Self {
            is_open: false,
            selected_index: 0,
            items,
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            self.selected_index = 0;
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn update_language(&mut self, language: Language) {
        if let Some(item) = self.items.get_mut(0) {
            *item = SettingsItem::Language { current: language };
        }
    }

    pub fn update_theme(&mut self, theme_index: usize) {
        if let Some(SettingsItem::Theme { total_themes, .. }) = self.items.get(1) {
            let total = *total_themes;
            if let Some(item) = self.items.get_mut(1) {
                *item = SettingsItem::Theme {
                    current_index: theme_index,
                    total_themes: total,
                };
            }
        }
    }

    pub fn update_mosaic(&mut self, enabled: bool) {
        if let Some(item) = self.items.get_mut(2) {
            *item = SettingsItem::Mosaic { enabled };
        }
    }

    pub fn get_selected_item(&self) -> Option<&SettingsItem> {
        self.items.get(self.selected_index)
    }

    pub fn click_at(&mut self, row: u16, settings_area: Rect) -> Option<usize> {
        // Calculate which item was clicked based on row position
        // Settings area starts with border, title, then 4 items
        let items_start = settings_area.y + 2; // Account for border and title
        if row >= items_start && row < items_start + self.items.len() as u16 {
            let clicked_index = (row - items_start) as usize;
            if clicked_index < self.items.len() {
                self.selected_index = clicked_index;
                return Some(clicked_index);
            }
        }
        None
    }

    pub fn render(&self, f: &mut Frame, theme: Theme) {
        let area = f.area();

        // Create centered overlay
        let popup_width = 60.min(area.width - 4);
        let popup_height = 12.min(area.height - 4);

        let popup_area = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        // Clear the background
        f.render_widget(Clear, popup_area);

        // Create the settings block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(if theme.is_retro {
                BorderType::Thick
            } else {
                BorderType::Rounded
            })
            .border_style(Style::default().fg(theme.accent))
            .title(vec![
                Span::styled(" [ ", Style::default().fg(theme.dim)),
                Span::styled(
                    "SETTINGS",
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ] ", Style::default().fg(theme.dim)),
            ])
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(theme.bg));

        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);

        // Render menu items
        let mut list_items = Vec::new();

        for (i, item) in self.items.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let (label, value) = match item {
                SettingsItem::Language { current } => {
                    let lang_str = match current {
                        Language::English => "English",
                        Language::Japanese => "日本語",
                    };
                    ("Language / 言語", lang_str.to_string())
                }
                SettingsItem::Theme {
                    current_index,
                    total_themes,
                } => (
                    "Theme / テーマ",
                    format!("{} / {}", current_index + 1, total_themes),
                ),
                SettingsItem::Mosaic { enabled } => {
                    let status = if *enabled { "ON / オン" } else { "OFF / オフ" };
                    ("Mosaic Artwork / モザイク", status.to_string())
                }
                SettingsItem::Close => ("Close / 閉じる", String::new()),
            };

            let line = if value.is_empty() {
                vec![Span::styled(
                    format!("  {}  ", label),
                    if is_selected {
                        Style::default()
                            .fg(theme.bg)
                            .bg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.primary)
                    },
                )]
            } else {
                vec![
                    Span::styled(
                        format!("  {}: ", label),
                        if is_selected {
                            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.dim)
                        },
                    ),
                    Span::styled(
                        format!(" {} ", value),
                        if is_selected {
                            Style::default()
                                .fg(theme.bg)
                                .bg(theme.accent)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                                .fg(theme.primary)
                                .bg(theme.dim)
                                .add_modifier(Modifier::BOLD)
                        },
                    ),
                ]
            };

            list_items.push(ListItem::new(Line::from(line)));
        }

        let list = List::new(list_items).block(Block::default());
        f.render_widget(list, inner);

        // Add help text at the bottom
        let help_text = "↑↓/jk: Navigate  │  Enter/Space: Select  │  Esc/S: Close";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme.dim))
            .alignment(Alignment::Center);

        let help_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height - 1,
            width: popup_area.width,
            height: 1,
        };

        f.render_widget(help, help_area);
    }
}

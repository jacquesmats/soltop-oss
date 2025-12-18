use std::sync::Arc;
use std::cmp::Reverse;
use std::time::Duration;
use anyhow::Result;
use tokio::sync::RwLock;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    Terminal,
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Row, Table, Cell, Paragraph},
};
use crate::stats::NetworkState;
use super::Theme;

/// Main TUI application
pub struct App {
    /// Reference to shared network state (updated by NetworkMonitor)
    network_state: Arc<RwLock<NetworkState>>,

    /// Whether the app should keep running
    pub running: bool,

    /// Currently selected row in the table
    pub selected_row: usize,

    cached_stats: Vec<ProgramStatsDisplay>,

    /// Theme configuration
    theme: Theme,
}

impl App {
    /// Create a new App with reference to network state
    pub fn new(network_state: Arc<RwLock<NetworkState>>) -> Self {
        Self {
            network_state,
            running: true,
            selected_row: 0,
            cached_stats: vec![],
            theme: Theme::flatline(),
        }
    }

    /// Update cached stats from network state
    async fn update_stats(&mut self) {
        self.cached_stats = self.get_stats().await;
    }
    
    /// Get cached stats for rendering
    fn get_cached_stats(&self) -> &[ProgramStatsDisplay] {
        &self.cached_stats
    }

    /// Run the main event loop
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // Tick rate: how often we update the UI
        let tick_rate = Duration::from_millis(500);
        let mut last_tick = tokio::time::Instant::now();
        
        loop {
            self.update_stats().await;

            // 1. Draw UI
            terminal.draw(|frame| {
                self.render(frame);
            })?;
            
            // 2. Handle events with timeout
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // Handle keyboard input
                    self.handle_key(key.code);
                }
            }
            
            // 3. Tick if enough time has elapsed
            if last_tick.elapsed() >= tick_rate {
                // This is where we'd update app state
                // (Currently no-op since data updates happen in background)
                last_tick = tokio::time::Instant::now();
            }
            
            // 4. Check exit condition
            if !self.running {
                break;
            }
        }
        
        Ok(())
    }

    /// Render the entire UI
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create main layout: header + table + footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Header
                Constraint::Min(10),        // Table (takes remaining space)
                Constraint::Length(1),      // Footer
            ])
            .split(area);

        // Render sections
        self.render_header(frame, chunks[0]);
        self.render_table(frame, chunks[1]);
        self.render_footer(frame, chunks[2]);
    }

    /// Render the header section
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        // Create header with neon green border
        let header_block = Block::default()
            .title(" soltop - Solana Network Monitor ")
            .borders(Borders::ALL)
            .border_style(self.theme.border_style())
            .title_style(self.theme.header_style());

        let inner = header_block.inner(area);
        frame.render_widget(header_block, area);

        // Split inner area for multiple info lines
        let info_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(inner);

        // Left side: Slot info (TODO: Get from network state when available)
        let rpc_text = Paragraph::new("Monitoring...")
            .style(self.theme.normal_style());
        frame.render_widget(rpc_text, info_chunks[0]);

        // Right side: Quit hint
        let hint_text = Paragraph::new("Press 'q' to quit")
            .style(self.theme.muted_style())
            .alignment(Alignment::Right);
        frame.render_widget(hint_text, info_chunks[1]);
    }

    /// Render the statistics table
    fn render_table(&self, frame: &mut Frame, area: Rect) {
        // Table header with neon green
        let header = Row::new(vec![
            Cell::from("Program ID"),
            Cell::from("Txs/s"),
            Cell::from("Total"),
            Cell::from("Success%"),
        ])
        .style(self.theme.table_header_style())
        .height(1);

        // Convert cached stats to color-coded rows
        let rows: Vec<Row> = self
            .get_cached_stats()
            .iter()
            .map(|stat| {
                // Color code based on metrics
                let tps_color = self.theme.tps_color(stat.tx_per_sec);
                let success_color = self.theme.success_rate_color(stat.success_rate);

                Row::new(vec![
                    // Program ID (truncated, muted color)
                    Cell::from(format!("{}...", &stat.program_id[..8]))
                        .style(Style::default().fg(self.theme.gray)),

                    // TPS (color coded: green=low, amber=medium, red=high)
                    Cell::from(format!("{:.1}", stat.tx_per_sec))
                        .style(Style::default().fg(tps_color)),

                    // Total (normal white)
                    Cell::from(format!("{}", stat.total_txs))
                        .style(self.theme.normal_style()),

                    // Success% (color coded: green>95%, amber>80%, red<80%)
                    Cell::from(format!("{:.1}%", stat.success_rate))
                        .style(Style::default().fg(success_color)),
                ])
            })
            .collect();

        // Table with border matching theme
        let table = Table::new(rows, vec![
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.border_style())
                .title(" Program Statistics ")
                .title_style(self.theme.header_style())
        );

        frame.render_widget(table, area);
    }

    /// Render the footer with function key hints
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        // htop-style function key hints
        let footer_text = vec![
            ("F1", "Help"),
            ("F2", "Setup"),
            ("F5", "Sort"),
            ("F9", "Filter"),
            ("F10", "Quit"),
        ];

        let spans: Vec<Span> = footer_text
            .iter()
            .flat_map(|(key, label)| {
                vec![
                    Span::styled(*key, self.theme.success_style()),  // Green F-key
                    Span::raw(format!("{} ", label)),                // White label
                    Span::raw(" "),
                ]
            })
            .collect();

        let footer = Paragraph::new(Line::from(spans))
            .style(Style::default().bg(self.theme.background));

        frame.render_widget(footer, area);
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.running = false;
            }
            KeyCode::Down => {
                // TODO: Move selection down (we'll implement this later)
            }
            KeyCode::Up => {
                // TODO: Move selection up (we'll implement this later)
            }
            _ => {}
        }
    }

    /// Get current network statistics
    async fn get_stats(&self) -> Vec<ProgramStatsDisplay> {
        let state = self.network_state.read().await;

        let mut display = Vec::new();

        for (program_id, stats) in state.programs.iter() {
            display.push(
                ProgramStatsDisplay {
                    program_id: program_id.clone(),
                    tx_per_sec: stats.transactions_per_second(),
                    total_txs: stats.total_transactions(),
                    success_rate: stats.success_rate(),
            });
        }
        
        // Sort by total_txs descending
        display.sort_by_key(|s| Reverse(s.total_txs));
        
        display
    }
}

/// Simplified struct for displaying program stats in UI
/// (We'll expand this as we build the table)
pub struct ProgramStatsDisplay {
    pub program_id: String,
    pub tx_per_sec: f64,
    pub total_txs: u32,
    pub success_rate: f64,
}
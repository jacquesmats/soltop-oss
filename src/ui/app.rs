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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, Cell, Paragraph},
};
use crate::stats::NetworkState;

/// Main TUI application
pub struct App {
    /// Reference to shared network state (updated by NetworkMonitor)
    network_state: Arc<RwLock<NetworkState>>,
    
    /// Whether the app should keep running
    pub running: bool,
    
    /// Currently selected row in the table
    pub selected_row: usize,

    cached_stats: Vec<ProgramStatsDisplay>,
}

impl App {
    /// Create a new App with reference to network state
    pub fn new(network_state: Arc<RwLock<NetworkState>>) -> Self {
        Self {
            network_state,
            running: true,
            selected_row: 0,
            cached_stats: vec![],
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
        
        // Create main layout: header + table
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Header
                Constraint::Min(0),         // Table (takes remaining space)
            ])
            .split(area);
        
        // Render header
        self.render_header(frame, chunks[0]);
        
        // Render table
        self.render_table(frame, chunks[1]);
    }

    /// Render the header section
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header_block = Block::default()
            .title("soltop - Solana Network Monitor")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        
        let inner = header_block.inner(area);
        frame.render_widget(header_block, area);
        
        // TODO: Add RPC URL, slot, uptime info here (next tutorial)
        let text = Paragraph::new("Press 'q' to quit")
            .style(Style::default().fg(Color::White));
        frame.render_widget(text, inner);
    }

    /// Render the statistics table
    fn render_table(&self, frame: &mut Frame, area: Rect) {
        let header = Row::new(vec![
            Cell::from("Program ID"),
            Cell::from("Txs/s"),
            Cell::from("Total"),
            Cell::from("Success%"),
        ])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        
        // Convert cached stats to table rows
        let rows: Vec<Row> = self
            .get_cached_stats()
            .iter()
            .map(|stat| {
                Row::new(vec![
                    // Truncate program ID to first 8 chars
                    Cell::from(format!("{}...", &stat.program_id[..8])),
                    // Format numbers nicely
                    Cell::from(format!("{:.1}", stat.tx_per_sec)),
                    Cell::from(format!("{}", stat.total_txs)),
                    Cell::from(format!("{:.1}%", stat.success_rate)),
                ])
            })
            .collect();
        
        let table = Table::new(rows, vec![
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Program Statistics"));
        
        frame.render_widget(table, area);
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
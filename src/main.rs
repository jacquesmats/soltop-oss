use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, Cell, Paragraph},
    Frame,
};
use std::time::Duration;
use std::io;
use clap::Parser;
use tokio;

use soltop::{NetworkMonitor, MonitorConfig};
use soltop::ui::App;

#[derive(Parser, Debug)]
#[command(name = "soltop")]
#[command(about = "Terminal UI for Solana network monitoring", long_about = None)]
struct Args {
    /// Enable verbose performance statistics
    #[arg(short, long)]
    verbose: bool,
    
    /// RPC endpoint URL
    #[arg(
        long,
        default_value = "https://api.mainnet-beta.solana.com",
        help = "RPC endpoint URL"
    )]
    rpc_url: String,

    /// Hide system programs (Vote, ComputeBudget, System)
    #[arg(long)]
    hide_system: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Create configuration
    let config = MonitorConfig {
        rpc_url: args.rpc_url,
        window_duration: Duration::from_secs(5 * 60),  // 5 minutes
        buffer_capacity: 750,
        poll_interval: Duration::from_millis(400),
    };
    
    // Create monitor
    let mut monitor = NetworkMonitor::new(config);
    
    // Get shared state reference for UI
    let network_state = monitor.get_state(); // Clone the Arc
    
    // Spawn stats printer task
    tokio::spawn(async move {
        if let Err(e) = monitor.start().await {
            eprintln!("Monitor error: {}", e);
        }
    });

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with the shared state
    let mut app = App::new(network_state);

    // Run event loop
    let result = run_app(&mut terminal, &mut app).await;

    // Cleanup: restore terminal
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    // Tick rate: how often we update the UI
    let tick_rate = Duration::from_millis(500);
    let mut last_tick = tokio::time::Instant::now();
    
    loop {
        app.update_stats().await;

        // 1. Draw UI
        terminal.draw(|frame| {
            render_ui(frame, app);
        })?;
        
        // 2. Handle events with timeout
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Handle keyboard input
                app.handle_key(key.code);
            }
        }
        
        // 3. Tick if enough time has elapsed
        if last_tick.elapsed() >= tick_rate {
            // This is where we'd update app state
            // (Currently no-op since data updates happen in background)
            last_tick = tokio::time::Instant::now();
        }
        
        // 4. Check exit condition
        if !app.running {
            break;
        }
    }
    
    Ok(())
}

fn render_ui(frame: &mut Frame, app: &App) {
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
    render_header(frame, chunks[0]);
    
    // Render table
    render_table(frame, app, chunks[1]);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect) {
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

fn render_table(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let header = Row::new(vec![
        Cell::from("Program ID"),
        Cell::from("Txs/s"),
        Cell::from("Total"),
        Cell::from("Success%"),
    ])
    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    
    // TODO: Convert cached stats to table rows
    let rows: Vec<Row> = app
        .get_cached_stats()
        .iter()
        .map(|stat| {
            Row::new(vec![
                // TODO: Truncate program ID to first 8 chars
                Cell::from(format!("{}...", &stat.program_id[..8])),
                // TODO: Format numbers nicely
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
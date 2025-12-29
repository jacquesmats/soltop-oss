use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub background: Color,
    pub border: Color,
    pub neon_green: Color,
    pub cyan: Color,
    pub amber: Color,
    pub success: Color,
    pub error: Color,
    pub white: Color,
    pub gray: Color,
}

impl Theme {
    pub const fn flatline() -> Self {
        Self {
            background: Color::Rgb(10, 10, 10),
            border: Color::Rgb(51, 51, 51),
            neon_green: Color::Rgb(0, 255, 0),
            cyan: Color::Rgb(34, 211, 238),
            amber: Color::Rgb(251, 191, 36),
            success: Color::Rgb(52, 211, 153),
            error: Color::Rgb(239, 68, 68),
            white: Color::Rgb(255, 255, 255),
            gray: Color::Rgb(156, 163, 175),
        }
    }

    // Style helpers
    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.neon_green)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn table_header_style(&self) -> Style {
        Style::default()
            .fg(self.neon_green)
            .add_modifier(Modifier::BOLD)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.amber)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn normal_style(&self) -> Style {
        Style::default().fg(self.white)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.gray)
    }

    // Get color based on success rate percentage
    pub fn success_rate_color(&self, rate: f64) -> Color {
        if rate >= 95.0 {
            self.success
        } else if rate >= 80.0 {
            self.amber
        } else {
            self.error
        }
    }

    // Get color based on TPS value
    pub fn tps_color(&self, tps: f64) -> Color {
        if tps >= 100.0 {
            self.error // Very high = might be spam
        } else if tps >= 10.0 {
            self.amber // Moderate activity
        } else {
            self.success // Low = normal
        }
    }

    // Get color based on CU/s (compute units per second)
    pub fn cu_per_sec_color(&self, cu_per_sec: f64) -> Color {
        if cu_per_sec >= 10_000_000.0 {
            // 10M+ CU/s
            self.error // Very high compute usage
        } else if cu_per_sec >= 1_000_000.0 {
            // 1M+ CU/s
            self.amber // Moderate compute usage
        } else {
            self.success // Low compute usage
        }
    }

    // Get color based on average CU per transaction
    pub fn avg_cu_color(&self, avg_cu: f64) -> Color {
        if avg_cu >= 200_000.0 {
            // 200K+ CU per tx
            self.error // Very compute-intensive
        } else if avg_cu >= 50_000.0 {
            // 50K+ CU per tx
            self.amber // Moderate
        } else {
            self.success // Low/efficient
        }
    }
}

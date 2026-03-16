use chrono::{Local, Timelike, Datelike};

pub struct QuietHoursConfig {
    pub enabled: bool,
    pub start_hour: u32,
    pub start_minute: u32,
    pub end_hour: u32,
    pub end_minute: u32,
    pub days: Vec<u32>, // 1=Mon to 7=Sun
}

impl QuietHoursConfig {
    pub fn is_active(&self) -> bool {
        if !self.enabled {
            return false;
        }
        let now = Local::now();
        let weekday = now.weekday().num_days_from_monday() + 1;
        if !self.days.contains(&weekday) {
            return false;
        }

        let current = now.hour() * 60 + now.minute();
        let start = self.start_hour * 60 + self.start_minute;
        let end = self.end_hour * 60 + self.end_minute;

        if start <= end {
            current >= start && current < end
        } else {
            // Crosses midnight (e.g., 22:00-08:00)
            current >= start || current < end
        }
    }
}

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub const DEFAULT_WINDOW_SECS: u64 = 60;
pub const SPIKE_THRESHOLD_TOKENS_PER_MIN: f32 = 75000.0;
pub const HIGH_THRESHOLD_TOKENS_PER_MIN: f32 = 35000.0;
pub const MEDIUM_THRESHOLD_TOKENS_PER_MIN: f32 = 10000.0;
pub const LOW_THRESHOLD_TOKENS_PER_MIN: f32 = 500.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionBurnRate {
    /// Rychlost spotřeby celkových tokenů za minutu (rolling window 60s)
    pub tokens_per_minute: f32,
    /// Rychlost růstu aktuálního kontextu v tokenech za minutu
    pub context_growth_per_minute: f32,
    /// Stupeň pálení: "idle" | "low" | "medium" | "high" | "spike"
    pub burn_level: String,
    /// HEX kód barvy pro zobrazení ve widgetu
    pub level_color: String,
    /// Odhadovaný počet minut do zaplnění limitu kontextu (např. 1M)
    pub est_minutes_to_context_limit: Option<u32>,
    /// Zda je detekován kritický skok / spike ve spotřebě (>75k tok/min)
    pub is_spike: bool,
    /// Krátký lidsky čitelný formátovaný text (např. "🔥 14.2k tok/min")
    pub formatted_rate: String,
}

impl Default for SessionBurnRate {
    fn default() -> Self {
        Self {
            tokens_per_minute: 0.0,
            context_growth_per_minute: 0.0,
            burn_level: "idle".to_string(),
            level_color: "#64748b".to_string(), // Slate
            est_minutes_to_context_limit: None,
            is_spike: false,
            formatted_rate: "Klid (0 tok/min)".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalBurnRadar {
    /// Celková rychlost spotřeby napříč všemi aktivními agenty (tok/min)
    pub total_tokens_per_minute: f32,
    /// Globální úroveň pálení: "idle" | "low" | "medium" | "high" | "spike"
    pub burn_level: String,
    pub level_color: String,
    /// Trend spotřeby: "idle" | "stable" | "accelerating" | "cooling"
    pub trend: String,
    /// Odhadovaný čas v minutách do vyčerpání 5h kvóty při současném tempu
    pub est_minutes_to_five_hour_limit: Option<u32>,
    /// Přehledný souhrn pro hlavičku widgetu
    pub radar_summary: String,
}

impl Default for GlobalBurnRadar {
    fn default() -> Self {
        Self {
            total_tokens_per_minute: 0.0,
            burn_level: "idle".to_string(),
            level_color: "#64748b".to_string(),
            trend: "idle".to_string(),
            est_minutes_to_five_hour_limit: None,
            radar_summary: "Klidový stav (0 tok/min)".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct TokenSample {
    timestamp: Instant,
    total_tokens: u64,
    context_tokens: u64,
}

pub struct BurnRateTracker {
    window_duration: Duration,
    session_samples: HashMap<u32, VecDeque<TokenSample>>,
    historical_global_rates: VecDeque<(Instant, f32)>,
}

impl BurnRateTracker {
    pub fn new() -> Self {
        Self {
            window_duration: Duration::from_secs(DEFAULT_WINDOW_SECS),
            session_samples: HashMap::new(),
            historical_global_rates: VecDeque::new(),
        }
    }

    pub fn with_window(window_secs: u64) -> Self {
        Self {
            window_duration: Duration::from_secs(window_secs),
            session_samples: HashMap::new(),
            historical_global_rates: VecDeque::new(),
        }
    }

    /// Zaznamená aktuální vzorek sezení a spočítá rychlost spotřeby
    pub fn record_and_evaluate_session(
        &mut self,
        pid: u32,
        total_tokens: u64,
        context_tokens: u64,
        context_limit: u64,
        now: Instant,
    ) -> SessionBurnRate {
        let samples = self.session_samples.entry(pid).or_insert_with(VecDeque::new);

        // Uložení nového vzorku
        samples.push_back(TokenSample {
            timestamp: now,
            total_tokens,
            context_tokens,
        });

        // Odstranění vzorků starších než definované okno
        while let Some(front) = samples.front() {
            if now.duration_since(front.timestamp) > self.window_duration {
                samples.pop_front();
            } else {
                break;
            }
        }

        if samples.len() < 2 {
            return SessionBurnRate::default();
        }

        let oldest = samples.front().unwrap();
        let newest = samples.back().unwrap();

        let elapsed_secs = newest.timestamp.duration_since(oldest.timestamp).as_secs_f32();
        if elapsed_secs < 1.0 {
            return SessionBurnRate::default();
        }

        let token_delta = newest.total_tokens.saturating_sub(oldest.total_tokens);
        let context_delta = newest.context_tokens.saturating_sub(oldest.context_tokens);

        let tokens_per_minute = ((token_delta as f32 / elapsed_secs) * 60.0 * 10.0).round() / 10.0;
        let context_growth_per_minute = ((context_delta as f32 / elapsed_secs) * 60.0 * 10.0).round() / 10.0;

        let (burn_level, level_color, is_spike) = if tokens_per_minute >= SPIKE_THRESHOLD_TOKENS_PER_MIN {
            ("spike", "#ef4444".to_string(), true) // Crimson red
        } else if tokens_per_minute >= HIGH_THRESHOLD_TOKENS_PER_MIN {
            ("high", "#ea580c".to_string(), false) // Orange red
        } else if tokens_per_minute >= MEDIUM_THRESHOLD_TOKENS_PER_MIN {
            ("medium", "#f59e0b".to_string(), false) // Amber orange
        } else if tokens_per_minute >= LOW_THRESHOLD_TOKENS_PER_MIN {
            ("low", "#10b981".to_string(), false) // Emerald green
        } else {
            ("idle", "#64748b".to_string(), false) // Slate gray
        };

        let est_minutes_to_context_limit = if context_growth_per_minute > 50.0 && context_tokens < context_limit {
            let rem_tokens = context_limit.saturating_sub(context_tokens);
            let mins = (rem_tokens as f32 / context_growth_per_minute).ceil() as u32;
            Some(mins)
        } else {
            None
        };

        let formatted_rate = if tokens_per_minute < 100.0 {
            "Klid (0 tok/min)".to_string()
        } else if tokens_per_minute >= 1000.0 {
            format!("{:.1}k tok/min", tokens_per_minute / 1000.0)
        } else {
            format!("{:.0} tok/min", tokens_per_minute)
        };

        SessionBurnRate {
            tokens_per_minute,
            context_growth_per_minute,
            burn_level: burn_level.to_string(),
            level_color,
            est_minutes_to_context_limit,
            is_spike,
            formatted_rate,
        }
    }

    /// Spočítá globální radar napříč všemi aktivními sezeními a vyhodnotí trend a 5h limit
    pub fn compute_global_radar(
        &mut self,
        session_burn_rates: &[SessionBurnRate],
        five_hour_quota_remaining_pct: f32,
        now: Instant,
    ) -> GlobalBurnRadar {
        let total_rate: f32 = ((session_burn_rates.iter().map(|r| r.tokens_per_minute).sum::<f32>()) * 10.0).round() / 10.0;

        // Uložit do historie globální rychlosti pro výpočet trendu
        self.historical_global_rates.push_back((now, total_rate));
        while let Some(front) = self.historical_global_rates.front() {
            if now.duration_since(front.0) > Duration::from_secs(120) {
                self.historical_global_rates.pop_front();
            } else {
                break;
            }
        }

        let trend = if total_rate < 500.0 {
            "idle".to_string()
        } else if self.historical_global_rates.len() >= 3 {
            let mid_point = now - Duration::from_secs(30);
            let older_avg = {
                let older: Vec<f32> = self.historical_global_rates.iter()
                    .filter(|(t, _)| *t < mid_point)
                    .map(|(_, r)| *r)
                    .collect();
                if older.is_empty() { total_rate } else { older.iter().sum::<f32>() / older.len() as f32 }
            };

            if total_rate > older_avg * 1.35 {
                "accelerating".to_string()
            } else if total_rate < older_avg * 0.75 {
                "cooling".to_string()
            } else {
                "stable".to_string()
            }
        } else {
            "stable".to_string()
        };

        let (burn_level, level_color) = if total_rate >= SPIKE_THRESHOLD_TOKENS_PER_MIN {
            ("spike", "#ef4444".to_string())
        } else if total_rate >= HIGH_THRESHOLD_TOKENS_PER_MIN {
            ("high", "#ea580c".to_string())
        } else if total_rate >= MEDIUM_THRESHOLD_TOKENS_PER_MIN {
            ("medium", "#f59e0b".to_string())
        } else if total_rate >= LOW_THRESHOLD_TOKENS_PER_MIN {
            ("low", "#10b981".to_string())
        } else {
            ("idle", "#64748b".to_string())
        };

        // Odhad vyčerpání 5h kvóty:
        // Orientačně předpokládáme 5h kapacitu ekvivalentní cca 1.5M tokenů (při 100% = 1.5M, 1% = 15k tokenů)
        let est_minutes_to_five_hour_limit = if total_rate > 2000.0 && five_hour_quota_remaining_pct > 0.0 {
            let remaining_estimated_tokens = five_hour_quota_remaining_pct * 15000.0;
            let mins = (remaining_estimated_tokens / total_rate).ceil() as u32;
            Some(mins)
        } else {
            None
        };

        let radar_summary = if total_rate < 500.0 {
            "Klidový stav (0 tok/min)".to_string()
        } else {
            let trend_icon = match trend.as_str() {
                "accelerating" => "↗ Rostoucí",
                "cooling" => "↘ Klesající",
                _ => "→ Stabilní",
            };
            format!("{:.1}k tok/min ({})", total_rate / 1000.0, trend_icon)
        };

        GlobalBurnRadar {
            total_tokens_per_minute: total_rate,
            burn_level: burn_level.to_string(),
            level_color,
            trend,
            est_minutes_to_five_hour_limit,
            radar_summary,
        }
    }

    /// Pročistí neaktivní PIDs ze sledování
    pub fn prune_inactive_sessions(&mut self, active_pids: &[u32]) {
        self.session_samples.retain(|pid, _| active_pids.contains(pid));
    }
}

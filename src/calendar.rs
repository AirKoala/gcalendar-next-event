use crate::config::Config;
use chrono::{prelude::*, TimeDelta};
use eyre::{eyre, Result};
use google_calendar::{
    types::{MinAccessRole, OrderBy},
    Client, StatusCode,
};
use serde::{Deserialize, Serialize};

pub struct Calendar<'a> {
    client: Client,
    config: &'a Config,
}

fn is_all_day(event: &google_calendar::types::Event) -> bool {
    if let Some(edt) = &event.start {
        if let Some(_) = edt.date_time {
            return false;
        }
    }

    if let Some(edt) = &event.end {
        if let Some(_) = edt.date_time {
            return false;
        }
    }

    true
}

impl<'a> Calendar<'a> {
    pub async fn new(config: &'a Config) -> Result<Calendar<'a>> {
        let mut client = Client::new(
            &config.creds.client_id,
            &config.creds.client_secret,
            "http://localhost:8080",
            &config.creds.token,
            &config.creds.refresh_token,
        );

        client.set_auto_access_token_refresh(true);
        client.refresh_access_token().await?;

        Ok(Self { client, config })
    }

    pub async fn get_next_event(self) -> Result<Option<Event>> {
        Ok(self.get_events().await?.first().cloned())
    }

    async fn get_events(&self) -> Result<Vec<Event>> {
        let mut events_cache = if self.config.nocache {
            self.events_cache_from_api().await?
        } else {
            match EventsCache::load_from_file() {
                Ok(cache) => cache,
                Err(_) => self.events_cache_from_api().await?,
            }
        };

        if events_cache.is_stale(TimeDelta::seconds(self.config.cache_duration_seconds)) {
            events_cache = self.events_cache_from_api().await?;
        }

        events_cache.save_to_file()?;

        Ok(events_cache.events)
    }

    async fn events_cache_from_api(&self) -> Result<EventsCache> {
        Ok(EventsCache::from_vec(self.fetch_events_from_api().await?))
    }

    async fn fetch_events_from_api(&self) -> Result<Vec<Event>> {
        let calendars = self
            .client
            .calendar_list()
            .list_all(MinAccessRole::Reader, false, false)
            .await?;

        if calendars.status != StatusCode::OK {
            return Err(eyre!("Failed to get calendar list"));
        }

        let calendars = calendars.body;

        let mut events = Vec::new();

        for calendar in calendars {
            #[rustfmt::skip]
            let cal_events = self
                .client
                .events()
                .list(
                    &calendar.id, "", 0, 5, OrderBy::StartTime, "", &[], "",
                    &[], false, false, true, "", &Utc::now().to_rfc3339(), "", "",
                )
                .await?;

            if cal_events.status != StatusCode::OK {
                return Err(eyre!("Failed to get events"));
            }

            for event in cal_events.body {
                if !is_all_day(&event) {
                    events.push(Event::from(event));
                }
            }
        }

        Ok(events)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EventsCache {
    pub events: Vec<Event>,
    last_updated: DateTime<Utc>,
}

impl EventsCache {
    pub fn is_stale(&self, cache_duration: TimeDelta) -> bool {
        Utc::now() - self.last_updated > cache_duration
    }

    pub fn load_from_file() -> Result<Self> {
        if !Self::get_cache_path()?.exists() {
            return Err(eyre!("Cache file does not exist"));
        }

        let cache = std::fs::read_to_string(Self::get_cache_path()?)?;
        Ok(serde_json::from_str(&cache)?)
    }

    pub fn save_to_file(&self) -> Result<()> {
        Ok(std::fs::write(
            Self::get_cache_path()?,
            serde_json::to_string_pretty(self)?,
        )?)
    }

    pub fn from_vec(events: Vec<Event>) -> Self {
        Self {
            events,
            last_updated: Utc::now(),
        }
    }

    fn get_cache_path() -> Result<std::path::PathBuf> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))?;

        if !xdg_dirs.get_cache_home().exists() {
            std::fs::create_dir_all(xdg_dirs.get_cache_home())?;
        }

        Ok(xdg_dirs.get_cache_home().join("events_cache.json"))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    title: String,
    location: Option<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

impl Event {
    pub fn format_status_line(&self) -> String {
        format!(
            "{}{}: {}",
            self.title,
            match &self.location {
                Some(loc) => format!(" [{}]", loc),
                None => "".to_string(),
            },
            self.start_time
                .with_timezone(&chrono::offset::Local)
                .format("%I:%M %p"),
        )
    }
}

impl From<google_calendar::types::Event> for Event {
    fn from(event: google_calendar::types::Event) -> Self {
        Self {
            title: event.summary,
            location: match event.location.is_empty() {
                true => None,
                false => Some(event.location),
            },
            start_time: event.start.unwrap().date_time.unwrap(),
            end_time: event.end.unwrap().date_time.unwrap(),
        }
    }
}

//! Conversions between domain entities and the wire types of the `types` crate.

use domain::{
    todo::{TodoItem, TodoList},
    user::User,
    weather::WeatherForecast,
};
use types::{
    todo::{TodoItemDto, TodoListDto},
    user::UserInfo,
    weather,
};

pub fn user_info(user: User) -> UserInfo {
    UserInfo {
        uuid: user.uuid,
        username: user.username,
        email: user.email,
        is_admin: user.is_admin,
    }
}

fn todo_item_dto(item: TodoItem) -> TodoItemDto {
    TodoItemDto {
        id: item.id,
        list_id: item.list_id,
        title: item.title,
        done: item.done,
        priority: item.priority,
        note: item.note,
    }
}

pub fn todo_list_dto(list: TodoList) -> TodoListDto {
    TodoListDto {
        id: list.id,
        title: list.title,
        colour: list.colour,
        items: list.items.into_iter().map(todo_item_dto).collect(),
    }
}

pub fn weather_forecast_dto(forecast: WeatherForecast) -> weather::WeatherForecast {
    weather::WeatherForecast::new(
        rfc3339(forecast.date_unix_secs),
        forecast.temperature_c,
        forecast.summary,
    )
}

/// Formats a unix timestamp (seconds) as an RFC 3339 UTC string.
pub fn rfc3339(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // civil-from-days (Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_unix_time_as_rfc3339() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(951_782_400), "2000-02-29T00:00:00Z"); // leap day
        assert_eq!(rfc3339(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn formats_end_of_day_and_year_boundaries() {
        assert_eq!(rfc3339(86_399), "1970-01-01T23:59:59Z");
        assert_eq!(rfc3339(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(rfc3339(951_868_800), "2000-03-01T00:00:00Z"); // day after leap day
    }

    /// The domain rules and the wire contract served to clients list the same lookups.
    #[test]
    fn domain_lookups_match_the_wire_contract() {
        assert_eq!(domain::todo::COLOURS, types::todo::COLOURS);
        assert_eq!(domain::todo::PRIORITY_LEVELS, types::todo::PRIORITY_LEVELS);
    }

    #[test]
    fn forecast_dto_has_rfc3339_date_and_fahrenheit() {
        let dto = weather_forecast_dto(WeatherForecast {
            date_unix_secs: 86_400,
            temperature_c: 100,
            summary: "Hot".into(),
        });
        assert_eq!(dto.date, "1970-01-02T00:00:00Z");
        assert_eq!(dto.temperature_f, 212);
    }
}

use colored::{ColoredString, Colorize};
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    pub static ref logger: Logger = Logger {
        logs: Mutex::new(HashMap::new())
    };
}

pub struct Logger {
    logs: Mutex<HashMap<String, Vec<String>>>,
}

pub enum LogType {
    Info,
    Error,
    Warn,
}

impl LogType {
    pub fn to_string(&self) -> ColoredString {
        match self {
            LogType::Info => "INFO".green(),
            LogType::Error => "ERROR".red(),
            LogType::Warn => "WARN".yellow(),
        }
    }
}

impl Logger {
    pub fn log(id: impl Into<String>, message: impl Into<String>, _type: LogType) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let id: String = id.into();
        let message = message.into();

        println!(
            "[{}] [{}] {} : {}",
            timestamp.cyan(),
            _type.to_string(),
            id.purple(),
            message,
        );

        let new_message = format!(
            "[{}] [{}] {} : {}",
            timestamp,
            _type.to_string().clear(),
            id,
            message,
        );
        let mut logs_lock = logger.logs.lock().unwrap();
        if let Some(logs) = logs_lock.get_mut(&id) {
            logs.push(new_message);
        } else {
            logs_lock.insert(id, vec![new_message]);
        }
    }

    pub fn info(id: impl Into<String>, message: impl Into<String>) {
        Logger::log(id, message, LogType::Info);
    }

    pub fn error(id: impl Into<String>, message: impl Into<String>) {
        Logger::log(id, message, LogType::Error);
    }

    pub fn warn(id: impl Into<String>, message: impl Into<String>) {
        Logger::log(id, message, LogType::Warn);
    }

    pub fn get_logs(id: String) -> Vec<String> {
        let logs_lock = logger.logs.lock().unwrap();
        if let Some(logs) = logs_lock.get(&id) {
            logs.clone()
        } else {
            vec![]
        }
    }

    pub fn get_all_logs() -> HashMap<String, Vec<String>> {
        logger.logs.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_and_get_logs() {
        let id = "test_id";
        let message1 = "Test message 1";
        let message2 = "Test message 2";

        Logger::info(id, message1);
        Logger::error(id, message2);

        let logs = Logger::get_logs(id.to_string());
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains(message1));
        assert!(logs[1].contains(message2));
        assert!(logs[0].contains("INFO"));
        assert!(logs[1].contains("ERROR"));
    }

    #[test]
    fn test_get_logs_empty() {
        let id = "non_existent_id";
        let logs = Logger::get_logs(id.to_string());
        assert!(logs.is_empty());
    }

    #[test]
    fn test_get_all_logs() {
        let id1 = "id1";
        let id2 = "id2";
        let message1 = "Message for id1";
        let message2 = "Message for id2";

        Logger::warn(id1, message1);
        Logger::info(id2, message2);

        let all_logs = Logger::get_all_logs();
        assert_eq!(all_logs.len(), 2);
        assert!(all_logs.get(id1).unwrap()[0].contains(message1));
        assert!(all_logs.get(id2).unwrap()[0].contains(message2));
        assert!(all_logs.get(id1).unwrap()[0].contains("WARN"));
        assert!(all_logs.get(id2).unwrap()[0].contains("INFO"));
    }

    #[test]
    fn test_multiple_logs_same_id() {
        let id = "multiple_logs_id";
        let messages = ["Message 1", "Message 2", "Message 3"];

        Logger::info(id, messages[0]);
        Logger::warn(id, messages[1]);
        Logger::error(id, messages[2]);

        let logs = Logger::get_logs(id.to_string());
        assert_eq!(logs.len(), 3);
        assert!(logs[0].contains("INFO"));
        assert!(logs[1].contains("WARN"));
        assert!(logs[2].contains("ERROR"));
    }

    #[test]
    fn test_different_id_types() {
        let string_id = String::from("string_id");
        let str_id = "str_id";
        let int_id = 42;

        Logger::info(string_id.clone(), "String ID message");
        Logger::warn(str_id, "Str ID message");
        Logger::error(int_id.to_string(), "Int ID message");

        let all_logs = Logger::get_all_logs();
        assert_eq!(all_logs.len(), 3);
        assert!(all_logs.get(&string_id).unwrap()[0].contains("String ID message"));
        assert!(all_logs.get(str_id).unwrap()[0].contains("Str ID message"));
        assert!(all_logs.get(&int_id.to_string()).unwrap()[0].contains("Int ID message"));
    }

    #[test]
    fn test_log_with_owned_types() {
        let id = String::from("owned_id");
        let message = String::from("Owned message");

        Logger::info(id.clone(), message.clone());

        let logs = Logger::get_logs(id);
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains(&message));
    }
}

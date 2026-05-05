use crate::model::task::Task;
use chrono::Local;

#[allow(dead_code)]
pub fn check_reminders(tasks: &[Task]) -> Vec<String> {
    let now = Local::now();
    let mut triggered = Vec::new();

    for task in tasks {
        for reminder in &task.reminders {
            if !reminder.acknowledged && reminder.trigger_at <= now {
                triggered.push(task.id.clone());
                break;
            }
        }
    }

    triggered
}

#[allow(dead_code)]
pub fn next_reminder(tasks: &[Task]) -> Option<(String, chrono::DateTime<Local>)> {
    let now = Local::now();
    let mut earliest: Option<(String, chrono::DateTime<Local>)> = None;

    for task in tasks {
        for reminder in &task.reminders {
            if !reminder.acknowledged && reminder.trigger_at > now {
                match &earliest {
                    None => earliest = Some((task.id.clone(), reminder.trigger_at)),
                    Some((_, t)) if reminder.trigger_at < *t => {
                        earliest = Some((task.id.clone(), reminder.trigger_at));
                    }
                    _ => {}
                }
            }
        }
    }

    earliest
}

#[allow(dead_code)]
pub fn format_relative_time(target: chrono::DateTime<Local>) -> String {
    let now = Local::now();
    let diff = target.signed_duration_since(now);

    let total_secs = diff.num_seconds();
    if total_secs < 0 {
        let abs_secs = total_secs.unsigned_abs();
        if abs_secs < 60 {
            return format!("{}s ago", abs_secs);
        } else if abs_secs < 3600 {
            return format!("{}m ago", abs_secs / 60);
        } else if abs_secs < 86400 {
            return format!("{}h ago", abs_secs / 3600);
        }
        return format!("{}d ago", abs_secs / 86400);
    }

    if total_secs < 60 {
        format!("in {}s", total_secs)
    } else if total_secs < 3600 {
        format!("in {}m", total_secs / 60)
    } else if total_secs < 86400 {
        format!("in {}h", total_secs / 3600)
    } else {
        let days = total_secs / 86400;
        if days == 1 {
            "tomorrow".to_string()
        } else {
            format!("in {}d", days)
        }
    }
}

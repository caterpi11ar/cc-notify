use crate::models::NotificationMessage;

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn render(template: &str, message: &NotificationMessage) -> String {
        let mut result = template.to_string();
        result = result.replace("{{event}}", &message.event);
        result = result.replace(
            "{{message}}",
            message.message.as_deref().unwrap_or(""),
        );
        result = result.replace("{{tool}}", message.tool.as_deref().unwrap_or(""));
        result = result.replace(
            "{{session_id}}",
            message.session_id.as_deref().unwrap_or(""),
        );
        result = result.replace(
            "{{project}}",
            message.project.as_deref().unwrap_or(""),
        );
        result = result.replace(
            "{{timestamp}}",
            &chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        );
        if let Some(event_type) = &message.event_type {
            result = result.replace("{{event_type}}", event_type);
        }
        result
    }
}

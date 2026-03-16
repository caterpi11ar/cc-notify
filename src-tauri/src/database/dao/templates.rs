use super::super::{lock_conn, Database};
use crate::error::AppError;
use crate::models::Template;
use rusqlite::OptionalExtension;

impl Database {
    /// Get all templates
    pub fn get_all_templates(&self) -> Result<Vec<Template>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, channel_type, body_template, format, is_default
                 FROM templates ORDER BY channel_type, name",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    channel_type: row.get(2)?,
                    body_template: row.get(3)?,
                    format: row.get(4)?,
                    is_default: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(templates)
    }

    /// Get a single template by ID
    pub fn get_template(&self, id: &str) -> Result<Option<Template>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, channel_type, body_template, format, is_default
                 FROM templates WHERE id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    channel_type: row.get(2)?,
                    body_template: row.get(3)?,
                    format: row.get(4)?,
                    is_default: row.get(5)?,
                })
            })
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result)
    }

    /// Get templates by channel type
    pub fn get_templates_by_channel_type(
        &self,
        channel_type: &str,
    ) -> Result<Vec<Template>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, channel_type, body_template, format, is_default
                 FROM templates WHERE channel_type = ?1 ORDER BY is_default DESC, name",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![channel_type], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    channel_type: row.get(2)?,
                    body_template: row.get(3)?,
                    format: row.get(4)?,
                    is_default: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(templates)
    }

    /// Get default template for a channel type
    pub fn get_default_template(
        &self,
        channel_type: &str,
    ) -> Result<Option<Template>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, channel_type, body_template, format, is_default
                 FROM templates WHERE channel_type = ?1 AND is_default = 1 LIMIT 1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![channel_type], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    channel_type: row.get(2)?,
                    body_template: row.get(3)?,
                    format: row.get(4)?,
                    is_default: row.get(5)?,
                })
            })
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result)
    }

    /// Insert a new template
    pub fn insert_template(&self, template: &Template) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT INTO templates (id, name, channel_type, body_template, format, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                template.id,
                template.name,
                template.channel_type,
                template.body_template,
                template.format,
                template.is_default,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update an existing template
    pub fn update_template(&self, template: &Template) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "UPDATE templates SET name = ?1, channel_type = ?2, body_template = ?3, format = ?4, is_default = ?5
             WHERE id = ?6",
            rusqlite::params![
                template.name,
                template.channel_type,
                template.body_template,
                template.format,
                template.is_default,
                template.id,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete a template by ID (only non-default)
    pub fn delete_template(&self, id: &str) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute(
                "DELETE FROM templates WHERE id = ?1 AND is_default = 0",
                rusqlite::params![id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(affected > 0)
    }
}

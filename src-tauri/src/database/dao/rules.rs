use super::super::{lock_conn, Database};
use crate::error::AppError;
use crate::models::Rule;
use rusqlite::OptionalExtension;

impl Database {
    /// Get all rules
    pub fn get_all_rules(&self) -> Result<Vec<Rule>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, rule_type, pattern, event_type_id, enabled, created_at
                 FROM rules ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Rule {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    rule_type: row.get(2)?,
                    pattern: row.get(3)?,
                    event_type_id: row.get(4)?,
                    enabled: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(rules)
    }

    /// Get a single rule by ID
    pub fn get_rule(&self, id: &str) -> Result<Option<Rule>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, rule_type, pattern, event_type_id, enabled, created_at
                 FROM rules WHERE id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(Rule {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    rule_type: row.get(2)?,
                    pattern: row.get(3)?,
                    event_type_id: row.get(4)?,
                    enabled: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result)
    }

    /// Insert a new rule
    pub fn insert_rule(&self, rule: &Rule) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT INTO rules (id, name, rule_type, pattern, event_type_id, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                rule.id,
                rule.name,
                rule.rule_type,
                rule.pattern,
                rule.event_type_id,
                rule.enabled,
                rule.created_at,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update an existing rule
    pub fn update_rule(&self, rule: &Rule) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "UPDATE rules SET name = ?1, rule_type = ?2, pattern = ?3, event_type_id = ?4, enabled = ?5
             WHERE id = ?6",
            rusqlite::params![
                rule.name,
                rule.rule_type,
                rule.pattern,
                rule.event_type_id,
                rule.enabled,
                rule.id,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete a rule by ID
    pub fn delete_rule(&self, id: &str) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute("DELETE FROM rules WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(affected > 0)
    }

    /// Toggle rule enabled status
    pub fn toggle_rule(&self, id: &str, enabled: bool) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "UPDATE rules SET enabled = ?1 WHERE id = ?2",
            rusqlite::params![enabled, id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get enabled rules only
    pub fn get_enabled_rules(&self) -> Result<Vec<Rule>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, rule_type, pattern, event_type_id, enabled, created_at
                 FROM rules WHERE enabled = 1 ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Rule {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    rule_type: row.get(2)?,
                    pattern: row.get(3)?,
                    event_type_id: row.get(4)?,
                    enabled: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(rules)
    }

    /// Get rules by event type
    pub fn get_rules_by_event_type(&self, event_type_id: &str) -> Result<Vec<Rule>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, rule_type, pattern, event_type_id, enabled, created_at
                 FROM rules WHERE event_type_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![event_type_id], |row| {
                Ok(Rule {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    rule_type: row.get(2)?,
                    pattern: row.get(3)?,
                    event_type_id: row.get(4)?,
                    enabled: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(rules)
    }
}

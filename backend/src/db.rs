use sqlx::{postgres::PgPool, Postgres, Transaction};

/// Runs the idempotent schema migration on startup. Statements are executed
/// one at a time inside a transaction so a partial failure never leaves a
/// half-applied schema.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    let schema = include_str!("migrations.sql");
    let statements = split_statements(schema);
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;
    for statement in &statements {
        sqlx::query(statement.as_str())
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("migration statement failed:\n{statement}\n-> {e}"))?;
    }
    tx.commit().await?;
    tracing::info!("schema migration applied");
    Ok(())
}

fn split_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("{s};"))
        .collect()
}

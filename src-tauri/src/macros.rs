/// Создаёт таблицы для всех переданных Entity
/// Использование: create_tables!(db, NovelsEntity, ChapterEntity);
#[macro_export]
macro_rules! create_tables {
    ($db:expr, $($entity:ident),+ $(,)?) => {

            let db_sqlite = sea_orm::DatabaseBackend::Sqlite;
            let schema = sea_orm::Schema::new(db_sqlite);

            $(
                {
                    let stmt = db_sqlite.build(&schema.create_table_from_entity($entity));
                    match $db.execute_raw(stmt).await {
                        Ok(o) => println!("(success) {:?}",o),
                        Err(e) => println!("(error) {:?}",e),
                    };
                }
            )+
    };
}

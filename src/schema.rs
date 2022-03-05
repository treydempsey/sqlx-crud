pub struct Metadata<'s, const C: usize> {
    pub table_name: &'s str,
    pub id_column: &'s str,
    pub columns: [&'s str; C],
    pub select_sql: &'s str,
    pub select_by_id_sql: &'s str,
    pub insert_sql: &'s str,
    pub update_by_id_sql: &'s str,
    pub delete_by_id_sql: &'s str,
}

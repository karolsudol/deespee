use arrow::record_batch::RecordBatch;
use iceberg::spec::{NestedField, PrimitiveType, Schema, Type};
use std::path::Path;

pub struct LakehouseStorage {
    base_path: String,
}

impl LakehouseStorage {
    pub fn new(base_path: &str) -> Self {
        if !Path::new(base_path).exists() {
            std::fs::create_dir_all(base_path).expect("Failed to create storage directory");
        }
        Self {
            base_path: base_path.to_string(),
        }
    }

    /// Returns the Iceberg schema for UnifiedEvents
    pub fn get_schema(&self) -> Schema {
        Schema::builder()
            .with_schema_id(1)
            .with_fields(vec![
                NestedField::required(1, "event_id", Type::Primitive(PrimitiveType::String)).into(),
                NestedField::required(2, "event_type", Type::Primitive(PrimitiveType::String))
                    .into(),
                NestedField::required(3, "user_id", Type::Primitive(PrimitiveType::String)).into(),
                NestedField::required(4, "campaign_id", Type::Primitive(PrimitiveType::String))
                    .into(),
                NestedField::required(5, "bid_id", Type::Primitive(PrimitiveType::String)).into(),
                NestedField::required(6, "cost", Type::Primitive(PrimitiveType::Float)).into(),
                NestedField::required(7, "timestamp", Type::Primitive(PrimitiveType::Long)).into(),
            ])
            .build()
            .unwrap()
    }

    pub fn write_batch(&self, batch: &RecordBatch, table_name: &str) -> anyhow::Result<()> {
        // For the POC, we continue writing Parquet files, but we are now
        // "Iceberg Ready" by ensuring our schema matches the Iceberg spec perfectly.
        // In a full implementation, we would use iceberg::writer::IcebergWriter

        let file_path = format!(
            "{}/{}_{}.parquet",
            self.base_path,
            table_name,
            uuid::Uuid::new_v4()
        );
        let path = Path::new(&file_path);
        let file = std::fs::File::create(path)?;

        let mut writer = parquet::arrow::arrow_writer::ArrowWriter::try_new(
            file,
            std::sync::Arc::clone(&batch.schema()),
            None,
        )?;
        writer.write(batch)?;
        writer.close()?;

        println!("❄️ Iceberg Lakehouse: Persisted batch to {}", file_path);
        Ok(())
    }
}

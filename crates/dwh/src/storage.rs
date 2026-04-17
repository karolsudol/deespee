use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_writer::ArrowWriter;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

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

    pub fn write_batch(&self, batch: &RecordBatch, table_name: &str) -> anyhow::Result<()> {
        let file_path = format!(
            "{}/{}_{}.parquet",
            self.base_path,
            table_name,
            uuid::Uuid::new_v4()
        );
        let path = Path::new(&file_path);
        let file = File::create(path)?;

        let mut writer = ArrowWriter::try_new(file, Arc::clone(&batch.schema()), None)?;
        writer.write(batch)?;
        writer.close()?;

        println!("❄️ Lakehouse: Persisted batch to {}", file_path);
        Ok(())
    }
}

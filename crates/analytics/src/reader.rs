use arrow_json::ArrayWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;

pub struct LakehouseReader {
    base_path: String,
}

impl LakehouseReader {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
        }
    }

    pub fn read_all_events(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let mut all_results = Vec::new();
        let paths = std::fs::read_dir(&self.base_path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "parquet"));

        for path in paths {
            if let Ok(file) = File::open(path) {
                if let Ok(builder) = ParquetRecordBatchReaderBuilder::try_new(file) {
                    if let Ok(mut reader) = builder.build() {
                        while let Some(Ok(batch)) = reader.next() {
                            let mut buf = Vec::new();
                            let mut writer = ArrayWriter::new(&mut buf);
                            writer.write(&batch)?;
                            writer.finish()?;

                            if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&buf)
                            {
                                if let Some(arr) = json_val.as_array() {
                                    all_results.extend(arr.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(all_results)
    }
}

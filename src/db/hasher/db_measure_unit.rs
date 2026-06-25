use sqlx::{MySql, Pool};
use crate::messages::requests::measure_request::{HashedValue};
use crate::logger::printers;

struct ForFlush{
    id: i32,
    value: HashedValue,
}

pub struct DbMeasureUnit {
    pool: Pool<MySql>,
    buff : Vec<ForFlush>,
    buff_errored : bool,
}

impl DbMeasureUnit {
    pub fn new(pool: Pool<MySql>) -> Self {
        DbMeasureUnit {
            buff : Vec::new(),
            buff_errored : false,
            pool
        }
    }
    pub async fn get_measures(&self, val_id: i32, from: i64, to: i64) -> Result<Vec<HashedValue>, String> {

        if self.buff_errored {
            return Err("Проблеми з архівацією даних".to_string()); // якшо в мене не получилось зберегти транзакцію, нада хоть якось це клієнту довести
        }
        let measures = sqlx::query_as::<_, HashedValue>(
            "
            SELECT
            measure_value as val,
            measure_time as timestamp
            FROM measures
            WHERE value_id = ?
            AND measure_time >= ?
            AND measure_time <= ?
            ORDER BY measure_time
            "
        )
            .bind(val_id)
            .bind(from)
            .bind(to)
            .fetch_all(&self.pool)
            .await
            .map_err(|e|{
                let msg = format!("Помилка читання вимірів із бази даних: {:?}", e);
                printers::err(msg.clone());
                msg
            })?;
        Ok(measures)
    }
    pub async fn save_value(&mut self, val_id: i32, val: HashedValue) -> Result<(), ()> {
        let saved_value = ForFlush {id: val_id, value: val};
        self.buff.push(saved_value);
        if self.buff.len() > 20 {
            self.flush().await?;
        }
        Ok(())
    }
    async fn flush(&mut self) -> Result<(), ()> {
        if self.buff.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| {
                self.buff_errored = true;
                printers::err(format!("Помилка відкриття транзакції для збереження буферу вимірів: {}", e));
                ()
            })?;

        if self.buff_errored {self.buff_errored = false;}
        for m in &self.buff {
            sqlx::query(
                "INSERT INTO measures
            (value_id, measure_value, measure_time)
            VALUES (?, ?, ?)"
            )
                .bind(m.id)
                .bind(m.value.val)
                .bind(m.value.timestamp)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    self.buff_errored = true;
                    printers::err(format!("Помилка проведення транзакції для збереження буферу вимірів: {}", e));
                    ()
                })?;
        }
        tx.commit().await.map_err(|e| {
            self.buff_errored = true;
            printers::err(format!("Помилка завершення транзакції для збереження буферу вимірів: {}", e));
            ()
        })?;
        self.buff.clear();
        Ok(())
    }
}
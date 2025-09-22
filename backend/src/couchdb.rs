use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone)]
pub struct CouchDb {
    client: Client,
    base_url: String,
    username: String,
    password: String,
}

impl CouchDb {
    pub fn new(base_url: &str, username: &str, password: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub async fn create_db(&self, db: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/{}", self.base_url, db);
        self.client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn delete_db(&self, db: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/{}", self.base_url, db);
        self.client
            .delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        // Ignore error_for_status here to allow deleting non-existent DB without failing
        Ok(())
    }

    pub async fn reset_db(&self, db: &str) -> Result<(), reqwest::Error> {
        // Best-effort delete, then create
        let _ = self.delete_db(db).await;
        self.create_db(db).await
    }

    pub async fn add_doc<T: Serialize>(&self, db: &str, id: &str, doc: &T) -> Result<(), reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url, db, id);
        self.client
            .put(&url) // Use PUT with ID for deterministic IDs
            .basic_auth(&self.username, Some(&self.password))
            .json(doc)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_doc<T: for<'de> Deserialize<'de>>(
        &self,
        db: &str,
        id: &str,
    ) -> Result<T, reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url, db, id);
        let res = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?;
        Ok(res)
    }

    pub async fn get_doc_with_rev<T: for<'de> Deserialize<'de>>(
        &self,
        db: &str,
        id: &str,
    ) -> Result<(T, String), reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url, db, id);
        let value = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let rev = value
            .get("_rev")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let doc: T = serde_json::from_value(value).unwrap();
        Ok((doc, rev))
    }

    pub async fn update_doc<T: Serialize>(
        &self,
        db: &str,
        id: &str,
        rev: &str,
        doc: &T,
    ) -> Result<(), reqwest::Error> {
        let mut value = serde_json::to_value(doc).expect("serialize doc");
        if let Value::Object(ref mut map) = value {
            map.insert("_rev".to_string(), Value::String(rev.to_string()));
        }
        let url = format!("{}/{}/{}", self.base_url, db, id);
        self.client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&value)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn list_docs<T: for<'de> Deserialize<'de>>(
        &self,
        db: &str,
    ) -> Result<Vec<T>, reqwest::Error> {
        let url = format!("{}/_all_docs?include_docs=true", format!("{}/{}", self.base_url, db));
        let res = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let mut herbs = Vec::new();
        if let Some(rows) = res.get("rows").and_then(|v| v.as_array()) {
            for row in rows {
                if let Some(doc) = row.get("doc") {
                    let herb: T = serde_json::from_value(doc.clone()).unwrap();
                    herbs.push(herb);
                }
            }
        }
        Ok(herbs)
    }

    pub async fn delete_doc(&self, db: &str, id: &str) -> Result<(), reqwest::Error> {
        let doc_url = format!("{}/{}/{}", self.base_url, db, id);
        let res = self.client
            .get(&doc_url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        if let Some(rev) = res.get("_rev").and_then(|v| v.as_str()) {
            let delete_url = format!("{}?rev={}", doc_url, rev);
            self.client
                .delete(&delete_url)
                .basic_auth(&self.username, Some(&self.password))
                .send()
                .await?
                .error_for_status()?;
        }

        Ok(())
    }
}
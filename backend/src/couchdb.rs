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

    pub async fn add_doc<T: Serialize>(&self, db: &str, doc: &T) -> Result<(), reqwest::Error> {
        let url = format!("{}/{}", self.base_url, db);
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
}